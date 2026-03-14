/**
 * Stylus-Trace Studio Viewer Logic
 */

const CONFIG = {
    rowHeight: 24,
    minBoxWidth: 0.5,
    font: '12px "Inter", sans-serif',
    colors: {
        StorageExpensive: '#dc143c',
        StorageNormal: '#ff8c00',
        Crypto: '#8a2be2',
        Memory: '#228b22',
        Call: '#4682b4',
        System: '#6495ed',
        Root: '#4b0082',
        UserCode: '#848d97',
        Background: '#010409',
        Border: '#30363d',
        Text: '#e6edf3',
        Regression: '#ff4d4d',
        Improvement: '#2ecc71',
        Stable: '#f0f0f0'
    }
};

class Flamegraph {
    constructor(canvasId, data, isDiff = false) {
        this.canvas = document.getElementById(canvasId);
        this.ctx = this.canvas.getContext('2d', { alpha: false });
        this.data = data;
        this.isDiff = isDiff;
        this.zoom = 1.0;
        this.offsetX = 0;
        this.nodes = [];
        this.hoveredNode = null;
        
        this.init();
    }

    init() {
        this.buildTree();
        this.setupListeners();
        this.resize();
        window.addEventListener('resize', () => this.resize());
    }

    buildTree() {
        if (!this.data) return;
        
        // Handle both Profile and Diff reports
        if (this.data.summary && this.data.metrics) { 
            // This is a diff report
            this.tree = this.convertDiffToTree(this.data);
        } else if (this.data.all_stacks) {
            // This is a standard profile
            this.tree = this.convertStacksToTree(this.data.all_stacks, this.data.total_gas);
        }
    }

    convertStacksToTree(stacks, totalGas) {
        const root = { name: 'root', value: totalGas, children: {}, depth: 0 };
        stacks.forEach(stack => {
            let current = root;
            const parts = stack.stack.split(';');
            if (parts[0] === 'root') parts.shift();

            parts.forEach((part, i) => {
                if (!current.children[part]) {
                    current.children[part] = { name: part, value: 0, children: {}, depth: i + 1 };
                }
                current = current.children[part];
                if (i === parts.length - 1) {
                    current.value += stack.weight;
                }
            });
        });
        this.calculateCumulative(root);
        return root;
    }

    convertDiffToTree(diffReport) {
        // Implement complex diff tree reconstruction if needed
        // For now, if we have Profile B data, we render two separate graphs
        return null; 
    }

    calculateCumulative(node) {
        let childWeight = Object.values(node.children).reduce((acc, child) => acc + this.calculateCumulative(child), 0);
        node.value = Math.max(node.value, childWeight);
        return node.value;
    }

    resize() {
        const dpr = window.devicePixelRatio || 1;
        const rect = this.canvas.parentElement.getBoundingClientRect();
        this.canvas.width = rect.width * dpr;
        this.canvas.height = rect.height * dpr;
        this.ctx.scale(dpr, dpr);
        this.render();
    }

    setupListeners() {
        this.canvas.addEventListener('mousemove', (e) => {
            const rect = this.canvas.getBoundingClientRect();
            const x = (e.clientX - rect.left) / this.zoom - this.offsetX;
            const y = e.clientY - rect.top;
            this.handleMouseMove(x, y, e.clientX, e.clientY);
        });

        this.canvas.addEventListener('mousedown', (e) => {
            this.isDragging = true;
            this.lastX = e.clientX;
        });

        window.addEventListener('mouseup', () => this.isDragging = false);

        window.addEventListener('mousemove', (e) => {
            if (this.isDragging) {
                const dx = (e.clientX - this.lastX) / this.zoom;
                this.offsetX += dx;
                this.lastX = e.clientX;
                this.render();
                if (window.app.syncZoom) {
                    const other = this === window.app.flamegraphA ? window.app.flamegraphB : window.app.flamegraphA;
                    if (other) {
                        other.offsetX = this.offsetX;
                        other.render();
                    }
                }
            }
        });

        this.canvas.addEventListener('wheel', (e) => {
            e.preventDefault();
            const rect = this.canvas.getBoundingClientRect();
            const mouseX = e.clientX - rect.left;
            
            const oldZoom = this.zoom;
            this.zoom *= e.deltaY > 0 ? 0.9 : 1.1;
            this.zoom = Math.max(0.1, Math.min(this.zoom, 100));

            // Zoom towards mouse
            this.offsetX -= (mouseX / oldZoom - mouseX / this.zoom);
            
            this.render();
            if (window.app.syncZoom) {
                const other = this === window.app.flamegraphA ? window.app.flamegraphB : window.app.flamegraphA;
                if (other) {
                    other.zoom = this.zoom;
                    other.offsetX = this.offsetX;
                    other.render();
                }
            }
        }, { passive: false });
    }

    handleMouseMove(x, y, screenX, screenY) {
        const hit = this.nodes.find(node => 
            x >= node.x && x <= node.x + node.w &&
            y >= node.y && y <= node.y + node.h
        );

        if (hit !== this.hoveredNode) {
            this.hoveredNode = hit;
            this.updateTooltip(screenX, screenY);
            this.render();
        }
    }

    updateTooltip(x, y) {
        const tooltip = document.getElementById('tooltip');
        if (this.hoveredNode) {
            tooltip.style.display = 'block';
            tooltip.style.left = (x + 15) + 'px';
            tooltip.style.top = (y + 15) + 'px';
            
            let colorDot = `<span style="display:inline-block;width:10px;height:10px;background:${CONFIG.colors[this.getCategory(this.hoveredNode.name)]};margin-right:8px;border-radius:2px"></span>`;
            
            tooltip.innerHTML = `
                <div style="font-weight:600;margin-bottom:4px;border-bottom:1px solid #333;padding-bottom:4px">${colorDot}${this.hoveredNode.name}</div>
                <div style="display:grid;grid-template-columns:auto 1fr;gap:8px;font-family:'JetBrains Mono'">
                    <span>Gas:</span> <span style="text-align:right">${this.hoveredNode.value.toLocaleString()}</span>
                    <span>Pct:</span> <span style="text-align:right">${((this.hoveredNode.value / this.tree.value) * 100).toFixed(2)}%</span>
                </div>
            `;
        } else {
            tooltip.style.display = 'none';
        }
    }

    render() {
        const width = this.canvas.width / (window.devicePixelRatio || 1);
        const height = this.canvas.height / (window.devicePixelRatio || 1);

        this.ctx.fillStyle = CONFIG.colors.Background;
        this.ctx.fillRect(0, 0, width, height);
        this.nodes = [];

        if (!this.tree) return;

        const self = this;
        function renderNode(node, x, depth, w) {
            const screenX = (x + self.offsetX) * self.zoom;
            const screenW = w * self.zoom;

            if (screenX + screenW < 0 || screenX > width) return; // Frustum culling
            if (screenW < CONFIG.minBoxWidth) return;

            const y = height - (depth + 1) * CONFIG.rowHeight - 40;
            const h = CONFIG.rowHeight;

            const category = self.getCategory(node.name);
            const color = CONFIG.colors[category] || CONFIG.colors.UserCode;

            self.ctx.fillStyle = color;
            self.ctx.fillRect(screenX, y, screenW, h);
            self.ctx.strokeStyle = CONFIG.colors.Background;
            self.ctx.lineWidth = 0.5;
            self.ctx.strokeRect(screenX, y, screenW, h);

            if (self.hoveredNode && self.hoveredNode.name === node.name) {
                self.ctx.strokeStyle = '#fff';
                self.ctx.lineWidth = 2;
                self.ctx.strokeRect(screenX, y, screenW, h);
            }

            if (screenW > 40) {
                self.ctx.fillStyle = '#fff';
                self.ctx.font = CONFIG.font;
                self.ctx.textAlign = 'left';
                const label = self.truncateText(node.name, screenW - 8);
                if (label) self.ctx.fillText(label, screenX + 4, y + h / 2 + 4);
            }

            self.nodes.push({ name: node.name, x, y, w, h, value: node.value });

            let currentX = x;
            Object.values(node.children)
                .sort((a, b) => b.value - a.value)
                .forEach(child => {
                    const childW = (child.value / node.value) * w;
                    renderNode(child, currentX, depth + 1, childW);
                    currentX += childW;
                });
        }

        renderNode(this.tree, 0, 0, width / this.zoom);
    }

    getCategory(name) {
        if (name === 'root') return 'Root';
        if (name.includes('storage_store') || name.includes('storage_flush')) return 'StorageExpensive';
        if (name.includes('storage_load') || name.includes('storage_cache')) return 'StorageNormal';
        if (name.includes('keccak')) return 'Crypto';
        if (name.includes('memory') || name.includes('args') || name.includes('result')) return 'Memory';
        if (name.includes('call') || name.includes('create')) return 'Call';
        if (name.includes('host') || name.includes('msg') || name.includes('block')) return 'System';
        return 'UserCode';
    }

    truncateText(text, maxWidth) {
        let width = this.ctx.measureText(text).width;
        if (width <= maxWidth) return text;
        let truncated = text;
        while (truncated.length > 0 && width > maxWidth) {
            truncated = truncated.slice(0, -1);
            width = this.ctx.measureText(truncated + '...').width;
        }
        return truncated ? truncated + '...' : '';
    }
}

// Global App State
window.app = {
    profileA: null,
    profileB: null,
    diff: null,
    flamegraphA: null,
    flamegraphB: null,
    syncZoom: true
};

document.addEventListener('DOMContentLoaded', () => {
    loadData();
    setupControls();
    
    if (window.app.profileA) {
        updateUI(window.app.profileA);
        window.app.flamegraphA = new Flamegraph('canvas-a', window.app.profileA);
    }
    
    if (window.app.profileB) {
        document.getElementById('flamegraph-b').classList.remove('hidden');
        document.getElementById('toggle-diff').classList.remove('hidden');
        window.app.flamegraphB = new Flamegraph('canvas-b', window.app.profileB);
    }
});

function loadData() {
    try {
        const getJson = id => {
            const el = document.getElementById(id);
            if (!el) return null;
            const text = el.textContent.trim();
            return (text && !text.startsWith('/*')) ? JSON.parse(text) : null;
        };

        window.app.profileA = getJson('profile-data');
        window.app.profileB = getJson('profile-b-data');
        window.app.diff = getJson('diff-data');
    } catch (e) {
        console.error('Data loading error', e);
    }
}

function setupControls() {
    const zoomIn = () => {
        window.app.flamegraphA.zoom *= 1.2;
        window.app.flamegraphA.render();
        if (window.app.flamegraphB) {
            window.app.flamegraphB.zoom = window.app.flamegraphA.zoom;
            window.app.flamegraphB.render();
        }
    };
    
    const zoomOut = () => {
        window.app.flamegraphA.zoom *= 0.8;
        window.app.flamegraphA.render();
        if (window.app.flamegraphB) {
            window.app.flamegraphB.zoom = window.app.flamegraphA.zoom;
            window.app.flamegraphB.render();
        }
    };

    const reset = () => {
        window.app.flamegraphA.zoom = 1.0;
        window.app.flamegraphA.offsetX = 0;
        window.app.flamegraphA.render();
        if (window.app.flamegraphB) {
            window.app.flamegraphB.zoom = 1.0;
            window.app.flamegraphB.offsetX = 0;
            window.app.flamegraphB.render();
        }
    }

    document.getElementById('zoom-in').onclick = zoomIn;
    document.getElementById('zoom-out').onclick = zoomOut;
    document.getElementById('reset-view').onclick = reset;
    
    document.getElementById('search-input').oninput = (e) => {
        const query = e.target.value.toLowerCase();
        // Implement search highlight in render
        window.app.searchQuery = query;
        window.app.flamegraphA.render();
        if (window.app.flamegraphB) window.app.flamegraphB.render();
    };

    window.addEventListener('keydown', (e) => {
        if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
            e.preventDefault();
            document.getElementById('search-input').focus();
        }
    });
}

function updateUI(profile) {
    document.querySelector('.tx-hash').textContent = profile.transaction_hash;
    document.getElementById('total-gas').textContent = profile.total_gas.toLocaleString();
    document.getElementById('total-hostios').textContent = profile.hostio_summary.total_calls;
    document.getElementById('profile-name').textContent = profile.transaction_hash.slice(0, 10) + '...';

    const hotPathsList = document.getElementById('hot-paths-list');
    profile.hot_paths.slice(0, 15).forEach(path => {
        const li = document.createElement('li');
        li.className = 'hot-path-item';
        const name = path.stack.split(';').pop();
        li.innerHTML = `
            <div style="display:flex;justify-content:space-between;align-items:center">
                <span class="percentage">${path.percentage.toFixed(1)}%</span>
                <span style="font-size:10px;color:#888">${(path.gas / 1000).toFixed(0)}k gas</span>
            </div>
            <span class="stack-name">${name}</span>
        `;
        li.onclick = () => {
             // Logic to find and zoom to node
        };
        hotPathsList.appendChild(li);
    });
}
