import { previewActivity, deleteMemory } from '../utils/api.js';
import { formatTimestamp, formatDate, formatActivityType, getActivityIcon } from '../utils/format.js';

export class PreviewView {
  constructor(container) {
    this.container = container;
    this.activityId = null;
    this.activity = null;
    this.render();
  }

  render() {
    this.container.innerHTML = `
      <div class="preview-view">
        <div class="preview-header">
          <h2 class="preview-title">Memory Details</h2>
          <button class="btn btn-secondary" id="preview-back">← Back</button>
        </div>
        <div class="preview-content" id="preview-content">
          <div class="empty-state">
            <div class="empty-state-icon">👆</div>
            <h3 class="empty-state-title">Select a Memory</h3>
            <p class="empty-state-text">Click on any memory to view its details</p>
          </div>
        </div>
      </div>
    `;

    this.container.querySelector('#preview-back').addEventListener('click', () => {
      window.dispatchEvent(new CustomEvent('navigate', { detail: { view: 'search' } }));
    });
  }

  async loadActivity(activityId) {
    this.activityId = activityId;
    const content = this.container.querySelector('#preview-content');

    content.innerHTML = `
      <div class="loading-spinner">
        <div class="spinner"></div>
      </div>
    `;

    try {
      this.activity = await previewActivity(activityId);
      this.renderActivity();
    } catch (error) {
      console.error('Preview error:', error);
      content.innerHTML = `
        <div class="empty-state">
          <div class="empty-state-icon">⚠️</div>
          <h3 class="empty-state-title">Error Loading Memory</h3>
          <p class="empty-state-text">The memory</p>
        </div>
      may have been deleted `;
    }
  }

  renderActivity() {
    const content = this.container.querySelector('#preview-content');

    if (!this.activity) {
      content.innerHTML = `
        <div class="empty-state">
          <div class="empty-state-icon">👆</div>
          <h3 class="empty-state-title">Select a Memory</h3>
          <p class="empty-state-text">Click on any memory to view its details</p>
        </div>
      `;
      return;
    }

    const { id, activity_type, title, app_name, file_path, url, content: activityContent, timestamp, session_id, related_nodes } = this.activity;

    content.innerHTML = `
      <div class="card" style="grid-column: 1 / -1;">
        <div class="card-header">
          <div class="card-icon" style="width: 48px; height: 48px; font-size: 24px;">
            ${getActivityIcon(activity_type)}
          </div>
          <div>
            <div class="card-title" style="font-size: 18px;">${title || formatActivityType(activity_type)}</div>
            <div class="card-subtitle">${app_name || formatActivityType(activity_type)}</div>
          </div>
        </div>
      </div>

      <div class="preview-section">
        <div class="preview-section-title">Details</div>
        <div class="card">
          <div class="settings-row">
            <span class="settings-label">Type</span>
            <span class="badge badge-primary">${formatActivityType(activity_type)}</span>
          </div>
          <div class="settings-row">
            <span class="settings-label">Time</span>
            <span class="settings-value">${formatDate(timestamp)} at ${formatTimestamp(timestamp)}</span>
          </div>
          ${app_name ? `
            <div class="settings-row">
              <span class="settings-label">Application</span>
              <span class="settings-value">${app_name}</span>
            </div>
          ` : ''}
          ${session_id ? `
            <div class="settings-row">
              <span class="settings-label">Session</span>
              <span class="settings-value">${session_id.substring(0, 8)}...</span>
            </div>
          ` : ''}
        </div>
      </div>

      ${url ? `
        <div class="preview-section">
          <div class="preview-section-title">URL</div>
          <div class="card">
            <a href="${url}" target="_blank" class="preview-text" style="color: var(--accent-primary); word-break: break-all;">${url}</a>
          </div>
        </div>
      ` : ''}

      ${file_path ? `
        <div class="preview-section">
          <div class="preview-section-title">File Path</div>
          <div class="card">
            <span class="preview-text" style="font-family: var(--font-mono);">${file_path}</span>
          </div>
        </div>
      ` : ''}

      ${activityContent ? `
        <div class="preview-section" style="grid-column: 1 / -1;">
          <div class="preview-section-title">Content</div>
          <div class="card">
            <pre class="preview-text" style="white-space: pre-wrap; max-height: 300px; overflow-y: auto;">${activityContent}</pre>
          </div>
        </div>
      ` : ''}

      ${related_nodes && related_nodes.length > 0 ? `
        <div class="preview-section" style="grid-column: 1 / -1;">
          <div class="preview-section-title">Related Memories</div>
          <div class="card">
            ${related_nodes.map(node => `
              <div class="timeline-item" data-id="${node.id}">
                <div class="timeline-icon">${getActivityIcon(node.node_type)}</div>
                <div class="timeline-content">
                  <div class="timeline-item-title">${node.label}</div>
                  <div class="timeline-item-subtitle">${node.edge_type}</div>
                </div>
              </div>
            `).join('')}
          </div>
        </div>
      ` : ''}

      <div class="preview-actions" style="grid-column: 1 / -1;">
        <button class="btn btn-danger" id="delete-memory" data-id="${id}">
          🗑️ Delete Memory
        </button>
      </div>
    `;

    content.querySelector('#delete-memory').addEventListener('click', async () => {
      if (confirm('Are you sure you want to delete this memory?')) {
        try {
          await deleteMemory(id);
          window.dispatchEvent(new CustomEvent('navigate', { detail: { view: 'search' } }));
        } catch (error) {
          console.error('Delete error:', error);
        }
      }
    });

    content.querySelectorAll('.timeline-item[data-id]').forEach(el => {
      el.addEventListener('click', () => {
        window.dispatchEvent(new CustomEvent('navigate', { detail: { view: 'preview', activityId: el.dataset.id } }));
      });
    });
  }

  show() {
    this.container.classList.remove('hidden');
  }

  hide() {
    this.container.classList.add('hidden');
  }
}
