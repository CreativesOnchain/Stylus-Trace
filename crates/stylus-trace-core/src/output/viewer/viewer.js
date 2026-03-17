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

/** Helper to get CSS variables for Canvas rendering */
function getThemeColor(varName) {
    return getComputedStyle(document.body).getPropertyValue(varName).trim();
}

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
            // Prefer server-provided category; fall back to legacy heuristic
            // for profiles generated before this feature was added.
            const category = path.category || this.getCategory(name);
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
            this.render();
            document.querySelectorAll('.hot-path-item').forEach(el => el.classList.remove('highlight'));
            if (hit && hit.name !== 'Other') {
                const el = document.getElementById(`path-${hit.name}`);
                if (el) el.classList.add('highlight');
            }
        }
        this.updateTooltip(screenX, screenY);
    }

    updateTooltip(x, y) {
        const tooltip = document.getElementById('tooltip');
        if (this.hoveredSlice) {
            tooltip.style.display = 'block';
            tooltip.style.left = (x + 15) + 'px';
            tooltip.style.top = (y + 15) + 'px';
            tooltip.innerHTML = `
                <div style="font-size: 24px; color: var(--text-bright); text-shadow: none;">>${this.hoveredSlice.name}</div>
                <div style="margin-top: 10px; color: var(--green-main);">
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
            let isHighlighted = (this.hoveredSlice === slice);
            if (!isHighlighted && this.searchQuery && slice.name !== 'Other') {
                const query = this.searchQuery.toLowerCase();
                const sliceName = slice.name.toLowerCase();
                // Highlight if exact match OR if it's a prefix of at least 3 chars
                isHighlighted = (sliceName === query) || (query.length >= 3 && sliceName.startsWith(query));
            }

            if (isHighlighted) this.ctx.fillStyle = getThemeColor('--text-bright') || '#ffffff';

            this.ctx.fill();
            this.ctx.strokeStyle = getThemeColor('--bg-color') || '#000000';
            this.ctx.lineWidth = 2 / this.zoom;
            this.ctx.stroke();

            let midAngle = slice.startAngle + (slice.endAngle - slice.startAngle) / 2;
            if (slice.percentage > 3 && this.zoom > 0.5) {
                let textX = Math.cos(midAngle) * (radius * 0.7);
                let textY = Math.sin(midAngle) * (radius * 0.7);
                this.ctx.fillStyle = '#000'; // Keep black for readability on colored slices
                this.ctx.font = `${Math.max(12, 16 / this.zoom)}px 'VT323'`;
                this.ctx.textAlign = 'center';
                this.ctx.textBaseline = 'middle';
                this.ctx.fillText(slice.name, textX, textY);
            }
        });

        this.ctx.restore();

        const greenMain = getThemeColor('--green-main');
        this.ctx.beginPath();
        this.ctx.arc(width / 2, height / 2, 5, 0, Math.PI * 2);
        this.ctx.fillStyle = greenMain;
        this.ctx.fill();
        this.ctx.beginPath();
        this.ctx.moveTo(width / 2 - 20, height / 2);
        this.ctx.lineTo(width / 2 + 20, height / 2);
        this.ctx.moveTo(width / 2, height / 2 - 20);
        this.ctx.lineTo(width / 2, height / 2 + 20);
        this.ctx.strokeStyle = greenMain;
        this.ctx.lineWidth = 1;
        this.ctx.stroke();
    }

    /** @deprecated Use `path.category` from the JSON profile instead.
     * Kept as a fallback for old profiles that lack the `category` field. */
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
    searchQuery: '',
    availableSymbols: []
};

document.addEventListener('DOMContentLoaded', () => {
    loadData();

    // Check if we correctly loaded data statically
    if (window.app.profileA) {
        initViewer();
    } else {
        // No static data, show the drag-and-drop zone
        const appContainer = document.getElementById('app');
        const dropZoneContainer = document.getElementById('drop-zone-container');
        if (appContainer && dropZoneContainer) {
            appContainer.classList.add('hidden');
            dropZoneContainer.classList.remove('hidden');
            setupDropZone();
        }
    }
});

function initViewer() {
    const dropZoneContainer = document.getElementById('drop-zone-container');
    const appContainer = document.getElementById('app');
    if (dropZoneContainer && appContainer) {
        dropZoneContainer.classList.add('hidden');
        appContainer.classList.remove('hidden');
    }

    setupControls();
    setupTabs();
    setupThemeToggle();
    updateUI();
    if (window.app.profileA) {
        window.app.chartA = new PieChart('canvas-a', window.app.profileA);
    }
    if (window.app.profileB) {
        document.getElementById('chart-b').classList.remove('hidden');
        window.app.chartB = new PieChart('canvas-b', window.app.profileB, true);
    }
    renderFlamegraph();
}

function setupThemeToggle() {
    const btn = document.getElementById('theme-toggle');
    if (!btn) return;

    // Restore persisted preference
    const saved = localStorage.getItem('stylus-trace-theme');
    if (saved === 'light') {
        document.body.setAttribute('data-theme', 'light');
        btn.textContent = '[ DARK ]';
    }

    btn.addEventListener('click', () => {
        const isLight = document.body.getAttribute('data-theme') === 'light';
        if (isLight) {
            document.body.removeAttribute('data-theme');
            btn.textContent = '[ LIGHT ]';
            localStorage.setItem('stylus-trace-theme', 'dark');
        } else {
            document.body.setAttribute('data-theme', 'light');
            btn.textContent = '[ DARK ]';
            localStorage.setItem('stylus-trace-theme', 'light');
        }
        // Re-render charts — canvas doesn't pick up CSS variable changes automatically
        if (window.app.chartA) window.app.chartA.render();
        if (window.app.chartB) window.app.chartB.render();
    });
}

function setupDropZone() {
    const dropZoneCtn = document.getElementById('drop-zone-container');
    const dropZone = dropZoneCtn.querySelector('.drop-zone');
    const fileInput = document.getElementById('file-input');

    dropZone.addEventListener('click', () => fileInput.click());
    
    dropZone.addEventListener('dragover', (e) => { 
        e.preventDefault(); 
        dropZone.classList.add('dragover'); 
    });
    
    dropZone.addEventListener('dragleave', () => {
        dropZone.classList.remove('dragover');
    });
    
    dropZone.addEventListener('drop', (e) => {
        e.preventDefault();
        dropZone.classList.remove('dragover');
        handleFiles(e.dataTransfer.files);
    });
    
    fileInput.addEventListener('change', (e) => {
        handleFiles(e.target.files);
    });
}

function handleFiles(files) {
    if (files.length === 0) return;
    
    const fileA = files[0];
    const fileB = files.length > 1 ? files[1] : null;

    const readJson = (file) => {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = (e) => {
                try {
                    resolve(JSON.parse(e.target.result));
                } catch (err) {
                    reject(err);
                }
            };
            reader.onerror = reject;
            reader.readAsText(file);
        });
    };

    if (fileB) {
        Promise.all([readJson(fileA), readJson(fileB)]).then(results => {
            window.app.profileA = results[0];
            window.app.profileB = results[1];
            initViewer();
        }).catch(err => {
            alert('Error parsing JSON files: ' + err.message);
        });
    } else {
        readJson(fileA).then(result => {
            window.app.profileA = result;
            window.app.profileB = null;
            document.getElementById('chart-b').classList.add('hidden');
            if (window.app.chartA) window.app.chartA = null;
            if (window.app.chartB) window.app.chartB = null;
            initViewer();
        }).catch(err => {
            alert('Error parsing JSON file: ' + err.message);
        });
    }
}


function loadData() {
    try {
        /** Decode a base64-encoded JSON blob injected by the Rust backend.
         * Falls back silently for empty or placeholder values. */
        const getJson = id => {
            const el = document.getElementById(id);
            if (!el) return null;
            const text = el.textContent.trim();
            if (!text || text.startsWith('/*')) return null;
            try {
                return JSON.parse(atob(text));
            } catch (_) {
                // Attempt raw JSON parse for drag-and-drop files loaded via handleFiles()
                return JSON.parse(text);
            }
        };
        window.app.profileA = getJson('profile-data');
        window.app.profileB = getJson('profile-b-data');
        window.app.diffData = getJson('diff-data');

        // Load flamegraph SVG (not base64, injected as raw SVG text)
        const svgEl = document.getElementById('flamegraph-svg-data');
        if (svgEl) {
            const svgText = svgEl.textContent.trim();
            window.app.flamegraphSvg = (svgText && !svgText.startsWith('/*')) ? svgText : null;
        }
    } catch (e) {
        console.error('Data loading error', e);
    }
}

function setupTabs() {
    const tabPie   = document.getElementById('tab-pie');
    const tabFlame = document.getElementById('tab-flame');
    const viewerContainer  = document.getElementById('viewer-container');
    const flamegraphView   = document.getElementById('flamegraph-view');

    if (!tabPie || !tabFlame) return;

    tabPie.addEventListener('click', () => {
        tabPie.classList.add('active');
        tabFlame.classList.remove('active');
        viewerContainer.classList.remove('hidden');
        flamegraphView.classList.add('hidden');
        // Resize charts to ensure they repaint correctly after being shown
        if (window.app.chartA) window.app.chartA.resize();
        if (window.app.chartB) window.app.chartB.resize();
    });

    tabFlame.addEventListener('click', () => {
        tabFlame.classList.add('active');
        tabPie.classList.remove('active');
        viewerContainer.classList.add('hidden');
        flamegraphView.classList.remove('hidden');
    });
}

function renderFlamegraph() {
    const container = document.getElementById('flamegraph-container');
    const emptyMsg  = document.getElementById('flamegraph-empty');
    if (!container) return;

    const svg = window.app.flamegraphSvg;
    if (svg) {
        container.innerHTML = svg;
        if (emptyMsg) emptyMsg.classList.add('hidden');
        // Make SVG responsive
        const svgEl = container.querySelector('svg');
        if (svgEl) {
            svgEl.removeAttribute('width');
            svgEl.removeAttribute('height');
            svgEl.setAttribute('preserveAspectRatio', 'xMinYMin meet');
        }
    } else {
        container.innerHTML = '';
        if (emptyMsg) emptyMsg.classList.remove('hidden');
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

    const searchInput = document.getElementById('search-input');
    
    const searchGhost = document.getElementById('search-ghost');
    searchGhost.textContent = ">_ SEARCH SYMBOLS...";

    searchInput.onfocus = () => { if (searchInput.value === '') searchGhost.textContent = ''; };
    searchInput.onblur = () => { if (searchInput.value === '') searchGhost.textContent = '>_ SEARCH SYMBOLS...'; };

    searchInput.oninput = (e) => {
        const val = e.target.value.toLowerCase();
        
        if (val.length === 0) {
            searchGhost.textContent = "";
            updateSearch('');
            return;
        }

        // Find matches for ghosting/autocomplete
        const suggestion = window.app.availableSymbols.find(s => s.toLowerCase().startsWith(val));
        
        if (suggestion) {
            // Display suggestion in ghost - ensure casing matches what's typed for the prefix
            const typedPrefix = e.target.value;
            const remaining = suggestion.slice(typedPrefix.length);
            searchGhost.textContent = typedPrefix + remaining;
        } else {
            searchGhost.textContent = "";
        }

        updateSearch(val);
    };

    searchInput.onkeydown = (e) => {
        if (e.key === 'Enter' || e.key === 'Tab') {
            const val = searchInput.value.toLowerCase();
            const suggestion = window.app.availableSymbols.find(s => s.toLowerCase().startsWith(val));
            if (suggestion) {
                searchInput.value = suggestion;
                searchGhost.textContent = suggestion; 
                updateSearch(suggestion);
                if (e.key === 'Tab') e.preventDefault();
            }
        }
    };

    function updateSearch(query) {
        window.app.searchQuery = query;
        if (window.app.chartA) {
            window.app.chartA.searchQuery = query;
            window.app.chartA.render();
        }
        if (window.app.chartB) {
            window.app.chartB.searchQuery = query;
            window.app.chartB.render();
        }
    }
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

    const profileName = profB ?
        `${profA.transaction_hash.slice(0, 8)}... vs ${profB.transaction_hash.slice(0, 8)}...` :
        profA.transaction_hash.slice(0, 10) + '...';
    document.getElementById('profile-name').textContent = profileName;

    // Collect symbols for autocomplete with safety guards
    const symbols = new Set();
    if (profA && profA.hot_paths) {
        profA.hot_paths.forEach(p => symbols.add(p.stack.split(';').pop()));
    }
    if (profB && profB.hot_paths) {
        profB.hot_paths.forEach(p => symbols.add(p.stack.split(';').pop()));
    }
    window.app.availableSymbols = Array.from(symbols).sort();

    // Hot Paths
    const hotPathsList = document.getElementById('hot-paths-list');
    hotPathsList.innerHTML = '';

    let pathsToShow = profA ? profA.hot_paths : [];

    if (profB && window.app.diffData && window.app.diffData.deltas && window.app.diffData.deltas.hot_paths) {
        // In diff mode, we show common paths from the diff data
        pathsToShow = window.app.diffData.deltas.hot_paths.common_paths;
        // Sort by magnitude of percentage change for diff view
        pathsToShow.sort((a, b) => Math.abs(b.percent_change) - Math.abs(a.percent_change));
    } else {
        // Manual merging/fallback for older data or single profile
        const allPathsMap = new Map();
        if (profA) profA.hot_paths.forEach(p => allPathsMap.set(p.stack, p));
        if (profB) {
            profB.hot_paths.forEach(p => {
                if (!allPathsMap.has(p.stack)) allPathsMap.set(p.stack, p);
            });
        }
        pathsToShow = Array.from(allPathsMap.values());
        pathsToShow.sort((a, b) => (b.gas || 0) - (a.gas || 0));
    }

    if (pathsToShow) {
        pathsToShow.slice(0, 10).forEach(path => {
            const li = document.createElement('li');
            li.className = 'hot-path-item';
            const name = path.stack.split(';').pop();
            li.id = `path-${name}`;

            let deltaDisplay = '';
            let rightSide = '';

            // If it's a HotPathComparison object from Rust (it has percent_change)
            const isDiffComparison = profB && path.hasOwnProperty('percent_change');

            if (isDiffComparison) {
                const gasDiff = path.gas_change || 0;
                const gasPct = path.percent_change || 0;
                const sign = gasDiff > 0 ? '+' : '';
                const cls = gasDiff > 0 ? 'pos' : (gasDiff < 0 ? 'neg' : 'neutral');
                deltaDisplay = `<span class="delta ${cls}">${sign}${gasPct.toFixed(2)}%</span>`;
                rightSide = ''; 
            } else if (profB) {
                // Manual fallback calculation
                const pathA = profA.hot_paths.find(p => p.stack === path.stack);
                const gasA = pathA ? (pathA.gas || 0) : 0;
                const gasB = path.gas || 0;
                const gasDiff = gasB - gasA;
                const gasPct = gasA === 0 ? (gasB > 0 ? 100 : 0) : (gasDiff / gasA) * 100;
                const sign = gasDiff > 0 ? '+' : '';
                const cls = gasDiff > 0 ? 'pos' : (gasDiff < 0 ? 'neg' : 'neutral');
                deltaDisplay = `<span class="delta ${cls}">${sign}${gasPct.toFixed(2)}%</span>`;
                rightSide = ''; 
            } else {
                deltaDisplay = `<span>[${(path.percentage || 0).toFixed(1)}%]</span>`;
                rightSide = `<span style="opacity: 0.6;">${((path.gas || 0) / 1000).toFixed(0)}k gas</span>`;
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

    // Render category breakdown in sidebar
    renderCategoryStats(profB || profA);
}

/**
 * Aggregate gas by category from a profile's hot_paths and render
 * a compact color-coded bar list in the sidebar.
 */
function renderCategoryStats(profile) {
    const list = document.getElementById('category-stats-list');
    if (!list || !profile || !profile.hot_paths) return;

    // Aggregate gas per category
    const totals = {};
    let grandTotal = 0;
    profile.hot_paths.forEach(path => {
        const cat = path.category || 'Other';
        totals[cat] = (totals[cat] || 0) + path.gas;
        grandTotal += path.gas;
    });

    if (grandTotal === 0) { list.innerHTML = ''; return; }

    // Sort descending by gas
    const sorted = Object.entries(totals).sort((a, b) => b[1] - a[1]);

    list.innerHTML = sorted.map(([cat, gas]) => {
        const pct = (gas / grandTotal * 100).toFixed(1);
        const color = (CONFIG.colors[cat]) || CONFIG.colors.Other;
        return [
            '<li class="cat-stat-item">',
            `  <span class="cat-dot" style="background:${color}"></span>`,
            `  <span class="cat-name">${cat}</span>`,
            '  <div class="cat-bar-track">',
            `    <div class="cat-bar-fill" style="width:${pct}%;background:${color}"></div>`,
            '  </div>',
            `  <span class="cat-pct">${pct}%</span>`,
            '</li>'
        ].join('');
    }).join('');
}
