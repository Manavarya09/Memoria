import { getStatus, getDebugMetrics, togglePause } from './utils/api.js';
import { SearchView } from './views/search.js';
import { TimelineView } from './views/timeline.js';
import { GraphView } from './views/graph.js';
import { PreviewView } from './views/preview.js';
import { SettingsView } from './views/settings.js';
import { formatNumber } from './utils/format.js';

class App {
  constructor() {
    this.currentView = 'search';
    this.debugMode = false;
    this.views = {};
    this.statusInterval = null;
    this.init();
  }

  init() {
    this.setupNavigation();
    this.setupKeyboardShortcuts();
    this.setupViews();
    this.setupDebugMode();
    this.startStatusUpdates();
  }

  setupNavigation() {
    const navItems = document.querySelectorAll('.nav-item');
    navItems.forEach(item => {
      item.addEventListener('click', (e) => {
        const view = e.currentTarget.dataset.view;
        this.navigate(view);
      });
    });

    window.addEventListener('navigate', (e) => {
      const { view, activityId } = e.detail;
      this.navigate(view, activityId);
    });
  }

  setupKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
      if (e.metaKey && e.shiftKey && e.key === 'd') {
        e.preventDefault();
        this.toggleDebugMode();
      }
      
      if (e.key === 'Escape') {
        this.navigate('search');
      }
    });
  }

  setupViews() {
    this.views = {
      search: new SearchView(document.getElementById('view-search')),
      timeline: new TimelineView(document.getElementById('view-timeline')),
      graph: new GraphView(document.getElementById('view-graph')),
      preview: new PreviewView(document.getElementById('view-preview')),
      settings: new SettingsView(document.getElementById('view-settings'))
    };
  }

  setupDebugMode() {
    const debugPanel = document.getElementById('debug-panel');
    document.querySelector('.status-right').addEventListener('click', () => {
      this.toggleDebugMode();
    });
  }

  toggleDebugMode() {
    this.debugMode = !this.debugMode;
    const debugPanel = document.getElementById('debug-panel');
    debugPanel.classList.toggle('hidden', !this.debugMode);
    
    if (this.debugMode) {
      this.updateDebugMetrics();
    }
  }

  async updateDebugMetrics() {
    if (!this.debugMode) return;

    try {
      const metrics = await getDebugMetrics();
      const debugPanel = document.getElementById('debug-panel');
      
      debugPanel.querySelector('.debug-value[data-metric="events"]').textContent = formatNumber(metrics.events_captured || 0);
      debugPanel.querySelector('.debug-value[data-metric="screenshots"]').textContent = formatNumber(metrics.screenshots_captured || 0);
      debugPanel.querySelector('.debug-value[data-metric="ocr"]').textContent = formatNumber(metrics.ocr_processed || 0);
      debugPanel.querySelector('.debug-value[data-metric="embeddings"]').textContent = formatNumber(metrics.embeddings_generated || 0);
      debugPanel.querySelector('.debug-value[data-metric="searches"]').textContent = formatNumber(metrics.search_queries || 0);
      debugPanel.querySelector('.debug-value[data-metric="queue"]').textContent = metrics.indexing_queue_size || 0;
      debugPanel.querySelector('.debug-value[data-metric="latency"]').textContent = (metrics.vector_search_latency_ms || 0) + 'ms';
      debugPanel.querySelector('.debug-value[data-metric="uptime"]').textContent = this.formatUptime(metrics.uptime_seconds || 0);
    } catch (error) {
      console.error('Debug metrics error:', error);
    }

    setTimeout(() => this.updateDebugMetrics(), 2000);
  }

  formatUptime(seconds) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  }

  navigate(view, activityId = null) {
    this.currentView = view;

    document.querySelectorAll('.nav-item').forEach(item => {
      item.classList.toggle('active', item.dataset.view === view);
    });

    Object.entries(this.views).forEach(([key, viewObj]) => {
      if (key === 'preview') {
        if (view === 'preview' && activityId) {
          viewObj.show();
          viewObj.loadActivity(activityId);
        } else {
          viewObj.hide();
        }
      } else if (key === view) {
        viewObj.show();
      } else {
        viewObj.hide();
      }
    });
  }

  async startStatusUpdates() {
    const updateStatus = async () => {
      try {
        const status = await getStatus();
        this.updateStatusBar(status);
      } catch (error) {
        console.error('Status update error:', error);
      }
    };

    await updateStatus();
    this.statusInterval = setInterval(updateStatus, 5000);
  }

  updateStatusBar(status) {
    const dot = document.querySelector('.recording-dot');
    if (dot) {
      dot.classList.toggle('paused', status.is_paused);
    }

    const countEl = document.querySelector('.activity-count');
    if (countEl) {
      countEl.textContent = formatNumber(status.activity_count || 0);
    }

    const pendingEl = document.querySelector('.pending-count');
    if (pendingEl) {
      pendingEl.textContent = status.pending_embeddings?.toString() || '0';
    }
  }
}

document.addEventListener('DOMContentLoaded', () => {
  new App();
});
