local function path(context)
	if context.source and context.source.path then
		return context.source.path
	end
	return context.path
end

local function source(path, span)
	local s = { path = path }
	if span then
		s.span = span
	end
	return s
end

local function parseContext(opts)
	return {
		source = opts.source,
		parent = opts.parent,
		role = opts.role,
		hints = opts.hints,
	}
end

local function childParse(c, componentId, context)
	local node = c:component(componentId):parse(context)
	if not node then
		return nil
	end
	node.componentId = componentId
	if context.role then
		node.role = context.role
	end
	node.parseContext = context
	return node
end

return {
	id = "flutter.model",
	description = "Immutable model classes under lib/models",
	identify = function(c)
		local out = {}
		for _, row in ipairs(c:matchFiles("lib/models/{name}.dart")) do
			out[#out + 1] = { path = row.path, name = row.name }
		end
		return out
	end,
	parse = function(c, locator)
		local filePath = path(locator)
		if not filePath then
			return nil
		end

		local classHits = c:query({
			path = filePath,
			language = "dart",
			query = "flutter.model_class",
			capture = "className",
		})
		local className = (#classHits > 0 and classHits[1].text) or c:filename(filePath)
		local modelId = locator.name or c:filename(filePath)
		local identity = {
			componentId = "flutter.model",
			id = modelId,
			name = className,
			path = filePath,
		}

		local children = {}
		local fieldHits = c:query({
			path = filePath,
			language = "dart",
			query = "flutter.model_field",
			capture = "fieldName",
		})
		for _, hit in ipairs(fieldHits) do
			local child = childParse(c, "flutter.property", parseContext({
				source = source(filePath, { from = hit.span.from, to = hit.span.to }),
				parent = identity,
				role = "field",
				hints = { name = hit.text },
			}))
			if child then
				children[#children + 1] = child
			end
		end

		local methodHits = c:query({
			path = filePath,
			language = "dart",
			query = "flutter.model_method",
			capture = "methodName",
		})
		for _, hit in ipairs(methodHits) do
			local child = childParse(c, "flutter.method", parseContext({
				source = source(filePath, { from = hit.span.from, to = hit.span.to }),
				parent = identity,
				role = "method",
				hints = { name = hit.text },
			}))
			if child then
				children[#children + 1] = child
			end
		end

		return {
			id = modelId,
			name = className,
			componentId = "flutter.model",
			source = source(filePath),
			children = children,
		}
	end,
	verify = function(_c, instance, graph)
		local findings = {}
		local ast = instance.ast
		local properties = {}
		for _, child in ipairs(ast.children or {}) do
			if child.componentId == "flutter.property" then
				properties[#properties + 1] = child
			end
		end
		if #properties == 0 then
			findings[#findings + 1] = {
				severity = "warning",
				code = "model.no_fields",
				message = "model has no fields",
				instanceId = instance.id,
			}
		end

		local storeEdges = graph:incoming(instance.id, "stores")
		if #storeEdges == 0 then
			findings[#findings + 1] = {
				severity = "warning",
				code = "model.missing_store",
				message = "model has no store",
				instanceId = instance.id,
			}
			return findings
		end

		local store = graph:resolveFrom(storeEdges[1])
		local storeAst = store and store.ast or {}
		local storePath = storeAst.source and storeAst.source.path or "unknown"

		local storeFields = {}
		for _, field in ipairs(storeAst.fields or {}) do
			storeFields[field.name] = true
		end
		for _, field in ipairs(properties) do
			if not storeFields[field.name] then
				findings[#findings + 1] = {
					severity = "warning",
					code = "model.field_missing_in_store",
					message = "field " .. field.name .. " is not present in store " .. storePath,
					instanceId = instance.id,
				}
			end
		end

		local modelFields = {}
		for _, field in ipairs(properties) do
			modelFields[field.name] = true
		end
		for _, field in ipairs(storeAst.fields or {}) do
			if not modelFields[field.name] then
				findings[#findings + 1] = {
					severity = "warning",
					code = "state.field_missing_for_model",
					message = "model field " .. field.name .. " is not tracked in store " .. storePath,
					instanceId = instance.id,
				}
			end
		end
		return findings
	end,
	toSummary = function(_c, instance)
		local ast = instance.ast
		local fieldNames = {}
		local methodCount = 0
		for _, child in ipairs(ast.children or {}) do
			if child.componentId == "flutter.property" then
				fieldNames[#fieldNames + 1] = child.name
			elseif child.componentId == "flutter.method" then
				methodCount += 1
			end
		end
		local facts = {
			#fieldNames .. " properties",
			methodCount .. " methods",
		}
		return {
			kind = "model",
			title = ast.name,
			id = ast.id,
			path = ast.source and ast.source.path or nil,
			facts = facts,
			fields = fieldNames,
		}
	end,
	scaffold = {
		args = {
			{ name = "name", required = true },
		},
		apply = function(c, args)
			local name = args.name
			if not name or name == "" then
				error("scaffold requires args.name")
			end
			local id = string.lower(name:sub(1, 1)) .. name:sub(2)
			id = id:gsub("(%u)", function(ch)
				return "_" .. string.lower(ch)
			end)
			if id:sub(1, 1) == "_" then
				id = id:sub(2)
			end
			local modelPath = "lib/models/" .. id .. ".dart"
			local storePath = "lib/stores/" .. id .. "_store.dart"
			c:create({
				path = modelPath,
				content = "class "
					.. name
					.. " {\n  final String id;\n\n  "
					.. name
					.. "({required this.id});\n}\n",
			})
			c:create({
				path = storePath,
				content = "class " .. name .. "Store {\n  String id = '';\n}\n",
			})
		end,
	},
	operations = {
		addStoreField = {
			args = {
				{ name = "fieldName", required = true },
			},
			apply = function(c, args, target, graph)
				local fieldName = args.fieldName or args.name
				if not fieldName or fieldName == "" then
					error("addStoreField requires args.fieldName")
				end
				local storeEdges = graph:incoming(target.id, "stores")
				local store = storeEdges[1] and graph:resolveFrom(storeEdges[1]) or nil
				local storePath = store and store.ast.source and store.ast.source.path or nil
				if not storePath then
					error("model instance has no store relation with source.path")
				end
				c:insert({
					path = storePath,
					language = "dart",
					query = "flutter.store_body",
					capture = "classBody",
					anchor = "end",
					text = "\n  String " .. fieldName .. " = '';",
				})
			end,
		},
	},
}
