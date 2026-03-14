/**
 * Stylus-Trace Studio - Pie Chart Viewer Logic (Retro Tech Theme)
 */

const CONFIG = {
    colors: {
        StorageExpensive: 'rgb(220, 20, 60)',
        StorageNormal: 'rgb(255, 140, 0)',
        Crypto: 'rgb(138, 43, 226)',
        Memory: 'rgb(34, 139, 34)',
        Call: 'rgb(70, 130, 180)',
        System: 'rgb(100, 149, 237)',
        Root: 'rgb(75, 0, 130)',
        UserCode: 'rgb(169, 169, 169)',
        Other: '#002209'
    }
};

class PieChart {
    constructor(canvasId, data, isDiff = false) {
        this.canvas = document.getElementById(canvasId);
        this.ctx = this.canvas.getContext('2d');
        this.data = data;
        this.isDiff = isDiff;
        this.zoom = 1.0;
        this.offsetX = 0;
        this.offsetY = 0;
        this.hoveredSlice = null;
        this.searchQuery = '';
        
        this.init();
    }

    init() {
        this.processData();
        this.setupListeners();
        setTimeout(() => {
            this.resize();
            window.addEventListener('resize', () => this.resize());
        }, 100);
    }

    processData() {
        if (!this.data || !this.data.hot_paths) return;
        
        let total = this.data.total_gas;
        let tracked = 0;
        this.slices = [];
        
        this.data.hot_paths.slice(0, 15).forEach(path => {
            let name = path.stack.split(';').pop();
            const category = this.getCategory(name);
            this.slices.push({
                name: name,
                fullStack: path.stack,
                value: path.gas,
                percentage: path.percentage,
                color: CONFIG.colors[category] || CONFIG.colors.UserCode
            });
            tracked += path.gas;
        });
        
        if (total > tracked) {
            this.slices.push({
                name: 'Other',
                fullStack: 'Other Operations',
                value: total - tracked,
                percentage: ((total - tracked) / total) * 100,
                color: CONFIG.colors.Other
            });
        }
        
        let startAngle = 0;
        this.slices.forEach(slice => {
            let sliceAngle = (slice.value / total) * 2 * Math.PI;
            slice.startAngle = startAngle;
            slice.endAngle = startAngle + sliceAngle;
            startAngle += sliceAngle;
        });
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
            const mouseX = e.clientX - rect.left;
            const mouseY = e.clientY - rect.top;
            this.handleMouseMove(mouseX, mouseY, e.clientX, e.clientY);
        });

        this.canvas.addEventListener('mousedown', (e) => {
            this.isDragging = true;
            this.lastX = e.clientX;
            this.lastY = e.clientY;
        });

        window.addEventListener('mouseup', () => this.isDragging = false);

        window.addEventListener('mousemove', (e) => {
            if (this.isDragging) {
                const dx = (e.clientX - this.lastX);
                const dy = (e.clientY - this.lastY);
                this.offsetX += dx / this.zoom;
                this.offsetY += dy / this.zoom;
                this.lastX = e.clientX;
                this.lastY = e.clientY;
                this.render();
                
                if (window.app.syncZoom) {
                    const other = this === window.app.chartA ? window.app.chartB : window.app.chartA;
                    if (other) {
                        other.offsetX = this.offsetX;
                        other.offsetY = this.offsetY;
                        other.render();
                    }
                }
            }
        });

        this.canvas.addEventListener('wheel', (e) => {
            e.preventDefault();
            this.zoom *= e.deltaY > 0 ? 0.9 : 1.1;
            this.zoom = Math.max(0.1, Math.min(this.zoom, 10));
            this.render();
            if (window.app.syncZoom) {
                const other = this === window.app.chartA ? window.app.chartB : window.app.chartA;
                if (other) {
                    other.zoom = this.zoom;
                    other.render();
                }
            }
        }, { passive: false });
    }

    handleMouseMove(x, y, screenX, screenY) {
        if (!this.slices) return;
        const width = this.canvas.width / (window.devicePixelRatio || 1);
        const height = this.canvas.height / (window.devicePixelRatio || 1);
        const centerX = width / 2;
        const centerY = height / 2;
        const adjustedX = (x - centerX) / this.zoom - this.offsetX;
        const adjustedY = (y - centerY) / this.zoom - this.offsetY;
        const distance = Math.sqrt(adjustedX * adjustedX + adjustedY * adjustedY);
        const radius = Math.min(width, height) / 2.5;
        
        let hit = null;
        if (distance <= radius) {
            let angle = Math.atan2(adjustedY, adjustedX);
            if (angle < 0) angle += 2 * Math.PI;
            hit = this.slices.find(slice => angle >= slice.startAngle && angle <= slice.endAngle);
        }

        if (hit !== this.hoveredSlice) {
            this.hoveredSlice = hit;
            this.updateTooltip(screenX, screenY);
            this.render();
            document.querySelectorAll('.hot-path-item').forEach(el => el.classList.remove('highlight'));
            if (hit && hit.name !== 'Other') {
                const el = document.getElementById(`path-${hit.name}`);
                if (el) el.classList.add('highlight');
            }
        }
    }

    updateTooltip(x, y) {
        const tooltip = document.getElementById('tooltip');
        if (this.hoveredSlice) {
            tooltip.style.display = 'block';
            tooltip.style.left = (x + 20) + 'px';
            tooltip.style.top = (y + 20) + 'px';
            tooltip.innerHTML = `
                <div style="font-size: 24px; color: #fff; text-shadow: none;">>${this.hoveredSlice.name}</div>
                <div style="margin-top: 10px;">
                    <div>GAS_USED: ${this.hoveredSlice.value.toLocaleString()}</div>
                    <div>SHARE:    ${this.hoveredSlice.percentage.toFixed(2)}%</div>
                </div>
            `;
        } else {
            tooltip.style.display = 'none';
        }
    }

    render() {
        const width = this.canvas.width / (window.devicePixelRatio || 1);
        const height = this.canvas.height / (window.devicePixelRatio || 1);
        this.ctx.clearRect(0, 0, width, height);

        if (!this.slices) return;

        this.ctx.save();
        this.ctx.translate(width / 2, height / 2);
        this.ctx.scale(this.zoom, this.zoom);
        this.ctx.translate(this.offsetX, this.offsetY);
        
        const radius = Math.min(width, height) / 2.5;
        
        this.slices.forEach(slice => {
            this.ctx.beginPath();
            this.ctx.moveTo(0, 0);
            this.ctx.arc(0, 0, radius, slice.startAngle, slice.endAngle);
            this.ctx.closePath();
            
            this.ctx.fillStyle = slice.color;
            let isHighlighted = (this.hoveredSlice === slice) || 
                               (this.searchQuery && slice.name.toLowerCase().includes(this.searchQuery) && slice.name !== 'Other');
            
            if (isHighlighted) this.ctx.fillStyle = '#ffffff'; 
            
            this.ctx.fill();
            this.ctx.strokeStyle = '#000000';
            this.ctx.lineWidth = 2 / this.zoom;
            this.ctx.stroke();
            
            let midAngle = slice.startAngle + (slice.endAngle - slice.startAngle) / 2;
            if (slice.percentage > 3 && this.zoom > 0.5) {
                let textX = Math.cos(midAngle) * (radius * 0.7);
                let textY = Math.sin(midAngle) * (radius * 0.7);
                this.ctx.fillStyle = '#000';
                this.ctx.font = `${Math.max(12, 16 / this.zoom)}px 'VT323'`;
                this.ctx.textAlign = 'center';
                this.ctx.textBaseline = 'middle';
                this.ctx.fillText(slice.name, textX, textY);
            }
        });
        
        this.ctx.restore();
        
        this.ctx.beginPath();
        this.ctx.arc(width/2, height/2, 5, 0, Math.PI * 2);
        this.ctx.fillStyle = '#00ff41';
        this.ctx.fill();
        this.ctx.beginPath();
        this.ctx.moveTo(width/2 - 20, height/2);
        this.ctx.lineTo(width/2 + 20, height/2);
        this.ctx.moveTo(width/2, height/2 - 20);
        this.ctx.lineTo(width/2, height/2 + 20);
        this.ctx.strokeStyle = '#00ff41';
        this.ctx.lineWidth = 1;
        this.ctx.stroke();
    }

    getCategory(name) {
        const n = name.toLowerCase();
        if (n === 'root') return 'Root';
        if (n.includes('storage_store') || n.includes('storage_flush')) return 'StorageExpensive';
        if (n.includes('storage_load') || n.includes('storage_cache')) return 'StorageNormal';
        if (n.includes('keccak')) return 'Crypto';
        if (n.includes('memory') || n.includes('args') || n.includes('result')) return 'Memory';
        if (n.includes('call') || n.includes('create')) return 'Call';
        if (n.includes('host') || n.includes('msg') || n.includes('block')) return 'System';
        return 'UserCode';
    }
}

window.app = {
    profileA: null,
    profileB: null,
    diff: null,
    chartA: null,
    chartB: null,
    syncZoom: true,
    searchQuery: ''
};

document.addEventListener('DOMContentLoaded', () => {
    loadData();
    setupControls();
    if (window.app.profileA) {
        updateUI();
        window.app.chartA = new PieChart('canvas-a', window.app.profileA);
    }
    if (window.app.profileB) {
        document.getElementById('chart-b').classList.remove('hidden');
        window.app.chartB = new PieChart('canvas-b', window.app.profileB, true);
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
        if (window.app.chartA) {
            window.app.chartA.zoom *= 1.2;
            window.app.chartA.render();
        }
        if (window.app.chartB) {
            window.app.chartB.zoom = window.app.chartA.zoom;
            window.app.chartB.render();
        }
    };
    const zoomOut = () => {
        if (window.app.chartA) {
            window.app.chartA.zoom *= 0.8;
            window.app.chartA.render();
        }
        if (window.app.chartB) {
            window.app.chartB.zoom = window.app.chartA.zoom;
            window.app.chartB.render();
        }
    };
    const reset = () => {
        if (window.app.chartA) {
            window.app.chartA.zoom = 1.0;
            window.app.chartA.offsetX = 0;
            window.app.chartA.offsetY = 0;
            window.app.chartA.render();
        }
        if (window.app.chartB) {
            window.app.chartB.zoom = 1.0;
            window.app.chartB.offsetX = 0;
            window.app.chartB.offsetY = 0;
            window.app.chartB.render();
        }
    }
    document.getElementById('zoom-in').onclick = zoomIn;
    document.getElementById('zoom-out').onclick = zoomOut;
    document.getElementById('reset-view').onclick = reset;
    document.getElementById('search-input').oninput = (e) => {
        window.app.searchQuery = e.target.value.toLowerCase();
        if (window.app.chartA) window.app.chartA.searchQuery = window.app.searchQuery;
        if (window.app.chartB) window.app.chartB.searchQuery = window.app.searchQuery;
        if (window.app.chartA) window.app.chartA.render();
        if (window.app.chartB) window.app.chartB.render();
    };
    window.addEventListener('keydown', (e) => {
        if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
            e.preventDefault();
            document.getElementById('search-input').focus();
        }
    });
}

function updateUI() {
    const profA = window.app.profileA;
    const profB = window.app.profileB;

    // Hashes
    document.getElementById('hash-a').textContent = profA.transaction_hash;
    if (profB) {
        document.getElementById('hash-b').textContent = profB.transaction_hash;
        document.getElementById('gas-label').textContent = 'GAS_DELTA:';
        document.getElementById('hostio-label').textContent = 'HOST_IO_DELTA:';
    } else {
        const targetTx = document.querySelector('.tx-item.target');
        if (targetTx) targetTx.classList.add('hidden');
        document.getElementById('gas-label').textContent = 'TOTAL_GAS:';
        document.getElementById('hostio-label').textContent = 'HOST_IO_CALLS:';
    }

    // Delta Stats Helper
    const formatDelta = (v1, v2) => {
        const diff = v2 - v1;
        const pct = v1 === 0 ? (v2 > 0 ? 100 : 0) : (diff / v1) * 100;
        const sign = diff > 0 ? '+' : '';
        const cls = diff > 0 ? 'pos' : (diff < 0 ? 'neg' : 'neutral');
        return `${v1.toLocaleString()} -> ${v2.toLocaleString()} <span class="delta ${cls}">(${sign}${pct.toFixed(2)}%)</span>`;
    };

    // Gas Delta
    const gasA = profA.total_gas;
    const gasB = profB ? profB.total_gas : gasA;
    if (profB) {
        document.getElementById('gas-delta').innerHTML = formatDelta(gasA, gasB);
    } else {
        document.getElementById('gas-delta').textContent = gasA.toLocaleString();
    }

    // HostIO Delta
    const ioA = profA.hostio_summary?.total_calls || 0;
    const ioB = profB ? (profB.hostio_summary?.total_calls || 0) : ioA;
    if (profB) {
        document.getElementById('hostio-delta').innerHTML = `📈 ${formatDelta(ioA, ioB)}`;
    } else {
        document.getElementById('hostio-delta').textContent = ioA.toLocaleString();
    }

    // Footer
    const profileName = profB ? 
        `${profA.transaction_hash.slice(0, 8)}... vs ${profB.transaction_hash.slice(0, 8)}...` : 
        profA.transaction_hash.slice(0, 10) + '...';
    document.getElementById('profile-name').textContent = profileName;

    // Hot Paths
    const hotPathsList = document.getElementById('hot-paths-list');
    hotPathsList.innerHTML = '';
    
    let pathsToShow = profB ? [...profB.hot_paths] : [...profA.hot_paths];
    
    // sorting logic
    if (profB) {
        pathsToShow.sort((a, b) => {
            const pathA_in_A = profA.hot_paths.find(p => p.stack === a.stack);
            const pathB_in_A = profA.hot_paths.find(p => p.stack === b.stack);
            const diffA = a.gas - (pathA_in_A ? pathA_in_A.gas : 0);
            const diffB = b.gas - (pathB_in_A ? pathB_in_A.gas : 0);
            return Math.abs(diffB) - Math.abs(diffA); // Sort by largest change magnitude
        });
    }

    if (pathsToShow) {
        pathsToShow.slice(0, 10).forEach(path => {
            const li = document.createElement('li');
            li.className = 'hot-path-item';
            const name = path.stack.split(';').pop();
            li.id = `path-${name}`;
            
            let deltaDisplay = '';
            let rightSide = '';
            if (profB) {
                const pathA = profA.hot_paths.find(p => p.stack === path.stack);
                const gasA = pathA ? pathA.gas : 0;
                const gasDiff = path.gas - gasA;
                const gasPct = gasA === 0 ? (path.gas > 0 ? 100 : 0) : (gasDiff / gasA) * 100;
                const sign = gasDiff > 0 ? '+' : '';
                const cls = gasDiff > 0 ? 'pos' : (gasDiff < 0 ? 'neg' : 'neutral');
                deltaDisplay = `<span class="delta ${cls}">${sign}${gasPct.toFixed(2)}%</span>`;
                rightSide = ''; // Don't show gas in hot path for diff
            } else {
                deltaDisplay = `<span>[${path.percentage.toFixed(1)}%]</span>`;
                rightSide = `<span style="opacity: 0.6;">${(path.gas / 1000).toFixed(0)}k gas</span>`;
            }

            li.innerHTML = `
                <div style="display:flex;justify-content:space-between;">
                    ${deltaDisplay}
                    ${rightSide}
                </div>
                <span class="stack-name">> ${name}</span>
            `;
            
            li.onmouseenter = () => {
                if (window.app.chartA) {
                    window.app.chartA.hoveredSlice = window.app.chartA.slices.find(s => s.name === name);
                    window.app.chartA.render();
                }
                if (window.app.chartB) {
                    window.app.chartB.hoveredSlice = window.app.chartB.slices.find(s => s.name === name);
                    window.app.chartB.render();
                }
            };
            li.onmouseleave = () => {
                if (window.app.chartA) window.app.chartA.hoveredSlice = null;
                if (window.app.chartB) window.app.chartB.hoveredSlice = null;
                if (window.app.chartA) window.app.chartA.render();
                if (window.app.chartB) window.app.chartB.render();
            };
            hotPathsList.appendChild(li);
        });
    }
}
