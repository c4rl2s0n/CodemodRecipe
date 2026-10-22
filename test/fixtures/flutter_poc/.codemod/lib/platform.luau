local platform = {}

function platform.path(context)
	if context.source and context.source.path then
		return context.source.path
	end
	return context.path
end

function platform.source(path, span)
	local source = { path = path }
	if span then
		source.span = span
	end
	return source
end

function platform.context(opts)
	return {
		source = opts.source,
		parent = opts.parent,
		role = opts.role,
		hints = opts.hints,
	}
end

function platform.child(c, componentId, context)
	local node = c:component(componentId):parse(context)
	if not node then
		return nil
	end
	node.componentId = componentId
	if context.role then
		node.role = context.role
	end
	return node
end

function platform.findChild(parsed, componentId, role)
	for _, child in ipairs(parsed.children or {}) do
		if child.componentId == componentId and (not role or child.role == role) then
			return child
		end
	end
	return nil
end

return platform
