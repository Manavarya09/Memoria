import { searchMemories } from '../utils/api.js';
import { formatRelativeTime, truncateText, getActivityIcon } from '../utils/format.js';
import { debounce } from '../utils/debounce.js';

export class SearchView {
  constructor(container) {
    this.container = container;
    this.query = '';
    this.results = [];
    this.loading = false;
    this.render();
  }

  render() {
    this.container.innerHTML = `
      <div class="search-view">
        <div class="search-header">
          <h1 class="search-title">Search Your Memory</h1>
          <p class="search-subtitle">Ask anything you've seen, read, or done on your computer</p>
          <div class="search-bar">
            <span class="search-icon">🔍</span>
            <input 
              type="text" 
              class="search-input" 
              id="search-input" 
              placeholder="What are you looking for?"
              autocomplete="off"
            >
            <button class="search-clear" id="search-clear">✕</button>
          </div>
        </div>
        <div class="search-results" id="search-results"></div>
      </div>
    `;

    this.setupEventListeners();
  }

  setupEventListeners() {
    const input = this.container.querySelector('#search-input');
    const clearBtn = this.container.querySelector('#search-clear');

    const performSearch = debounce(async (query) => {
      if (query.length < 2) {
        this.results = [];
        this.renderResults();
        return;
      }

      this.loading = true;
      this.renderResults();

      try {
        this.results = await searchMemories(query, 20);
      } catch (error) {
        console.error('Search error:', error);
        this.results = [];
      }

      this.loading = false;
      this.renderResults();
    }, 300);

    input.addEventListener('input', (e) => {
      this.query = e.target.value;
      clearBtn.classList.toggle('visible', this.query.length > 0);
      performSearch(this.query);
    });

    clearBtn.addEventListener('click', () => {
      this.query = '';
      input.value = '';
      input.focus();
      clearBtn.classList.remove('visible');
      this.results = [];
      this.renderResults();
    });

    input.focus();
  }

  renderResults() {
    const resultsContainer = this.container.querySelector('#search-results');

    if (this.loading) {
      resultsContainer.innerHTML = `
        <div class="loading-spinner">
          <div class="spinner"></div>
        </div>
      `;
      return;
    }

    if (this.query.length < 2) {
      resultsContainer.innerHTML = `
        <div class="empty-state">
          <div class="empty-state-icon">🧠</div>
          <h3 class="empty-state-title">Start Searching</h3>
          <p class="empty-state-text">Type at least 2 characters to search your digital memories</p>
        </div>
      `;
      return;
    }

    if (this.results.length === 0) {
      resultsContainer.innerHTML = `
        <div class="empty-state">
          <div class="empty-state-icon">🔍</div>
          <h3 class="empty-state-title">No Results Found</h3>
          <p class="empty-state-text">Try different keywords or check back later as your memory grows</p>
        </div>
      `;
      return;
    }

    resultsContainer.innerHTML(result => `
      <div class="search-result card" data-id="${result = this.results.map.id}">
        <div class="card-header">
          <div class="card-icon">${getActivityIcon(result.activity_type)}</div>
          <div>
            <div class="card-title">${truncateText(result.title || result.activity_type, 60)}</div>
            <div class="card-subtitle">${result.app_name || result.activity_type}</div>
          </div>
        </div>
        <div class="card-content">${truncateText(result.content || result.url || '')}</div>
        <div class="card-meta">
          <span>${formatRelativeTime(result.timestamp)}</span>
          <div class="result-score">
            <span>${Math.round(result.score * 100)}%</span>
            <div class="score-bar">
              <div class="score-fill" style="width: ${result.score * 100}%"></div>
            </div>
          </div>
        </div>
      </div>
    `).join('');

    resultsContainer.querySelectorAll('.search-result').forEach(el => {
      el.addEventListener('click', () => {
        const id = el.dataset.id;
        window.dispatchEvent(new CustomEvent('navigate', { detail: { view: 'preview', activityId: id } }));
      });
    });
  }

  show() {
    this.container.classList.remove('hidden');
    this.container.querySelector('#search-input')?.focus();
  }

  hide() {
    this.container.classList.add('hidden');
  }
}
