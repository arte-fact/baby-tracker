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
let currentView = 'day'; // 'day' | 'report'
let currentMetric = 'total_feedings';

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

function shortDate(dateString) {
  const d = new Date(dateString + 'T12:00:00');
  return d.toLocaleDateString([], { month: 'short', day: 'numeric' });
}

const ICONS = {
  'breast-left': '\u{1F931}',
  'breast-right': '\u{1F931}',
  'bottle': '\u{1F37C}',
  'solid': '\u{1F963}',
  'urine': '\u{1F4A7}',
  'poop': '\u{1F4A9}',
  'weight': '\u{2696}\u{FE0F}',
};

const LABELS = {
  'breast-left': 'Breast (Left)',
  'breast-right': 'Breast (Right)',
  'bottle': 'Bottle',
  'solid': 'Solid',
  'urine': 'Pee',
  'poop': 'Poop',
  'weight': 'Weight',
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
}

function goDay(offset) {
  currentDate.setDate(currentDate.getDate() + offset);
  resetToStartOfDay(currentDate);
  render();
}

// --- View switching ---

const $viewDay = document.getElementById('view-day');
const $viewReport = document.getElementById('view-report');
const $fab = document.getElementById('fab');

function showView(view) {
  currentView = view;
  if (view === 'day') {
    $viewDay.classList.remove('hidden');
    $viewReport.classList.add('hidden');
    $fab.classList.remove('hidden');
    render();
  } else {
    $viewDay.classList.add('hidden');
    $viewReport.classList.remove('hidden');
    $fab.classList.add('hidden');
    renderReport();
  }
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

    if (Math.abs(dx) > 60 && Math.abs(dx) > Math.abs(dy) * 1.5) {
      if (dx > 0) {
        goDay(-1);
      } else if (!isToday(currentDate)) {
        goDay(1);
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
    if (e.weight_kg != null) meta.push(`${e.weight_kg} kg`);
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

  $timeline.querySelectorAll('.tl-entry').forEach(el => {
    el.addEventListener('click', () => {
      const id = parseInt(el.dataset.id);
      const entry = entries.find(e => e.id === id);
      if (entry) openEditModal(entry);
    });
  });
}

function renderDaySummary() {
  const ds = dateStr(currentDate);
  let summary;
  try {
    summary = JSON.parse(tracker.getSummary(undefined, ds));
  } catch {
    summary = { total_feedings: 0, total_ml: 0, total_minutes: 0, total_urine: 0, total_poop: 0, latest_weight_kg: null };
  }

  const { total_feedings, total_ml, total_minutes, total_urine, total_poop, latest_weight_kg } = summary;
  const hasAnything = total_feedings + total_urine + total_poop > 0 || latest_weight_kg;

  if (!hasAnything) {
    $daySummary.innerHTML = '';
    return;
  }

  let stats = `
    <div class="day-stat"><span class="val">${total_feedings}</span><br>feedings</div>
    <div class="day-stat"><span class="val">${total_ml > 0 ? Math.round(total_ml) + ' ml' : '\u2014'}</span><br>volume</div>
    <div class="day-stat"><span class="val">${total_minutes > 0 ? total_minutes + ' min' : '\u2014'}</span><br>nursing</div>
    <div class="day-stat"><span class="val">${total_urine}\u{1F4A7} ${total_poop}\u{1F4A9}</span><br>diapers</div>
  `;
  if (latest_weight_kg != null) {
    stats += `<div class="day-stat"><span class="val">${latest_weight_kg} kg</span><br>weight</div>`;
  }

  $daySummary.innerHTML = stats;
}

// --- FAB / Timer ---

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

  clearInterval(timerInterval);
  timerInterval = null;
  activeFeeding = null;
  localStorage.removeItem(ACTIVE_KEY);

  $fab.classList.remove('recording');
  $fabTimer.classList.add('hidden');
  $fabIconStart.classList.remove('hidden');

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
  $typePicker.querySelector('.type-picker-backdrop').addEventListener('click', hideTypePicker);

  $typePicker.querySelectorAll('.type-pick-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      const kind = btn.dataset.kind;
      const type = btn.dataset.type;
      hideTypePicker();

      if (kind === 'dejection') {
        const name = getBabyName();
        const timestamp = toISOTimestamp(new Date());
        try {
          tracker.addDejection(name, type, undefined, timestamp);
          save();
          currentDate = new Date();
          resetToStartOfDay(currentDate);
          render();
        } catch (err) {
          console.error('Failed to save dejection:', err);
        }
      } else if (kind === 'feeding-bottle') {
        openSliderModal('bottle');
      } else if (kind === 'weight') {
        openSliderModal('weight');
      } else {
        startFeeding(type);
      }
    });
  });
}

// --- Slider Modal ---

const $sliderModal = document.getElementById('slider-modal');
const $sliderTitle = document.getElementById('slider-title');
const $sliderDisplay = document.getElementById('slider-display');
const $sliderUnit = document.getElementById('slider-unit');
const $sliderInput = document.getElementById('slider-input');
const $sliderMin = document.getElementById('slider-min');
const $sliderMax = document.getElementById('slider-max');
const $sliderNotes = document.getElementById('slider-notes');
const $sliderSave = document.getElementById('slider-save');

let sliderMode = null;

function openSliderModal(mode) {
  sliderMode = mode;
  if (mode === 'bottle') {
    $sliderTitle.textContent = 'Bottle Volume';
    $sliderUnit.textContent = 'ml';
    $sliderInput.min = '0';
    $sliderInput.max = '300';
    $sliderInput.step = '5';
    $sliderInput.value = '90';
    $sliderMin.textContent = '0';
    $sliderMax.textContent = '300';
    $sliderDisplay.textContent = '90';
  } else {
    $sliderTitle.textContent = 'Weight';
    $sliderUnit.textContent = 'kg';
    $sliderInput.min = '1.0';
    $sliderInput.max = '15.0';
    $sliderInput.step = '0.1';
    $sliderInput.value = '4.0';
    $sliderMin.textContent = '1.0';
    $sliderMax.textContent = '15.0';
    $sliderDisplay.textContent = '4.0';
  }
  $sliderNotes.value = '';
  $sliderModal.classList.remove('hidden');
}

function hideSliderModal() {
  $sliderModal.classList.add('hidden');
}

function setupSliderModal() {
  $sliderModal.querySelector('.edit-modal-backdrop').addEventListener('click', hideSliderModal);

  $sliderInput.addEventListener('input', () => {
    $sliderDisplay.textContent = $sliderInput.value;
  });

  $sliderSave.addEventListener('click', () => {
    const name = getBabyName();
    const timestamp = toISOTimestamp(new Date());
    const notes = $sliderNotes.value.trim() || undefined;

    try {
      if (sliderMode === 'bottle') {
        const ml = parseFloat($sliderInput.value);
        tracker.addFeeding(name, 'bottle', ml, undefined, notes, timestamp);
      } else {
        const kg = parseFloat($sliderInput.value);
        tracker.addWeight(name, kg, notes, timestamp);
      }
      save();
      hideSliderModal();
      currentDate = new Date();
      resetToStartOfDay(currentDate);
      render();
    } catch (err) {
      console.error('Failed to save:', err);
    }
  });
}

// --- Edit Modal (type-specific forms) ---

const $editModal = document.getElementById('edit-modal');
const $editSheet = document.getElementById('edit-sheet');

function hideEditModal() {
  $editModal.classList.add('hidden');
}

function setupEditModal() {
  $editModal.querySelector('.edit-modal-backdrop').addEventListener('click', hideEditModal);
}

function openEditModal(entry) {
  const icon = ICONS[entry.subtype] || '';
  const label = LABELS[entry.subtype] || entry.subtype;
  let fieldsHTML = '';

  if (entry.kind === 'feeding') {
    fieldsHTML = buildFeedingForm(entry);
  } else if (entry.kind === 'dejection') {
    fieldsHTML = buildDejectionForm(entry);
  } else if (entry.kind === 'weight') {
    fieldsHTML = buildWeightForm(entry);
  }

  $editSheet.innerHTML = `
    <div class="edit-modal-title">
      <span class="edit-icon">${icon}</span>
      Edit ${label}
    </div>
    ${fieldsHTML}
    <div class="form-group">
      <label for="edit-notes">Notes</label>
      <input type="text" id="edit-notes" value="${escAttr(entry.notes || '')}" placeholder="Optional notes">
    </div>
    <div class="form-group">
      <label for="edit-time">Time</label>
      <input type="datetime-local" id="edit-time" value="${toDatetimeLocal(entry.timestamp)}">
    </div>
    <div class="edit-actions">
      <button id="edit-delete" class="btn-danger">Delete</button>
      <button id="edit-save" class="btn-primary">Save</button>
    </div>
  `;

  // Wire up save and delete
  $editSheet.querySelector('#edit-save').addEventListener('click', () => {
    handleEditSave(entry);
  });
  $editSheet.querySelector('#edit-delete').addEventListener('click', () => {
    handleEditDelete(entry);
  });

  $editModal.classList.remove('hidden');
}

function escAttr(s) {
  return s.replace(/"/g, '&quot;').replace(/</g, '&lt;');
}

// --- Feeding edit form ---

function buildFeedingForm(entry) {
  const subtypes = ['breast-left', 'breast-right', 'bottle', 'solid'];
  const options = subtypes.map(s =>
    `<option value="${s}" ${s === entry.subtype ? 'selected' : ''}>${LABELS[s]}</option>`
  ).join('');

  const amountVal = entry.amount_ml != null ? entry.amount_ml : '';
  const durVal = entry.duration_minutes != null ? entry.duration_minutes : '';

  return `
    <div class="form-group">
      <label for="edit-subtype">Type</label>
      <select id="edit-subtype">${options}</select>
    </div>
    <div class="form-row">
      <div class="form-group">
        <label for="edit-amount">Amount (ml)</label>
        <input type="number" id="edit-amount" min="0" step="5" placeholder="ml" value="${amountVal}">
      </div>
      <div class="form-group">
        <label for="edit-duration">Duration (min)</label>
        <input type="number" id="edit-duration" min="0" placeholder="min" value="${durVal}">
      </div>
    </div>
  `;
}

// --- Dejection edit form ---

function buildDejectionForm(entry) {
  const subtypes = ['urine', 'poop'];
  const options = subtypes.map(s =>
    `<option value="${s}" ${s === entry.subtype ? 'selected' : ''}>${LABELS[s]}</option>`
  ).join('');

  return `
    <div class="form-group">
      <label for="edit-subtype">Type</label>
      <select id="edit-subtype">${options}</select>
    </div>
  `;
}

// --- Weight edit form ---

function buildWeightForm(entry) {
  const kgVal = entry.weight_kg != null ? entry.weight_kg : '';
  return `
    <div class="form-group">
      <label for="edit-weight-kg">Weight (kg)</label>
      <input type="number" id="edit-weight-kg" min="0.1" step="0.1" placeholder="kg" value="${kgVal}">
    </div>
  `;
}

// --- Edit save handler ---

function handleEditSave(entry) {
  const notes = $editSheet.querySelector('#edit-notes').value.trim() || undefined;
  const timestamp = $editSheet.querySelector('#edit-time').value + ':00';
  const id = entry.id;

  try {
    if (entry.kind === 'feeding') {
      const subtype = $editSheet.querySelector('#edit-subtype').value;
      const amountEl = $editSheet.querySelector('#edit-amount');
      const durEl = $editSheet.querySelector('#edit-duration');
      const amount = amountEl.value ? parseFloat(amountEl.value) : undefined;
      const duration = durEl.value ? parseInt(durEl.value) : undefined;
      tracker.updateFeeding(id, subtype, amount, duration, notes, timestamp);
    } else if (entry.kind === 'dejection') {
      const subtype = $editSheet.querySelector('#edit-subtype').value;
      tracker.updateDejection(id, subtype, notes, timestamp);
    } else if (entry.kind === 'weight') {
      const kg = parseFloat($editSheet.querySelector('#edit-weight-kg').value);
      tracker.updateWeight(id, kg, notes, timestamp);
    }
    save();
    hideEditModal();
    render();
  } catch (err) {
    console.error('Failed to update entry:', err);
  }
}

// --- Edit delete handler ---

function handleEditDelete(entry) {
  try {
    if (entry.kind === 'feeding') {
      tracker.deleteFeeding(entry.id);
    } else if (entry.kind === 'dejection') {
      tracker.deleteDejection(entry.id);
    } else if (entry.kind === 'weight') {
      tracker.deleteWeight(entry.id);
    }
    save();
    hideEditModal();
    render();
  } catch (err) {
    console.error('Failed to delete entry:', err);
  }
}

// --- Report View ---

const $reportRange = document.getElementById('report-range');
const $reportMetrics = document.getElementById('report-metrics');
const $reportChart = document.getElementById('report-chart');
const $reportTable = document.getElementById('report-table');

function setupReport() {
  document.getElementById('btn-report').addEventListener('click', () => showView('report'));
  document.getElementById('btn-back-day').addEventListener('click', () => showView('day'));

  $reportMetrics.querySelectorAll('.metric-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      $reportMetrics.querySelector('.active').classList.remove('active');
      btn.classList.add('active');
      currentMetric = btn.dataset.metric;
      renderReport();
    });
  });
}

function getReportData() {
  const end = new Date();
  end.setDate(end.getDate() + 1);
  const start = new Date();
  start.setDate(start.getDate() - 13);
  try {
    return JSON.parse(tracker.getReport(undefined, dateStr(start), dateStr(end)));
  } catch {
    return [];
  }
}

function renderReport() {
  const data = getReportData();
  if (data.length === 0) return;

  $reportRange.textContent = `${shortDate(data[0].date)} - ${shortDate(data[data.length - 1].date)}`;

  const values = data.map(d => d[currentMetric] ?? null);

  drawChart(data, values);
  drawTable(data, values);
}

function drawChart(data, values) {
  const canvas = $reportChart;
  const ctx = canvas.getContext('2d');
  const dpr = window.devicePixelRatio || 1;

  // Fixed bar width for scrollability
  const barW = 28;
  const gap = 10;
  const padLeft = 16;
  const padRight = 16;
  const padTop = 22;
  const padBottom = 28;

  const totalW = padLeft + padRight + data.length * (barW + gap) - gap;
  const containerW = canvas.parentElement.parentElement.clientWidth;
  const w = Math.max(totalW, containerW);
  const h = 200;

  // Set canvas to computed width so it scrolls
  canvas.width = w * dpr;
  canvas.height = h * dpr;
  canvas.style.width = w + 'px';
  canvas.style.height = h + 'px';
  canvas.parentElement.style.width = w + 'px';
  ctx.scale(dpr, dpr);

  ctx.clearRect(0, 0, w, h);

  const numericValues = values.map(v => (v == null ? 0 : v));
  const maxVal = Math.max(...numericValues, 1);

  const chartH = h - padTop - padBottom;

  // Grid lines
  ctx.strokeStyle = '#eee';
  ctx.lineWidth = 1;
  for (let i = 0; i <= 4; i++) {
    const y = padTop + (chartH / 4) * i;
    ctx.beginPath();
    ctx.moveTo(padLeft, y);
    ctx.lineTo(w - padRight, y);
    ctx.stroke();
  }

  const isWeight = currentMetric === 'weight_kg';

  // Bars
  data.forEach((day, i) => {
    const val = numericValues[i];
    const x = padLeft + i * (barW + gap);
    const barH = maxVal > 0 ? (val / maxVal) * chartH : 0;
    const y = padTop + chartH - barH;

    if (isWeight && values[i] == null) {
      // no bar
    } else {
      ctx.fillStyle = val > 0 ? '#6c5ce7' : '#e0e0e0';
      ctx.beginPath();
      ctx.roundRect(x, y, barW, barH || 1, [3, 3, 0, 0]);
      ctx.fill();
    }

    // Date label
    ctx.fillStyle = '#636e72';
    ctx.font = '10px -apple-system, sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText(shortDate(day.date), x + barW / 2, h - 6);
  });

  // Value labels
  ctx.fillStyle = '#6c5ce7';
  ctx.font = 'bold 10px -apple-system, sans-serif';
  ctx.textAlign = 'center';
  data.forEach((day, i) => {
    const val = numericValues[i];
    if (val > 0) {
      const x = padLeft + i * (barW + gap) + barW / 2;
      const barH = (val / maxVal) * chartH;
      const y = padTop + chartH - barH - 4;
      const label = isWeight ? val.toFixed(1) : String(Math.round(val));
      ctx.fillText(label, x, y);
    }
  });
}

function drawTable(data, values) {
  const isWeight = currentMetric === 'weight_kg';
  const unit = {
    total_feedings: '',
    total_ml: ' ml',
    total_minutes: ' min',
    total_urine: '',
    total_poop: '',
    weight_kg: ' kg',
  }[currentMetric] || '';

  const reversed = [...data].reverse();
  const reversedValues = [...values].reverse();

  $reportTable.innerHTML = reversed.map((day, i) => {
    const val = reversedValues[i];
    let display;
    if (val == null || (val === 0 && !isWeight)) {
      display = '\u2014';
    } else {
      display = (isWeight ? val.toFixed(1) : String(Math.round(val))) + unit;
    }
    return `
      <div class="report-row">
        <span class="report-row-date">${shortDate(day.date)}</span>
        <span class="report-row-val">${display}</span>
      </div>
    `;
  }).join('');
}

// --- Name Prompt ---

const $namePrompt = document.getElementById('name-prompt');
const $nameInput = document.getElementById('name-input');
const $nameSave = document.getElementById('name-save');

function setupNamePrompt() {
  if (!getBabyName()) {
    $namePrompt.classList.remove('hidden');
    $nameInput.focus();
  }

  $nameSave.addEventListener('click', saveName);
  $nameInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') saveName();
  });

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
  setupSliderModal();
  setupEditModal();
  setupNamePrompt();
  setupReport();

  $btnPrev.addEventListener('click', () => goDay(-1));
  $btnNext.addEventListener('click', () => goDay(1));

  render();

  if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('sw.js').catch(() => {});
  }
}

main();
