const shiftFieldLabels = {
  title: 'Role', company: 'Company', area: 'Location', rate: 'Gross zł/hour',
  date: 'Date', start: 'Start time', end: 'End time', requirements: 'Requirements', category: 'Category'
};
const shiftCategories = ['Hospitality', 'Events', 'Retail'];

function shiftField(key, value) {
  if (key === 'requirements') {
    return Array.isArray(value) ? value.filter(v => typeof v === 'string').map(v => v.trim().slice(0, 80)).filter(Boolean).slice(0, 12) : [];
  }
  if (key === 'rate') return typeof value === 'number' && value >= 1 && value <= 1000 ? value : '';
  if (typeof value !== 'string') return '';
  value = value.trim();
  if (key === 'category') return shiftCategories.includes(value) ? value : '';
  if (key === 'start' || key === 'end') return /^([01][0-9]|2[0-3]):[0-5][0-9]$/.test(value) ? value : '';
  if (key === 'date') {
    if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return '';
    const date = new Date(value + 'T00:00:00Z');
    return Number.isFinite(date.valueOf()) && date.toISOString().slice(0, 10) === value ? value : '';
  }
  return value.slice(0, 80);
}

function makeShiftDraft(brief, extracted = {}) {
  const draft = { brief, commute: 15 };
  for (const key of Object.keys(shiftFieldLabels)) {
    const field = extracted[key];
    const evidence = typeof field?.evidence === 'string' ? field.evidence.trim() : '';
    draft[key] = shiftField(key, evidence && brief.includes(evidence) ? field.value : undefined);
  }
  return draft;
}

function studio() {
  return heading('', 'Draft a shift', 'Enter a staffing brief, then review the shift before you publish.') +
    `<div class="studio">
      <form class="panel workflow-form" onsubmit="generateDraft(event)">
        <h2>Staffing brief</h2>
        <label class="input-label" for="brief">Who do you need, and when?</label>
        <textarea id="brief" name="brief" required maxlength="1500" placeholder="Example: Need a barista in Mokotów on Friday, 08:00–16:00, 35 zł/hour.">${esc(state.draft?.brief || '')}</textarea>
        <div class="workflow-actions">
          ${Object.keys(examples).map(k => `<button class="button light small" type="button" onclick="document.getElementById('brief').value=examples['${k}']">${{cafe:'Café example',event:'Event example',retail:'Retail example'}[k]}</button>`).join('')}
        </div>
        <p class="panel-note">The brief goes to the LARPd server and DeepSeek. Use demo data only.</p>
        <div class="workflow-actions">
          <button class="button" id="generateButton">Extract shift details</button>
          <button class="button light" type="button" onclick="manualDraft()">Enter details manually</button>
        </div>
        <p id="draftStatus" class="panel-note" role="status"></p>
      </form>
      <section aria-label="Shift draft">${draftForm()}</section>
    </div>`;
}

function draftForm() {
  const draft = state.draft;
  if (!draft) return '<div class="panel workflow-empty"><h2>Shift preview</h2><p class="panel-note">Role, location, hours, and gross pay will appear here for review.</p></div>';
  const input = (name, type = 'text') => {
    const value = name === 'requirements' ? draft.requirements.join(', ') : draft[name] ?? '';
    const limits = name === 'rate' ? ' min="1" max="1000" step=".01"' : type === 'text' ? ` maxlength="${name === 'requirements' ? 600 : 80}"` : '';
    return `<div><label class="input-label" for="df-${name}">${shiftFieldLabels[name]}</label>
      <input id="df-${name}" name="${name}" type="${type}" value="${esc(value)}"${limits} required></div>`;
  };
  const postings = previousPostings(draft);
  return `<form id="shiftDraftForm" class="panel workflow-form" onsubmit="publishDraft(event)" oninput="saveShiftField(event.target)">
    <h2>Review the shift</h2>
    <p class="panel-note">Fill in any blank fields before publishing.</p>
    ${input('title')}${input('company')}${input('area')}
    <div class="field-grid">${input('rate', 'number')}${input('date', 'date')}${input('start', 'time')}${input('end', 'time')}</div>
    ${input('requirements')}
    <label class="input-label" for="df-category">Category</label>
    <select id="df-category" name="category" required><option value="">Choose a category</option>
      ${shiftCategories.map(c => `<option ${draft.category === c ? 'selected' : ''}>${c}</option>`).join('')}
    </select>
    ${postings.length ? `<details class="workflow-history"><summary>Use details from a past posting</summary>
      <label class="input-label" for="previousPosting">Past posting</label>
      <select id="previousPosting" onchange="previewPosting(this)">
        <option value="">Choose a posting</option>
        ${postings.map(j => `<option value="${j.id}">${esc(j.title)} · ${esc(j.company)} · ${esc(j.date)}</option>`).join('')}
      </select>
      <div id="postingPreview" aria-live="polite"></div>
    </details>` : ''}
    <button class="button">Publish reviewed shift</button>
  </form>`;
}

function previousPostings(draft) {
  const seeds = new Set([...initialJobs, ...larpJobs, ...everydayJobs].map(j => j.id));
  const own = state.jobs.filter(j => j.postedByDemoEmployer || !seeds.has(j.id));
  return own.sort((a, b) => Number(b.company === draft.company) - Number(a.company === draft.company)).slice(0, 15);
}

function postingSuggestions(select) {
  const posting = previousPostings(state.draft).find(j => String(j.id) === select.value);
  if (!posting) return [];
  const form = select.form;
  return Object.keys(shiftFieldLabels).filter(key => key !== 'date' && !form.elements[key].value.trim()).flatMap(key => {
    const value = shiftField(key, posting[key]);
    return value && value.length !== 0 ? [[key, Array.isArray(value) ? value.join(', ') : String(value)]] : [];
  });
}

function previewPosting(select) {
  const preview = document.getElementById('postingPreview');
  const suggestions = postingSuggestions(select);
  preview.innerHTML = !select.value ? '' : suggestions.length ?
    `<p class="panel-note">Review these values from the selected posting.</p>
      <dl>${suggestions.map(([key, value]) => `<div><dt>${shiftFieldLabels[key]}</dt><dd>${esc(value)}</dd></div>`).join('')}</dl>
      <button class="button light small" type="button">Confirm and fill blanks</button>` :
    '<p class="panel-note">This posting has no values for the remaining blanks.</p>';
  preview.querySelector('button')?.addEventListener('click', () => {
    for (const [key, value] of suggestions) {
      const input = select.form.elements[key];
      if (!input.value.trim()) {
        input.value = value;
        saveShiftField(input);
      }
    }
    preview.textContent = 'Past posting details added. Review the shift before publishing.';
  });
}

function saveShiftField(input) {
  const key = input.name;
  if (!state.draft || !Object.hasOwn(shiftFieldLabels, key)) return;
  const value = key === 'rate' ? Number(input.value) : key === 'requirements' ? input.value.split(',') : input.value;
  state.draft[key] = shiftField(key, value);
  persist();
}

function manualDraft() {
  const brief = document.getElementById('brief').value.trim();
  if (state.draft) return document.getElementById('df-title').focus();
  state.draft = makeShiftDraft(brief);
  render();
  document.getElementById('df-title')?.focus();
}

async function extractShift(brief, signal) {
  const data = await window.larpdAssistant.draft('shift', brief, signal);
  const draft = makeShiftDraft(brief, data);
  if (!Object.keys(shiftFieldLabels).some(key => draft[key] && draft[key].length !== 0)) {
    throw new Error('No shift details were found. Add more to the brief or enter the shift manually.');
  }
  return draft;
}

async function generateDraft(event) {
  event.preventDefault();
  const form = event.currentTarget;
  const brief = form.elements.brief.value.trim();
  const status = document.getElementById('draftStatus');
  if (!brief) { status.textContent = 'Enter a staffing brief first.'; return; }
  if (!window.larpdAssistant) { status.textContent = 'The assistant is unavailable. You can enter the shift details manually.'; return; }
  const button = form.querySelector('#generateButton');
  button.disabled = true;
  status.textContent = 'Reading your staffing brief…';
  try {
    const draft = await extractShift(brief);
    if (!form.isConnected || form.elements.brief.value.trim() !== brief) return;
    state.draft = draft;
    render();
    document.getElementById('df-title')?.focus();
  } catch (error) {
    status.textContent = error.message;
  } finally {
    if (form.isConnected && status.textContent === 'Reading your staffing brief…') status.textContent = 'The brief changed. Extract shift details again to use the new text.';
    button.disabled = false;
  }
}

function publishDraft(event) {
  event.preventDefault();
  const fields = Object.fromEntries(new FormData(event.currentTarget));
  const draft = { brief: state.draft?.brief || '' };
  for (const key of Object.keys(shiftFieldLabels)) {
    const value = key === 'rate' ? Number(fields.rate) : key === 'requirements' ? fields.requirements.split(',') : fields[key];
    draft[key] = shiftField(key, value);
  }
  if (Object.keys(shiftFieldLabels).some(key => !draft[key] || draft[key].length === 0)) return toast('Fill in all shift details before publishing.');
  if (minutes(draft.end) <= minutes(draft.start)) return toast('Choose an end time later on the same day.');
  state.jobs.unshift({
    ...draft,
    id: Math.max(...state.jobs.map(j => j.id)) + 1,
    scene: draft.category === 'Events' ? 'event' : draft.category === 'Retail' ? 'retail' : 'cafe',
    description: draft.brief || draft.title,
    commute: 15, urgent: true, postedByDemoEmployer: true
  });
  state.draft = null;
  go('overview');
  toast('Shift published to the demo.');
}

function openCvBuilder() {
  openModal(`<div class="cv-editor">
    <h2>Generate a CV</h2>
    <p class="panel-note">Describe your skills and things you’ve done. Use sample details. Your notes go to the LARPd server and DeepSeek.</p>
    <form class="workflow-form" onsubmit="generateCv(event)">
      <label class="input-label" for="cv-notes">Skills and experience</label>
      <textarea id="cv-notes" name="notes" required maxlength="1500" placeholder="Example: I make espresso drinks, helped train one new starter, and keep the counter organized."></textarea>
      <button class="button">Draft my CV</button>
      <p class="panel-note" role="status" id="cv-status"></p>
    </form>
    <div id="cv-draft"></div>
  </div><section id="cv-print-sheet" hidden></section>`);
}

function cvLines(values) {
  return Array.isArray(values) ? values.filter(v => typeof v === 'string').map(v => v.trim().slice(0, 220)).filter(Boolean).slice(0, 10).join('\n') : '';
}

async function generateCv(event) {
  event.preventDefault();
  const form = event.currentTarget;
  const notes = form.elements.notes.value.trim();
  const status = document.getElementById('cv-status');
  if (!notes) { status.textContent = 'Enter your skills or experience first.'; return; }
  if (!window.larpdAssistant) { status.textContent = 'The assistant is unavailable. Try again when the API is connected.'; return; }
  const button = form.querySelector('button');
  button.disabled = true;
  status.textContent = 'Drafting from your notes…';
  try {
    const cv = await window.larpdAssistant.draft('cv', notes);
    if (!form.isConnected || !modal.open || form.elements.notes.value.trim() !== notes) return;
    const summary = typeof cv.summary === 'string' ? cv.summary.trim().slice(0, 850) : '';
    const skills = cvLines(cv.skills);
    const experience = cvLines(cv.experience);
    const extras = [
      ['education', 'Education and training', cvLines(cv.education)],
      ['projects', 'Projects and volunteering', cvLines(cv.projects)],
      ['languages', 'Languages', cvLines(cv.languages)]
    ];
    if (!summary && !skills && !experience && !extras.some(([, , value]) => value)) { status.textContent = 'Add a few specific skills or things you’ve done.'; return; }
    document.getElementById('cv-draft').innerHTML = `<form class="workflow-form cv-review" onsubmit="printCv(event)">
      <h3>Review your draft</h3>
      <p class="panel-note">Check that each claim is accurate. The name you enter here stays in your browser.</p>
      <label class="input-label" for="cv-name">Name on CV (optional)</label><input id="cv-name" name="displayName" maxlength="80">
      <label class="input-label" for="cv-template">Template</label>
      <select id="cv-template" name="template"><option value="classic">Classic</option><option value="skills">Skills first</option></select>
      <label class="input-label" for="cv-summary">Summary</label><textarea id="cv-summary" name="summary" maxlength="850">${esc(summary)}</textarea>
      <label class="input-label" for="cv-skills">Skills, one per line</label><textarea id="cv-skills" name="skills" maxlength="2200">${esc(skills)}</textarea>
      <label class="input-label" for="cv-experience">Experience, one bullet per line</label><textarea id="cv-experience" name="experience" maxlength="2200">${esc(experience)}</textarea>
      <details class="workflow-history" ${extras.some(([, , value]) => value) ? 'open' : ''}>
        <summary>Education, projects and languages</summary>
        ${extras.map(([name, label, value]) => `<label class="input-label" for="cv-${name}">${label}, one item per line</label><textarea id="cv-${name}" name="${name}" maxlength="2200">${esc(value)}</textarea>`).join('')}
      </details>
      <button class="button">Print or save PDF</button>
    </form>`;
    status.textContent = 'Draft ready to edit.';
  } catch (error) {
    status.textContent = error.message;
  } finally {
    if (form.isConnected && status.textContent === 'Drafting from your notes…') status.textContent = 'Your notes changed. Submit again to use the new text.';
    button.disabled = false;
  }
}

function printCv(event) {
  event.preventDefault();
  const data = Object.fromEntries(new FormData(event.currentTarget));
  const sheet = document.getElementById('cv-print-sheet');
  const list = (title, body) => {
    const items = body.split('\n').map(s => s.trim()).filter(Boolean);
    return items.length ? `<section><h2>${title}</h2><ul>${items.map(item => `<li>${esc(item)}</li>`).join('')}</ul></section>` : '';
  };
  const summary = data.summary.trim() ? `<section><h2>Summary</h2><p>${esc(data.summary.trim())}</p></section>` : '';
  const skills = list('Skills', data.skills);
  const experience = list('Experience', data.experience);
  const projects = list('Projects and volunteering', data.projects);
  sheet.innerHTML = `<h1>${esc(data.displayName.trim() || 'Curriculum vitae')}</h1>` +
    summary + (data.template === 'skills' ? skills + projects + experience : experience + skills + projects) +
    list('Education and training', data.education) + list('Languages', data.languages);
  document.body.classList.add('cv-print');
  try { window.print(); } finally { document.body.classList.remove('cv-print'); }
}
