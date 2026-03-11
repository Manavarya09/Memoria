import { getMemoryGraph } from '../utils/api.js';

export class GraphView {
  constructor(container) {
    this.container = container;
    this.graph = null;
    this.render();
  }

  async render() {
    this.container.innerHTML = `
      <div class="graph-view">
        <div class="graph-header">
          <h2 class="graph-title">Memory Graph</h2>
          <button class="btn btn-secondary" id="refresh-graph">Refresh</button>
        </div>
        <div class="graph-container" id="graph-container">
          <div class="loading-spinner">
            <div class="spinner"></div>
          </div>
        </div>
      </div>
    `;

    this.setupEventListeners();
    await this.loadGraph();
  }

  setupEventListeners() {
    this.container.querySelector('#refresh-graph').addEventListener('click', () => {
      this.loadGraph();
    });
  }

  async loadGraph() {
    const container = this.container.querySelector('#graph-container');
    
    container.innerHTML = `
      <div class="loading-spinner">
        <div class="spinner"></div>
      </div>
    `;

    try {
      this.graph = await getMemoryGraph();
      this.renderGraph();
    } catch (error) {
      console.error('Graph error:', error);
      container.innerHTML = `
        <div class="empty-state">
          <div class="empty-state-icon">⚠️</div>
          <h3 class="empty-state-title">Error Loading Graph</h3>
          <p class="empty-state-text">Please try again later</p>
        </div>
      `;
    }
  }

  renderGraph() {
    const container = this.container.querySelector('#graph-container');

    if (!this.graph || this.graph.nodes.length === 0) {
      container.innerHTML = `
        <div class="empty-state">
          <div class="empty-state-icon">🔗</div>
          <h3 class="empty-state-title">No Connections Yet</h3>
          <p class="empty-state-text">Your memory connections will appear here as the system learns</p>
        </div>
      `;
      return;
    }

    const width = container.clientWidth;
    const height = container.clientHeight;
    const centerX = width / 2;
    const centerY = height / 2;
    const radius = Math.min(width, height) * 0.35;

    const nodePositions = {};
    const totalNodes = this.graph.nodes.length;
    
    this.graph.nodes.forEach((node, i) => {
      const angle = (2 * Math.PI * i) / totalNodes - Math.PI / 2;
      nodePositions[node.id] = {
        x: centerX + radius * Math.cos(angle),
        y: centerY + radius * Math.sin(angle),
        node
      };
    });

    container.innerHTML = `
      <svg class="graph-canvas" viewBox="0 0 ${width} ${height}">
        <defs>
          <filter id="glow">
            <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
            <feMerge>
              <feMergeNode in="coloredBlur"/>
              <feMergeNode in="SourceGraphic"/>
            </feMerge>
          </filter>
        </defs>
        ${this.graph.edges.map(edge => {
          const source = nodePositions[edge.source];
          const target = nodePositions[edge.target];
          if (!source || !target) return '';
          return `
            <line 
              x1="${source.x}" y1="${source.y}"
              x2="${target.x}" y2="${target.y}"
              stroke="rgba(255,255,255,0.1)"
              stroke-width="2"
            />
          `;
        }).join('')}
        ${Object.values(nodePositions).map(({ x, y, node }) => `
          <g class="graph-node ${node.node_type}" data-id="${node.id}" transform="translate(${x - 20}, ${y - 20})">
            <circle cx="20" cy="20" r="20" filter="url(#glow)"/>
            <text x="20" y="25" text-anchor="middle" font-size="14">${this.getNodeIcon(node.node_type)}</text>
            <title>${node.label}</title>
          </g>
        `).join('')}
      </svg>
    `;

    container.querySelectorAll('.graph-node').forEach(el => {
      el.addEventListener('click', () => {
        const id = el.dataset.id;
        window.dispatchEvent(new CustomEvent('navigate', { detail: { view: 'preview', activityId: id } }));
      });
    });
  }

  getNodeIcon(nodeType) {
    const icons = {
      'app': '⌨',
      'file': '📄',
      'url': '🌐',
      'session': '📁',
      'clipboard': '📋',
      'screenshot': '📸'
    };
    return icons[nodeType] || '📌';
  }

  show() {
    this.container.classList.remove('hidden');
    setTimeout(() => this.renderGraph(), 100);
  }

  hide() {
    this.container.classList.add('hidden');
  }
}
