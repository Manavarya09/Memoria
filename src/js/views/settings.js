import { getSettings, updateSettings, togglePause, getStatus, deleteMemory } from '../utils/api.js';

export class SettingsView {
  constructor(container) {
    this.container = container;
    this.settings = null;
    this.status = null;
    this.render();
  }

  async render() {
    this.container.innerHTML = `
      <div class="settings-view">
        <h2 class="settings-title">Settings</h2>

        <div class="settings-section">
          <div class="settings-section-title">Status</div>
          <div class="settings-row">
            <div class="settings-toggle-row">
              <span class="settings-label">Recording</span>
              <span class="settings-description">Pause or resume all activity capture</span>
            </div>
            <button class="toggle" id="pause-toggle">
              <div class="toggle-thumb"></div>
            </button>
          </div>
          <div class="card" id="status-card">
            <div class="settings-row">
              <span class="settings-label">Activities</span>
              <span class="settings-value" id="activity-count">Loading...</span>
            </div>
            <div class="settings-row">
              <span class="settings-label">Screenshots</span>
              <span class="settings-value" id="screenshot-count">Loading...</span>
            </div>
            <div class="settings-row">
              <span class="settings-label">Pending Embeddings</span>
              <span class="settings-value" id="pending-count">Loading...</span>
            </div>
          </div>
        </div>

        <div class="settings-section">
          <div class="settings-section-title">Data Capture</div>
          <div class="settings-row">
            <div class="settings-toggle-row">
              <span class="settings-label">App Monitoring</span>
              <span class="settings-description">Track which applications you use</span>
            </div>
            <button class="toggle" id="app-monitoring-toggle">
              <div class="toggle-thumb"></div>
            </button>
          </div>
          <div class="settings-row">
            <div class="settings-toggle-row">
              <span class="settings-label">Screen Capture</span>
              <span class="settings-description">Periodically capture screenshots for OCR</span>
            </div>
            <button class="toggle" id="screen-capture-toggle">
              <div class="toggle-thumb"></div>
            </button>
          </div>
          <div class="settings-row">
            <div class="settings-toggle-row">
              <span class="settings-label">Clipboard Monitoring</span>
              <span class="settings-description">Record clipboard content</span>
            </div>
            <button class="toggle" id="clipboard-toggle">
              <div class="toggle-thumb"></div>
            </button>
          </div>
        </div>

        <div class="settings-section">
          <div class="settings-section-title">Privacy</div>
          <div class="settings-row">
            <div class="settings-toggle-row">
              <span class="settings-label">Pause on Lock</span>
              <span class="settings-description">Pause recording when screen is locked</span>
            </div>
            <button class="toggle" id="pause-lock-toggle">
              <div class="toggle-thumb"></div>
            </button>
          </div>
          <div class="settings-row">
            <div class="settings-toggle-row">
              <span class="settings-label">Retention Period</span>
              <span class="settings-description">Automatically delete data older than this</span>
            </div>
            <select class="input-field" id="retention-select" style="width: auto;">
              <option value="30">30 days</option>
              <option value="60">60 days</option>
              <option value="90" selected>90 days</option>
              <option value="180">180 days</option>
              <option value="365">1 year</option>
            </select>
          </div>
        </div>

        <div class="settings-section settings-danger">
          <div class="settings-section-title">Danger Zone</div>
          <div class="settings-row">
            <div class="settings-toggle-row">
              <span class="settings-label">Delete All Memory</span>
              <span class="settings-description">Permanently delete all stored memories</span>
            </div>
            <button class="btn btn-danger" id="delete-all">Delete All</button>
          </div>
        </div>
      </div>
    `;

    this.setupEventListeners();
    await this.loadData();
  }

  setupEventListeners() {
    this.container.querySelector('#pause-toggle').addEventListener('click', async () => {
      try {
        const isPaused = await togglePause();
        this.updatePauseToggle(isPaused);
        this.updateStatus();
      } catch (error) {
        console.error('Toggle pause error:', error);
      }
    });

    this.container.querySelectorAll('.toggle[id$="-toggle"]').forEach(toggle => {
      if (toggle.id === 'pause-toggle') return;
      
      toggle.addEventListener('click', async () => {
        toggle.classList.toggle('active');
        await this.saveSettings();
      });
    });

    this.container.querySelector('#retention-select').addEventListener('change', async () => {
      await this.saveSettings();
    });

    this.container.querySelector('#delete-all').addEventListener('click', async () => {
      if (confirm('Are you absolutely sure? This will permanently delete ALL your memories. This action cannot be undone.')) {
        if (confirm('This is your last chance to cancel. Delete everything?')) {
          try {
            await deleteMemory(null);
            await this.loadData();
            alert('All memories have been deleted.');
          } catch (error) {
            console.error('Delete all error:', error);
          }
        }
      }
    });
  }

  async loadData() {
    try {
      this.settings = await getSettings();
      this.status = await getStatus();
      this.applySettings();
      this.updateStatus();
    } catch (error) {
      console.error('Load settings error:', error);
    }
  }

  applySettings() {
    this.updateToggle('app-monitoring-toggle', this.settings.activity_capture?.app_monitoring ?? true);
    this.updateToggle('screen-capture-toggle', this.settings.screen_capture?.enabled ?? true);
    this.updateToggle('clipboard-toggle', this.settings.activity_capture?.clipboard_monitoring ?? true);
    this.updateToggle('pause-lock-toggle', this.settings.privacy?.pause_on_lock ?? true);

    const retention = this.settings.privacy?.retention_days ?? 90;
    this.container.querySelector('#retention-select').value = retention.toString();
  }

  updateToggle(id, value) {
    const toggle = this.container.querySelector(`#${id}`);
    if (toggle) {
      toggle.classList.toggle('active', value);
    }
  }

  updatePauseToggle(isPaused) {
    const toggle = this.container.querySelector('#pause-toggle');
    toggle.classList.toggle('active', isPaused);
    
    const dot = document.querySelector('.recording-dot');
    if (dot) {
      dot.classList.toggle('paused', isPaused);
    }
  }

  updateStatus() {
    if (!this.status) return;

    this.container.querySelector('#activity-count').textContent = this.status.activity_count?.toLocaleString() || '0';
    this.container.querySelector('#screenshot-count').textContent = this.status.screenshot_count?.toLocaleString() || '0';
    this.container.querySelector('#pending-count').textContent = this.status.pending_embeddings?.toString() || '0';

    this.updatePauseToggle(this.status.is_paused);
  }

  async saveSettings() {
    try {
      this.settings.activity_capture.app_monitoring = this.container.querySelector('#app-monitoring-toggle').classList.contains('active');
      this.settings.screen_capture.enabled = this.container.querySelector('#screen-capture-toggle').classList.contains('active');
      this.settings.activity_capture.clipboard_monitoring = this.container.querySelector('#clipboard-toggle').classList.contains('active');
      this.settings.privacy.pause_on_lock = this.container.querySelector('#pause-lock-toggle').classList.contains('active');
      this.settings.privacy.retention_days = parseInt(this.container.querySelector('#retention-select').value);

      await updateSettings(this.settings);
    } catch (error) {
      console.error('Save settings error:', error);
    }
  }

  show() {
    this.container.classList.remove('hidden');
    this.loadData();
  }

  hide() {
    this.container.classList.add('hidden');
  }
}
