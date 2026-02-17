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
    y.getMonth() === d.getMonth() &&
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

function toDatetimeLocal(isoStr) {
  const d = new Date(isoStr);
  const y = d.getFullYear();
  const mo = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  const h = String(d.getHours()).padStart(2, '0');
  const mi = String(d.getMinutes()).padStart(2, '0');
  return `${y}-${mo}-${day}T${h}:${mi}`;
}

const ICONS = {
  'breast-left': '\u{1F931}',
  'breast-right': '\u{1F931}',
  'bottle': '\u{1F37C}',
  'solid': '\u{1F963}',
  'urine': '\u{1F4A7}',
  'poop': '\u{1F4A9}',
};

const LABELS = {
  'breast-left': 'Breast (Left)',
  'breast-right': 'Breast (Right)',
  'bottle': 'Bottle',
  'solid': 'Solid',
  'urine': 'Pee',
  'poop': 'Poop',
};

const FEEDING_SUBTYPES = ['breast-left', 'breast-right', 'bottle', 'solid'];
const DEJECTION_SUBTYPES = ['urine', 'poop'];

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

function getTimeline() {
  const ds = dateStr(currentDate);
  try {
    return JSON.parse(tracker.timelineForDay(undefined, ds));
  } catch {
    return [];
  }
}

function renderTimeline() {
  const entries = getTimeline();

  if (entries.length === 0) {
    $timeline.innerHTML = '<div class="empty-state">No entries today</div>';
    return;
  }

  $timeline.innerHTML = entries.map(e => {
    const icon = ICONS[e.subtype] || '';
    const label = LABELS[e.subtype] || e.subtype;
    const meta = [];
    if (e.amount_ml != null) meta.push(`${e.amount_ml} ml`);
    if (e.duration_minutes != null) meta.push(`${e.duration_minutes} min`);
    if (e.notes) meta.push(e.notes);

    return `
      <div class="tl-entry" data-id="${e.id}" data-kind="${e.kind}">
        <div class="tl-dot">${icon}</div>
        <div class="tl-body">
          <div class="tl-info">
            <div class="tl-type">${label}</div>
            ${meta.length ? `<div class="tl-meta">${meta.join(' \u00b7 ')}</div>` : ''}
          </div>
          <div class="tl-time">${formatTime(e.timestamp)}</div>
        </div>
      </div>
    `;
  }).join('');

  // Tap entry to edit
  $timeline.querySelectorAll('.tl-entry').forEach(el => {
    el.addEventListener('click', () => {
      const id = parseInt(el.dataset.id);
      const kind = el.dataset.kind;
      const entry = entries.find(e => e.id === id);
      if (entry) openEditModal(entry);
    });
  });
}

function renderDaySummary() {
  const ds = dateStr(currentDate);
  let summary;
  try {
    summary = JSON.parse(tracker.getSummary(undefined, `${ds}T00:00:00`));
  } catch {
    summary = { total_feedings: 0, total_ml: 0, total_minutes: 0, total_urine: 0, total_poop: 0 };
  }

  const { total_feedings, total_ml, total_minutes, total_urine, total_poop } = summary;
  const hasAnything = total_feedings + total_urine + total_poop > 0;

  if (!hasAnything) {
    $daySummary.innerHTML = '';
    return;
  }

  $daySummary.innerHTML = `
    <div class="day-stat"><span class="val">${total_feedings}</span><br>feedings</div>
    <div class="day-stat"><span class="val">${total_ml > 0 ? Math.round(total_ml) + ' ml' : '\u2014'}</span><br>volume</div>
    <div class="day-stat"><span class="val">${total_minutes > 0 ? total_minutes + ' min' : '\u2014'}</span><br>nursing</div>
    <div class="day-stat"><span class="val">${total_urine}\u{1F4A7} ${total_poop}\u{1F4A9}</span><br>diapers</div>
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
      const kind = btn.dataset.kind;
      const type = btn.dataset.type;
      hideTypePicker();

      if (kind === 'dejection') {
        // Dejections are instant - save immediately
        const name = getBabyName();
        const timestamp = toISOTimestamp(new Date());
        try {
          tracker.addDejection(name, type, undefined, timestamp);
          save();
          // Navigate to today and re-render
          currentDate = new Date();
          resetToStartOfDay(currentDate);
          render();
        } catch (err) {
          console.error('Failed to save dejection:', err);
        }
      } else {
        // Feedings use the timer
        startFeeding(type);
      }
    });
  });
}

// --- Edit Modal ---

const $editModal = document.getElementById('edit-modal');
const $editId = document.getElementById('edit-id');
const $editKind = document.getElementById('edit-kind');
const $editSubtype = document.getElementById('edit-subtype');
const $editAmount = document.getElementById('edit-amount');
const $editDuration = document.getElementById('edit-duration');
const $editNotes = document.getElementById('edit-notes');
const $editTime = document.getElementById('edit-time');
const $editFeedingFields = document.getElementById('edit-feeding-fields');
const $editDelete = document.getElementById('edit-delete');
const $editSave = document.getElementById('edit-save');

function openEditModal(entry) {
  $editId.value = entry.id;
  $editKind.value = entry.kind;

  // Populate subtype options
  const subtypes = entry.kind === 'feeding' ? FEEDING_SUBTYPES : DEJECTION_SUBTYPES;
  $editSubtype.innerHTML = subtypes.map(s =>
    `<option value="${s}" ${s === entry.subtype ? 'selected' : ''}>${LABELS[s]}</option>`
  ).join('');

  // Show/hide feeding-specific fields
  if (entry.kind === 'feeding') {
    $editFeedingFields.classList.remove('hidden');
    $editAmount.value = entry.amount_ml != null ? entry.amount_ml : '';
    $editDuration.value = entry.duration_minutes != null ? entry.duration_minutes : '';
  } else {
    $editFeedingFields.classList.add('hidden');
    $editAmount.value = '';
    $editDuration.value = '';
  }

  $editNotes.value = entry.notes || '';
  $editTime.value = toDatetimeLocal(entry.timestamp);

  $editModal.classList.remove('hidden');
}

function hideEditModal() {
  $editModal.classList.add('hidden');
}

function setupEditModal() {
  // Backdrop dismiss
  $editModal.querySelector('.edit-modal-backdrop').addEventListener('click', hideEditModal);

  // Save
  $editSave.addEventListener('click', () => {
    const id = parseInt($editId.value);
    const kind = $editKind.value;
    const subtype = $editSubtype.value;
    const notes = $editNotes.value.trim() || undefined;
    const timestamp = $editTime.value + ':00'; // add seconds

    try {
      if (kind === 'feeding') {
        const amount = $editAmount.value ? parseFloat($editAmount.value) : undefined;
        const duration = $editDuration.value ? parseInt($editDuration.value) : undefined;
        tracker.updateFeeding(id, subtype, amount, duration, notes, timestamp);
      } else {
        tracker.updateDejection(id, subtype, notes, timestamp);
      }
      save();
      hideEditModal();
      render();
    } catch (err) {
      console.error('Failed to update entry:', err);
    }
  });

  // Delete
  $editDelete.addEventListener('click', () => {
    const id = parseInt($editId.value);
    const kind = $editKind.value;

    if (kind === 'feeding') {
      tracker.deleteFeeding(id);
    } else {
      tracker.deleteDejection(id);
    }
    save();
    hideEditModal();
    render();
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
  setupEditModal();
  setupNamePrompt();

  $btnPrev.addEventListener('click', () => goDay(-1));
  $btnNext.addEventListener('click', () => goDay(1));

  render();

  if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('sw.js').catch(() => {});
  }
}

main();
