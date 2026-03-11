import { getTimeline } from '../utils/api.js';
import { formatTimestamp, formatDate, getActivityIcon } from '../utils/format.js';

export class TimelineView {
  constructor(container) {
    this.container = container;
    this.days = 7;
    this.timeline = null;
    this.render();
  }

  async render() {
    this.container.innerHTML = `
      <div class="timeline-view">
        <div class="timeline-header">
          <h2 class="timeline-title">Activity Timeline</h2>
          <div class="timeline-filters">
            <button class="btn btn-secondary" data-days="7">Week</button>
            <button class="btn btn-secondary" data-days="30">Month</button>
          </div>
        </div>
        <div class="timeline-content" id="timeline-content">
          <div class="loading-spinner">
            <div class="spinner"></div>
          </div>
        </div>
      </div>
    `;

    this.setupEventListeners();
    await this.loadTimeline();
  }

  setupEventListeners() {
    this.container.querySelectorAll('.btn[data-days]').forEach(btn => {
      btn.addEventListener('click', async (e) => {
        this.days = parseInt(e.target.dataset.days);
        this.container.querySelectorAll('.btn[data-days]').forEach(b => {
          b.classList.toggle('btn-primary', b === e.target);
          b.classList.toggle('btn-secondary', b !== e.target);
        });
        await this.loadTimeline();
      });
    });
  }

  async loadTimeline() {
    const content = this.container.querySelector('#timeline-content');
    content.innerHTML = `
      <div class="loading-spinner">
        <div class="spinner"></div>
      </div>
    `;

    try {
      this.timeline = await getTimeline(this.days);
      this.renderTimeline();
    } catch (error) {
      console.error('Timeline error:', error);
      content.innerHTML = `
        <div class="empty-state">
          <div class="empty-state-icon">⚠️</div>
          <h3 class="empty-state-title">Error Loading Timeline</h3>
          <p class="empty-state-text">Please try again later</p>
        </div>
      `;
    }
  }

  renderTimeline() {
    const content = this.container.querySelector('#timeline-content');

    if (!this.timeline || this.timeline.days.length === 0) {
      content.innerHTML = `
        <div class="empty-state">
          <div class="empty-state-icon">📅</div>
          <h3 class="empty-state-title">No Activity Yet</h3>
          <p class="empty-state-text">Your activity will appear here as you use your computer</p>
        </div>
      `;
      return;
    }

    const today = new Date();
    const getRelative = (dateStr) => {
      const date = new Date(dateStr);
      const diffDays = Math.floor((today - date) / (1000 * 60 * 60 * 24));
      if (diffDays === 0) return 'Today';
      if (diffDays === 1) return 'Yesterday';
      return `${diffDays} days ago`;
    };

    content.innerHTML = this.timeline.days.map(day => `
      <div class="timeline-day">
        <div class="day-header">
          <span class="day-date">${formatDate(new Date(day.date).getTime() / 1000)}</span>
          <span class="day-relative">${getRelative(day.date)}</span>
        </div>
        <div class="timeline-activities">
          ${day.activities.slice(0, 20).map(activity => `
            <div class="timeline-item" data-id="${activity.id}">
              <span class="timeline-time">${formatTimestamp(activity.timestamp)}</span>
              <div class="timeline-icon">${getActivityIcon(activity.activity_type)}</div>
              <div class="timeline-content">
                <div class="timeline-item-title">${activity.title || activity.activity_type}</div>
                <div class="timeline-item-subtitle">${activity.app_name || ''}</div>
              </div>
            </div>
          `).join('')}
        </div>
      </div>
    `).join('');

    content.querySelectorAll('.timeline-item').forEach(el => {
      el.addEventListener('click', () => {
        const id = el.dataset.id;
        window.dispatchEvent(new CustomEvent('navigate', { detail: { view: 'preview', activityId: id } }));
      });
    });
  }

  show() {
    this.container.classList.remove('hidden');
    this.loadTimeline();
  }

  hide() {
    this.container.classList.add('hidden');
  }
}
