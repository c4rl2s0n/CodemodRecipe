(function () {
  const vscode = acquireVsCodeApi();
  const state = window.__CODEMOD_RECIPE_STATE__;

  const recipe = state.recipe;
  const recipes = state.recipes;
  const discoveryError = state.discoveryError;
  const initialArgs = state.initialArgs;
  let activeTab = state.activeTab;
  let files = [];
  let activeChangeIndex = 0;
  let previewInFlight = false;

  const byId = (id) => document.getElementById(id);

  const elements = {
    params: byId('params'),
    form: byId('form'),
    recipeList: byId('recipeList'),
    recipesPane: byId('recipesPane'),
    runnerPane: byId('runnerPane'),
    recipesTab: byId('recipesTab'),
    runnerTab: byId('runnerTab'),
    templates: byId('templates'),
    review: byId('review'),
    files: byId('files'),
    error: byId('error'),
    previewButton: byId('previewBtn'),
    applyButton: byId('applyBtn'),
    backButton: byId('backBtn'),
    previousButton: byId('prevBtn'),
    nextButton: byId('nextBtn'),
    openActiveButton: byId('openActiveBtn'),
  };

  function post(type, payload) {
    vscode.postMessage({ type, ...payload });
  }

  function renderTabs() {
    elements.recipesPane.classList.toggle('hidden', activeTab !== 'recipes');
    elements.runnerPane.classList.toggle('hidden', activeTab !== 'runner');
    elements.recipesTab.classList.toggle('active', activeTab === 'recipes');
    elements.runnerTab.classList.toggle('active', activeTab === 'runner');
  }

  function renderRecipeList() {
    elements.recipeList.innerHTML = '';
    if (!recipes.length) {
      elements.recipeList.appendChild(createEmptyState());
      return;
    }

    for (const item of recipes) {
      const button = document.createElement('button');
      button.className = 'recipe-button secondary';
      button.onclick = () => post('selectRecipe', { id: item.id });

      const name = document.createElement('span');
      name.className = 'recipe-title';
      name.textContent = item.name;

      const desc = document.createElement('span');
      desc.className = 'recipe-desc';
      desc.textContent = item.description || item.args.map((arg) => arg.name).join(', ');

      button.appendChild(name);
      button.appendChild(desc);
      elements.recipeList.appendChild(button);
    }
  }

  function createEmptyState() {
    const empty = document.createElement('div');
    empty.className = 'empty-state';

    const message = document.createElement('p');
    message.className = 'desc';
    message.textContent = discoveryError ? 'Recipe discovery failed.' : 'No recipes found.';
    empty.appendChild(message);

    if (discoveryError) {
      const detail = document.createElement('code');
      detail.textContent = discoveryError;
      empty.appendChild(detail);
    }

    const actions = document.createElement('div');
    actions.className = 'empty-actions';

    const refresh = document.createElement('button');
    refresh.textContent = 'Refresh';
    refresh.onclick = () => post('refreshRecipes', {});

    const configure = document.createElement('button');
    configure.className = 'secondary';
    configure.textContent = 'Set Host Entry Point';
    configure.onclick = () => post('configureHost', {});

    actions.appendChild(refresh);
    actions.appendChild(configure);
    empty.appendChild(actions);
    return empty;
  }

  function renderForm() {
    renderParams();
    elements.form.innerHTML = '';
    if (!recipe) {
      elements.form.innerHTML = '<p class="desc">Choose a recipe from the Recipes tab.</p>';
      elements.templates.innerHTML = '';
      return;
    }

    for (const arg of recipe.args) {
      const label = document.createElement('label');
      label.textContent = arg.name + (arg.required ? ' *' : '');
      if (arg.help) {
        const help = document.createElement('span');
        help.className = 'help';
        help.textContent = ' — ' + arg.help;
        label.appendChild(help);
      }

      const row = document.createElement('div');
      row.className = 'row';

      const input = document.createElement('input');
      input.type = 'text';
      input.id = 'arg-' + arg.name;
      input.value = initialArgs[arg.name] || arg.defaultsTo || '';
      input.oninput = () => {
        renderParams();
        renderTemplates();
      };

      addOptions(arg, input, row);
      row.appendChild(input);
      addPicker(arg, row);
      elements.form.appendChild(label);
      elements.form.appendChild(row);
    }
    renderTemplates();
  }

  function addOptions(arg, input, row) {
    if (!arg.options || !arg.options.length) return;

    const listId = 'options-' + arg.name;
    input.setAttribute('list', listId);

    const list = document.createElement('datalist');
    list.id = listId;
    for (const option of arg.options) {
      const el = document.createElement('option');
      el.value = option;
      list.appendChild(el);
    }
    row.appendChild(list);

    if (arg.allowCustomValue === false) {
      input.placeholder = 'Choose one of the suggested values';
    }
  }

  function addPicker(arg, row) {
    const inputKind = effectiveInputKind(arg);
    if (inputKind !== 'file' && inputKind !== 'directory') return;

    const pick = document.createElement('button');
    pick.textContent = inputKind === 'directory' ? 'Browse Folder…' : 'Browse…';
    pick.style.marginTop = '0';
    pick.onclick = () =>
      post(inputKind === 'directory' ? 'pickDirectory' : 'pickFile', { arg: arg.name });
    row.appendChild(pick);
  }

  function effectiveInputKind(arg) {
    if (arg.inputKind && arg.inputKind !== 'text') return arg.inputKind;
    return looksLikePath(arg.name) ? 'file' : 'text';
  }

  function looksLikePath(name) {
    return name === 'file' || name === 'path' || name.endsWith('_file') || name.endsWith('Path');
  }

  function renderParams() {
    elements.params.innerHTML = '';
    if (!recipe) return;

    for (const arg of recipe.args) {
      const row = document.createElement('div');
      row.className = 'param';

      const name = document.createElement('span');
      name.className = 'param-name';
      name.textContent = arg.name + (arg.required ? ' *' : '');

      const meta = document.createElement('span');
      const parts = [arg.inputKind || 'text'];
      if (arg.contextKey) parts.push('from ' + arg.contextKey);
      if (arg.options && arg.options.length) parts.push(arg.options.length + ' suggestions');
      meta.textContent = parts.join(' · ');

      row.appendChild(name);
      row.appendChild(meta);
      elements.params.appendChild(row);
    }
  }

  function collectArgs() {
    const args = {};
    if (!recipe) return args;
    for (const arg of recipe.args) {
      const el = byId('arg-' + arg.name);
      if (el && el.value) args[arg.name] = el.value;
    }
    return args;
  }

  function renderTemplates() {
    elements.templates.innerHTML = '';
    if (!recipe || !recipe.previewTemplates || !recipe.previewTemplates.length) return;

    const args = collectArgs();
    const heading = document.createElement('h3');
    heading.textContent = 'Template Preview';
    elements.templates.appendChild(heading);

    for (const template of recipe.previewTemplates) {
      const card = document.createElement('div');
      card.className = 'template-card';

      const title = document.createElement('div');
      title.className = 'template-title';

      const label = document.createElement('strong');
      label.textContent = template.label;

      const target = document.createElement('span');
      target.className = 'template-path';
      target.innerHTML = renderTemplate(template.path, args);

      const code = document.createElement('div');
      code.className = 'template-code';
      code.innerHTML = renderTemplate(
        template.content || '// Template preview loaded on demand.',
        args
      );

      title.appendChild(label);
      title.appendChild(target);
      card.appendChild(title);
      card.appendChild(code);
      elements.templates.appendChild(card);
    }
  }

  function renderTemplate(source, args) {
    return escapeHtml(source).replace(/\{\{\s*([A-Za-z_]\w*)(?::(snake|camel|pascal))?\s*\}\}/g, (_, name, casing) => {
      const raw = args[name] || '';
      const original = '{{' + name + (casing ? ':' + casing : '') + '}}';
      if (!raw) {
        return '<mark class="placeholder missing" title="' + escapeAttr(original) + '">missing ' + escapeHtml(name) + '</mark>';
      }
      return '<mark class="placeholder" title="' + escapeAttr(original) + '">' + escapeHtml(applyCasing(raw, casing)) + '</mark>';
    });
  }

  function renderReview() {
    elements.files.innerHTML = '';
    activeChangeIndex = 0;
    for (const file of files) {
      elements.files.appendChild(createFileCard(file));
    }
    selectChange(0, false);
    elements.review.classList.remove('hidden');
  }

  function createFileCard(file) {
    const card = document.createElement('details');
    card.className = 'file';
    card.open = true;
    card.dataset.path = file.path;

    const head = document.createElement('summary');
    head.className = 'file-head';

    const title = document.createElement('span');
    title.className = 'file-path';
    title.textContent = file.path;

    const badge = document.createElement('span');
    badge.className = 'badge';
    badge.textContent = file.isNew ? 'new' : file.kind;
    title.appendChild(badge);

    const right = document.createElement('div');
    const fileToggle = document.createElement('input');
    fileToggle.type = 'checkbox';
    fileToggle.checked = true;
    fileToggle.className = 'file-toggle';

    const diffBtn = document.createElement('button');
    diffBtn.textContent = 'Open Diff';
    diffBtn.className = 'secondary';
    diffBtn.style.marginTop = '0';
    diffBtn.onclick = () => post('openDiff', { path: file.path });

    right.appendChild(fileToggle);
    right.appendChild(diffBtn);
    head.appendChild(title);
    head.appendChild(right);
    card.appendChild(head);

    const patches = file.patches.length
      ? file.patches
      : [{ index: -1, description: file.isNew ? 'Create file' : 'Whole-file change', replacement: file.modified }];

    for (const patch of patches) {
      card.appendChild(createPatchRow(file, patch));
    }
    return card;
  }

  function createPatchRow(file, patch) {
    const row = document.createElement('div');
    row.className = 'patch';
    row.dataset.path = file.path;
    row.dataset.index = patch.index;
    row.onclick = (event) => {
      if (event.target.tagName !== 'INPUT') {
        selectChange(findChangeIndex(file.path, patch.index), true);
      }
    };

    const cb = document.createElement('input');
    cb.type = 'checkbox';
    cb.checked = true;
    cb.className = patch.index < 0 ? 'whole-file-toggle' : 'patch-toggle';
    cb.dataset.index = patch.index;

    const code = document.createElement('code');
    const replacementText = patch.replacement || patch.replacementPreview || '';
    code.textContent = (patch.description ? patch.description + '\n' : '') + replacementText.trim();

    row.appendChild(cb);
    row.appendChild(code);
    return row;
  }

  function allChangeRows() {
    return [...elements.files.querySelectorAll('.patch')];
  }

  function findChangeIndex(path, patchIndex) {
    return allChangeRows().findIndex((row) => row.dataset.path === path && Number(row.dataset.index) === Number(patchIndex));
  }

  function selectChange(index, openDiff) {
    const rows = allChangeRows();
    if (!rows.length) return;

    activeChangeIndex = Math.max(0, Math.min(index, rows.length - 1));
    rows.forEach((row, i) => row.classList.toggle('active', i === activeChangeIndex));
    rows[activeChangeIndex].scrollIntoView({ block: 'nearest' });
    if (openDiff) {
      post('openDiff', { path: rows[activeChangeIndex].dataset.path });
    }
  }

  function buildSelection() {
    const selection = { files: {} };
    for (const card of elements.files.querySelectorAll('.file')) {
      const p = card.dataset.path;
      const include = card.querySelector('.file-toggle').checked;
      const patches = [...card.querySelectorAll('.patch-toggle')]
        .filter((c) => c.checked)
        .map((c) => Number(c.dataset.index));
      const entry = { include };
      if (card.querySelectorAll('.patch-toggle').length > 0) {
        entry.patches = patches;
      }
      selection.files[p] = entry;
    }
    return selection;
  }

  function showError(msg) {
    elements.error.textContent = msg;
    elements.error.classList.remove('hidden');
  }

  function clearError() {
    elements.error.classList.add('hidden');
    elements.error.textContent = '';
  }

  function setPreviewInFlight(value) {
    previewInFlight = value;
    elements.previewButton.disabled = value;
    elements.previewButton.textContent = value ? 'Previewing…' : 'Preview Changes';
  }

  function applyCasing(value, casing) {
    if (casing === 'snake') {
      return value.replace(/([a-z0-9])([A-Z])/g, '$1_$2').replace(/[\s-]+/g, '_').toLowerCase();
    }
    if (casing === 'pascal') {
      return value.split(/[\s_-]+/).filter(Boolean).map((p) => p.charAt(0).toUpperCase() + p.slice(1)).join('');
    }
    if (casing === 'camel') {
      const pascal = applyCasing(value, 'pascal');
      return pascal.charAt(0).toLowerCase() + pascal.slice(1);
    }
    return value;
  }

  function escapeHtml(value) {
    return String(value)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
  }

  function escapeAttr(value) {
    return escapeHtml(value).replace(/"/g, '&quot;');
  }

  elements.previewButton.onclick = () => {
    clearError();
    if (previewInFlight) {
      return;
    }
    if (!recipe) {
      showError('Select a recipe first.');
      return;
    }
    setPreviewInFlight(true);
    post('preview', { args: collectArgs() });
  };
  elements.applyButton.onclick = () => {
    clearError();
    post('apply', { selection: buildSelection() });
  };
  elements.backButton.onclick = () => elements.review.classList.add('hidden');
  elements.previousButton.onclick = () => selectChange(activeChangeIndex - 1, true);
  elements.nextButton.onclick = () => selectChange(activeChangeIndex + 1, true);
  elements.openActiveButton.onclick = () => {
    const rows = allChangeRows();
    if (rows.length) post('openDiff', { path: rows[activeChangeIndex].dataset.path });
  };
  elements.recipesTab.onclick = () => {
    activeTab = 'recipes';
    renderTabs();
    post('showRecipes', {});
  };
  elements.runnerTab.onclick = () => {
    activeTab = 'runner';
    renderTabs();
    post('showRunner', {});
  };

  window.addEventListener('message', (event) => {
    const msg = event.data;
    if (msg.type === 'filePicked') {
      const el = byId('arg-' + msg.arg);
      if (el) {
        el.value = msg.value;
        renderParams();
        renderTemplates();
      }
    } else if (msg.type === 'previewResult') {
      files = msg.files;
      setPreviewInFlight(false);
      if (!files.length) {
        showError('No changes produced by this recipe.');
        elements.review.classList.add('hidden');
      } else {
        renderReview();
      }
    } else if (msg.type === 'applyResult') {
      elements.review.classList.add('hidden');
    } else if (msg.type === 'error') {
      setPreviewInFlight(false);
      showError(msg.message);
    } else if (msg.type === 'previewState') {
      setPreviewInFlight(Boolean(msg.inFlight));
    }
  });

  renderRecipeList();
  renderForm();
  renderTabs();
  setPreviewInFlight(false);
})();
