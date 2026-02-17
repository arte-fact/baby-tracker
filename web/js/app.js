import init, { BabyTracker } from '../pkg/baby_tracker.js';

const STORAGE_KEY = 'baby-tracker-data';
const NAME_KEY = 'baby-tracker-name';
const ACTIVE_KEY = 'baby-tracker-active';

let tracker = null;

// --- State ---

let currentDate = new Date();
resetToStartOfDay(currentDate);

let activeFeeding = null; // { type, startedAt (ms timestamp) }
let timerInterval = null;

// --- Persistence ---

function save() {
  localStorage.setItem(STORAGE_KEY, tracker.exportData());
}

function load() {
  const data = localStorage.getItem(STORAGE_KEY);
  if (data) {
    try { tracker = BabyTracker.loadData(data); } catch { tracker = new BabyTracker(); }
  } else {
    tracker = new BabyTracker();
  }
  // Restore active feeding from localStorage (survives page reload)
  const active = localStorage.getItem(ACTIVE_KEY);
  if (active) {
    try { activeFeeding = JSON.parse(active); } catch { activeFeeding = null; }
  }
}

function getBabyName() {
  return localStorage.getItem(NAME_KEY) || '';
}

// --- Helpers ---

function resetToStartOfDay(d) {
  d.setHours(0, 0, 0, 0);
  return d;
}

function dateStr(d) {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

function isToday(d) {
  const now = new Date();
  return d.getFullYear() === now.getFullYear() &&
    d.getMonth() === now.getMonth() &&
    d.getDate() === now.getDate();
}

function isYesterday(d) {
  const y = new Date();
  y.setDate(y.getDate() - 1);
  return d.getFullYear() === y.getFullYear() &&
    d.getMonth() === y.getMonth() &&
    d.getDate() === y.getDate();
}

function formatTime(isoStr) {
  const d = new Date(isoStr);
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

function formatElapsed(ms) {
  const totalSec = Math.floor(ms / 1000);
  const m = Math.floor(totalSec / 60);
  const s = totalSec % 60;
  return `${m}:${String(s).padStart(2, '0')}`;
}

function toISOTimestamp(date) {
  const y = date.getFullYear();
  const mo = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  const h = String(date.getHours()).padStart(2, '0');
  const mi = String(date.getMinutes()).padStart(2, '0');
  const s = String(date.getSeconds()).padStart(2, '0');
  return `${y}-${mo}-${d}T${h}:${mi}:${s}`;
}

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

// --- Day Navigation ---

const $dayTitle = document.getElementById('day-title');
const $dayDate = document.getElementById('day-date');
const $btnPrev = document.getElementById('btn-prev');
const $btnNext = document.getElementById('btn-next');
const $timeline = document.getElementById('timeline');
const $daySummary = document.getElementById('day-summary');

function updateDayHeader() {
  if (isToday(currentDate)) {
    $dayTitle.textContent = 'Today';
  } else if (isYesterday(currentDate)) {
    $dayTitle.textContent = 'Yesterday';
  } else {
    $dayTitle.textContent = currentDate.toLocaleDateString([], { weekday: 'long' });
  }
  $dayDate.textContent = currentDate.toLocaleDateString([], {
    month: 'short', day: 'numeric', year: 'numeric',
  });

  // Hide "next" if we're on today
  $btnNext.style.visibility = isToday(currentDate) ? 'hidden' : 'visible';
}

function goDay(offset) {
  currentDate.setDate(currentDate.getDate() + offset);
  resetToStartOfDay(currentDate);
  render();
}

// --- Swipe ---

let touchStartX = 0;
let touchStartY = 0;

function setupSwipe() {
  const el = $timeline;

  el.addEventListener('touchstart', (e) => {
    touchStartX = e.touches[0].clientX;
    touchStartY = e.touches[0].clientY;
  }, { passive: true });

  el.addEventListener('touchend', (e) => {
    const dx = e.changedTouches[0].clientX - touchStartX;
    const dy = e.changedTouches[0].clientY - touchStartY;

    // Only trigger swipe if horizontal movement dominates
    if (Math.abs(dx) > 60 && Math.abs(dx) > Math.abs(dy) * 1.5) {
      if (dx > 0) {
        goDay(-1); // swipe right = previous day
      } else if (!isToday(currentDate)) {
        goDay(1);  // swipe left = next day
      }
    }
  }, { passive: true });
}

// --- Timeline Rendering ---

function render() {
  updateDayHeader();
  renderTimeline();
  renderDaySummary();
}

function renderTimeline() {
  const ds = dateStr(currentDate);
  let feedings;
  try {
    feedings = JSON.parse(tracker.listFeedingsForDay(undefined, ds));
  } catch {
    feedings = [];
  }

  if (feedings.length === 0) {
    $timeline.innerHTML = '<div class="empty-state">No feedings this day</div>';
    return;
  }

  $timeline.innerHTML = feedings.map(f => {
    const icon = TYPE_ICONS[f.feeding_type] || '';
    const label = TYPE_LABELS[f.feeding_type] || f.feeding_type;
    const meta = [];
    if (f.amount_ml != null) meta.push(`${f.amount_ml} ml`);
    if (f.duration_minutes != null) meta.push(`${f.duration_minutes} min`);
    if (f.notes) meta.push(f.notes);

    return `
      <div class="tl-entry" data-id="${f.id}">
        <div class="tl-dot">${icon}</div>
        <div class="tl-body">
          <div class="tl-info">
            <div class="tl-type">${label}</div>
            ${meta.length ? `<div class="tl-meta">${meta.join(' \u00b7 ')}</div>` : ''}
          </div>
          <div class="tl-time">${formatTime(f.timestamp)}</div>
          <button class="tl-delete" title="Delete">\u00d7</button>
        </div>
      </div>
    `;
  }).join('');

  // Delete handlers
  $timeline.querySelectorAll('.tl-delete').forEach(btn => {
    btn.addEventListener('click', () => {
      const entry = btn.closest('.tl-entry');
      const id = parseInt(entry.dataset.id);
      tracker.deleteFeeding(id);
      save();
      render();
    });
  });
}

function renderDaySummary() {
  const ds = dateStr(currentDate);
  let feedings;
  try {
    feedings = JSON.parse(tracker.listFeedingsForDay(undefined, ds));
  } catch {
    feedings = [];
  }

  const count = feedings.length;
  const totalMl = feedings.reduce((s, f) => s + (f.amount_ml || 0), 0);
  const totalMin = feedings.reduce((s, f) => s + (f.duration_minutes || 0), 0);

  if (count === 0) {
    $daySummary.innerHTML = '';
    return;
  }

  $daySummary.innerHTML = `
    <div class="day-stat"><span class="val">${count}</span><br>feedings</div>
    <div class="day-stat"><span class="val">${totalMl > 0 ? Math.round(totalMl) + ' ml' : '\u2014'}</span><br>volume</div>
    <div class="day-stat"><span class="val">${totalMin > 0 ? totalMin + ' min' : '\u2014'}</span><br>nursing</div>
  `;
}

// --- FAB / Timer ---

const $fab = document.getElementById('fab');
const $fabIconStart = document.getElementById('fab-icon-start');
const $fabTimer = document.getElementById('fab-timer');
const $fabTimerText = document.getElementById('fab-timer-text');

function setupFAB() {
  $fab.addEventListener('click', () => {
    if (activeFeeding) {
      stopFeeding();
    } else {
      showTypePicker();
    }
  });
  // Restore timer UI if there's an active feeding
  if (activeFeeding) {
    enterRecordingMode();
  }
}

function startFeeding(type) {
  activeFeeding = { type, startedAt: Date.now() };
  localStorage.setItem(ACTIVE_KEY, JSON.stringify(activeFeeding));
  enterRecordingMode();
}

function enterRecordingMode() {
  $fab.classList.add('recording');
  $fabIconStart.classList.add('hidden');
  $fabTimer.classList.remove('hidden');
  updateTimerDisplay();
  timerInterval = setInterval(updateTimerDisplay, 1000);
}

function updateTimerDisplay() {
  if (!activeFeeding) return;
  const elapsed = Date.now() - activeFeeding.startedAt;
  $fabTimerText.textContent = formatElapsed(elapsed);
}

function stopFeeding() {
  if (!activeFeeding) return;

  const name = getBabyName();
  const startDate = new Date(activeFeeding.startedAt);
  const elapsed = Date.now() - activeFeeding.startedAt;
  const durationMin = Math.max(1, Math.round(elapsed / 60000));
  const timestamp = toISOTimestamp(startDate);

  try {
    tracker.addFeeding(name, activeFeeding.type, undefined, durationMin, undefined, timestamp);
    save();
  } catch (err) {
    console.error('Failed to save feeding:', err);
  }

  // Clean up
  clearInterval(timerInterval);
  timerInterval = null;
  activeFeeding = null;
  localStorage.removeItem(ACTIVE_KEY);

  $fab.classList.remove('recording');
  $fabTimer.classList.add('hidden');
  $fabIconStart.classList.remove('hidden');

  // Navigate to the day where the feeding was recorded and re-render
  currentDate = new Date(startDate);
  resetToStartOfDay(currentDate);
  render();
}

// --- Type Picker ---

const $typePicker = document.getElementById('type-picker');

function showTypePicker() {
  $typePicker.classList.remove('hidden');
}

function hideTypePicker() {
  $typePicker.classList.add('hidden');
}

function setupTypePicker() {
  // Backdrop dismiss
  $typePicker.querySelector('.type-picker-backdrop').addEventListener('click', hideTypePicker);

  // Type buttons
  $typePicker.querySelectorAll('.type-pick-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      const type = btn.dataset.type;
      hideTypePicker();
      startFeeding(type);
    });
  });
}

// --- Name Prompt ---

const $namePrompt = document.getElementById('name-prompt');
const $nameInput = document.getElementById('name-input');
const $nameSave = document.getElementById('name-save');

function setupNamePrompt() {
  // Show if no name is saved yet
  if (!getBabyName()) {
    $namePrompt.classList.remove('hidden');
    $nameInput.focus();
  }

  $nameSave.addEventListener('click', saveName);
  $nameInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') saveName();
  });

  // Long-press on day title to change name
  let pressTimer = null;
  document.getElementById('day-label').addEventListener('pointerdown', () => {
    pressTimer = setTimeout(() => {
      $nameInput.value = getBabyName();
      $namePrompt.classList.remove('hidden');
      $nameInput.focus();
    }, 600);
  });
  document.getElementById('day-label').addEventListener('pointerup', () => clearTimeout(pressTimer));
  document.getElementById('day-label').addEventListener('pointerleave', () => clearTimeout(pressTimer));
}

function saveName() {
  const name = $nameInput.value.trim();
  if (!name) return;
  localStorage.setItem(NAME_KEY, name);
  $namePrompt.classList.add('hidden');
}

// --- Init ---

async function main() {
  await init();
  load();

  setupSwipe();
  setupFAB();
  setupTypePicker();
  setupNamePrompt();

  $btnPrev.addEventListener('click', () => goDay(-1));
  $btnNext.addEventListener('click', () => goDay(1));

  render();

  if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('sw.js').catch(() => {});
  }
}

main();
