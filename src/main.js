const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// Global state
let currentEngineer = null;

/**
 * Change window size via Tauri
 */
async function resizeWindow(width, height) {
    try {
        await invoke("resize_window", { width, height });
    } catch (e) {
        console.error("Failed to resize window:", e);
    }
}

/**
 * Update UI of engineers (only if fields are existing)
 */
function updateEngineerUI(engineer) {
    if (!engineer) return;
    currentEngineer = engineer;

    const elements = {
        'short-user-name': `DEW: ${engineer.phone_number}`,
        'eng-name': `${engineer.first_name} ${engineer.last_name}`,
        'phone-val': engineer.phone_number
    };

    for (const [id, value] of Object.entries(elements)) {
        const el = document.getElementById(id);
        if (el) el.innerText = value;
    }
}


/**
 * Template of uncollpsed widget
 */
const getFullTemplate = (eng) => `
    <div class="widget-card overflow-hidden">
        <div class="bg-slate-800 p-3 flex justify-between items-center drag-handle" data-tauri-drag-region>
            <div class="flex items-center" data-tauri-drag-region>
                <span class="status-dot" data-tauri-drag-region></span>
                <span class="text-white text-xs font-bold uppercase tracking-wider" data-tauri-drag-region>Черговий Інженер</span>
            </div>
            <button id="minimizeButton" class="text-gray-400 hover:text-white transition-colors">
                <i class="fas fa-minus text-xs"></i>
            </button>
        </div>
        <div class="p-4">
            <div class="flex items-center gap-4 mb-4">
                <div class="h-12 w-12 rounded-full bg-blue-100 flex items-center justify-center text-blue-600 text-xl font-bold border border-blue-200">
                    <i class="fas fa-user-gear"></i>
                </div>
                <div>
                    <h2 id="eng-name" class="font-bold text-gray-800 text-lg leading-tight">${eng.first_name} ${eng.last_name}</h2>
                    <p class="text-xs text-gray-500 font-medium">Інженер пульту</p>
                </div>
            </div>
            <div class="space-y-2">
                <div class="flex items-center justify-between bg-gray-50 p-2 rounded-lg border border-gray-100 group">
                    <div class="flex items-center gap-3">
                        <i class="fas fa-phone text-blue-500 text-sm"></i>
                        <span id="phone-val" class="text-sm font-mono text-gray-700">${eng.phone_number}</span>
                    </div>
                    <button id="phone-call" class="opacity-0 group-hover:opacity-100 transition-opacity text-gray-400 hover:text-blue-600">
                        <i class="fas fa-phone"></i>
                    </button>
                </div>
            </div>
            <div class="mt-4 pt-3 border-t border-gray-100 flex justify-between items-center">
                <div class="text-[10px] text-gray-400">
                    Рекомендуємо дзвонити до: <span class="font-bold text-gray-600">23:00 (Київ)</span>
                </div>
            </div>
        </div>
    </div>`;


/**
 * Template of collapsed widget
 */
const getCollapsedTemplate = (eng) => `
    <div class="bg-slate-800 p-2 flex justify-between items-center drag-handle rounded-lg" data-tauri-drag-region>
        <div class="flex items-center" data-tauri-drag-region>
            <span class="status-dot" data-tauri-drag-region></span>
            <span id="short-user-name" class="text-white text-[10px] font-bold" data-tauri-drag-region>DEW: ${eng.phone_number}</span>
        </div>
        <button id="minimizeButton" class="text-gray-400 hover:text-white ml-4">
            <i class="fas fa-expand-alt text-[10px]"></i>
        </button>
    </div>`;


/**
 * Switch widget state
 */
async function toggleWidget() {
    const widget = document.getElementById('widget');
    const isCollapsed = widget.dataset.collapsed === 'true';

    if (isCollapsed) {
        widget.innerHTML = getFullTemplate(currentEngineer);
        widget.dataset.collapsed = 'false';
        widget.style.width = '320px';
        await resizeWindow(320, 233);
    } else {
        widget.innerHTML = getCollapsedTemplate(currentEngineer);
        widget.dataset.collapsed = 'true';
        widget.style.width = '180px';
        await resizeWindow(180, 42);
    }
}

/**
 * Initialization
 */
async function init() {
    // Single click listener (Event delegation)
    document.addEventListener("click", async (e) => {
        // Handle minimize button
        if (e.target.closest('#minimizeButton')) {
            await toggleWidget();
        }
        // Handle phone call button
        if (e.target.closest('#phone-call')) {
            e.preventDefault();
            await invoke("call_engineer");
        }
    });

    // Data retrieving
    try {
        const rawData = await invoke("retrieve_current_engineer");
        const engineer = JSON.parse(rawData);
        
        updateEngineerUI(engineer);
    } catch (err) {
        console.error("Initialization error:", err);
    }

    // Subscribe on backend event
    await listen("duty-changed", (event) => {
        console.log("Duty changed:", event.payload);
        const engineer = JSON.parse(event.payload);
        updateEngineerUI(engineer);
        
        // We update the current view (to avoid redrawing everything, we just change the text)
        const isCollapsed = document.getElementById('widget').dataset.collapsed === 'true';
        if (isCollapsed) {
            const el = document.getElementById('short-user-name');
            if (el) el.innerText = `DEW: ${engineer.phone_number}`;
        } else {
            updateEngineerUI(engineer);
        }
    });
}

window.addEventListener("DOMContentLoaded", init);