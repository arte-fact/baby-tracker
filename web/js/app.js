import init, { BabyTracker } from '../pkg/baby_tracker.js';

const STORAGE_KEY = 'baby-tracker-data';
const NAME_KEY = 'baby-tracker-last-name';

let tracker = null;

// --- Persistence ---

function save() {
  localStorage.setItem(STORAGE_KEY, tracker.exportData());
}

function load() {
  const data = localStorage.getItem(STORAGE_KEY);
  if (data) {
    try {
      tracker = BabyTracker.loadData(data);
    } catch {
      tracker = new BabyTracker();
    }
  } else {
    tracker = new BabyTracker();
  }
}

// --- Icons ---

const TYPE_ICONS = {
  'breast-left': '\u{1F931}',
  'breast-right': '\u{1F931}',
  'bottle': '\u{1F37C}',
  'solid': '\u{1F963}',
};

const TYPE_LABELS = {
  'breast-left': 'Breast (Left)',
  'breast-right': 'Breast (Right)',
  'bottle': 'Bottle',
  'solid': 'Solid',
};

// --- Navigation ---

function switchView(name) {
  document.querySelectorAll('.view').forEach(v => v.classList.remove('active'));
  document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));
  document.getElementById(`view-${name}`).classList.add('active');
  document.querySelector(`.nav-btn[data-view="${name}"]`).classList.add('active');

  if (name === 'history') renderHistory();
  if (name === 'summary') renderSummary();
}

// --- Add Feeding ---

let selectedType = null;

function setupForm() {
  const form = document.getElementById('feeding-form');
  const nameInput = document.getElementById('baby-name');
  const tsInput = document.getElementById('timestamp');

  // Restore last used name
  const lastName = localStorage.getItem(NAME_KEY);
  if (lastName) nameInput.value = lastName;

  // Default timestamp to now
  tsInput.value = new Date().toISOString().slice(0, 16);

  // Type selection
  document.querySelectorAll('.type-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.type-btn').forEach(b => b.classList.remove('selected'));
      btn.classList.add('selected');
      selectedType = btn.dataset.type;
    });
  });

  form.addEventListener('submit', (e) => {
    e.preventDefault();
    if (!selectedType) {
      alert('Please select a feeding type');
      return;
    }

    const name = nameInput.value.trim();
    const amount = document.getElementById('amount').value || undefined;
    const duration = document.getElementById('duration').value || undefined;
    const notes = document.getElementById('notes').value || undefined;
    const ts = tsInput.value;

    // Convert datetime-local to ISO format
    const timestamp = ts.replace('T', 'T') + ':00';

    try {
      tracker.addFeeding(
        name,
        selectedType,
        amount ? parseFloat(amount) : undefined,
        duration ? parseInt(duration) : undefined,
        notes || undefined,
        timestamp,
      );
      save();
      localStorage.setItem(NAME_KEY, name);

      // Reset form (keep name and type)
      document.getElementById('amount').value = '';
      document.getElementById('duration').value = '';
      document.getElementById('notes').value = '';
      tsInput.value = new Date().toISOString().slice(0, 16);

      // Brief visual feedback
      const btn = document.getElementById('btn-save');
      btn.textContent = 'Saved!';
      btn.style.background = '#00b894';
      setTimeout(() => {
        btn.textContent = 'Save Feeding';
        btn.style.background = '';
      }, 1000);
    } catch (err) {
      alert(err.message || err);
    }
  });
}

// --- History ---

function renderHistory() {
  const container = document.getElementById('history-list');
  const json = tracker.listFeedings(undefined, 50);
  const feedings = JSON.parse(json);

  if (feedings.length === 0) {
    container.innerHTML = '<p class="empty-state">No feedings recorded yet.</p>';
    return;
  }

  container.innerHTML = feedings.map(f => {
    const icon = TYPE_ICONS[f.feeding_type] || '';
    const typeLabel = TYPE_LABELS[f.feeding_type] || f.feeding_type;
    const details = [];
    if (f.amount_ml != null) details.push(`${f.amount_ml} ml`);
    if (f.duration_minutes != null) details.push(`${f.duration_minutes} min`);
    if (f.notes) details.push(f.notes);
    const ts = new Date(f.timestamp).toLocaleString([], {
      month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
    });

    return `
      <div class="feeding-card" data-id="${f.id}">
        <div class="icon">${icon}</div>
        <div class="info">
          <div class="name-type">${f.baby_name} &mdash; ${typeLabel}</div>
          <div class="details">${details.join(' &bull; ')}</div>
        </div>
        <div class="time">${ts}</div>
        <button class="delete-btn" title="Delete">&times;</button>
      </div>
    `;
  }).join('');

  // Delete handlers
  container.querySelectorAll('.delete-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      const card = btn.closest('.feeding-card');
      const id = parseInt(card.dataset.id);
      if (confirm('Delete this feeding?')) {
        tracker.deleteFeeding(id);
        save();
        renderHistory();
      }
    });
  });
}

// --- Summary ---

let summaryDays = 1;

function renderSummary() {
  const container = document.getElementById('summary-content');
  const since = new Date();
  since.setDate(since.getDate() - summaryDays);
  const sinceStr = since.toISOString().slice(0, 19);

  let summary;
  try {
    summary = JSON.parse(tracker.getSummary(undefined, sinceStr));
  } catch {
    container.innerHTML = '<p class="empty-state">No data for this period.</p>';
    return;
  }

  if (summary.total_feedings === 0) {
    container.innerHTML = '<p class="empty-state">No feedings in this period.</p>';
    return;
  }

  const maxCount = Math.max(...summary.by_type.map(([, c]) => c), 1);

  container.innerHTML = `
    <div class="stat-row">
      <div class="stat-card">
        <div class="stat-label">Total Feedings</div>
        <div class="stat-value">${summary.total_feedings}</div>
      </div>
      <div class="stat-card">
        <div class="stat-label">Total Volume</div>
        <div class="stat-value">${summary.total_ml > 0 ? Math.round(summary.total_ml) + ' ml' : '—'}</div>
      </div>
    </div>
    <div class="stat-card">
      <div class="stat-label">Total Nursing Time</div>
      <div class="stat-value">${summary.total_minutes > 0 ? summary.total_minutes + ' min' : '—'}</div>
    </div>
    <div class="stat-card">
      <div class="stat-label">By Type</div>
      <div class="type-breakdown">
        ${summary.by_type.map(([type, count]) => `
          <div class="type-bar">
            <span class="label">${TYPE_LABELS[type] || type}</span>
            <div class="bar"><div class="fill" style="width: ${(count / maxCount) * 100}%"></div></div>
            <span class="count">${count}</span>
          </div>
        `).join('')}
      </div>
    </div>
  `;
}

function setupSummary() {
  document.querySelectorAll('.period-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.period-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      summaryDays = parseInt(btn.dataset.days);
      renderSummary();
    });
  });
}

// --- Init ---

async function main() {
  await init();
  load();
  setupForm();
  setupSummary();

  document.querySelectorAll('.nav-btn').forEach(btn => {
    btn.addEventListener('click', () => switchView(btn.dataset.view));
  });

  // Register service worker
  if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('sw.js').catch(() => {});
  }
}

main();
