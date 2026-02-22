const { getCurrentWindow } = window.__TAURI__.window;
const { invoke } = window.__TAURI__.core;

// Global state of current config
let currentConfig = null;
const appWindow = getCurrentWindow();


/**
* Get current config file from RAM
 */
async function getCurrentConfig() {
    let rawData = await invoke("get_current_config");
    currentConfig = JSON.parse(rawData);
}

/**
 * Save all config file
 * */
async function saveConfig() {
    await invoke("save_config", {newConfig: currentConfig});
}

/**
 * Save only config fields, that used by asterisk
 * */
async function saveAsteriskConfig() {
    let server_address = document.getElementById("server-address").value;
    let server_port = document.getElementById("server-port").value;
    let ami_login = document.getElementById("ami-login").value;
    // TODO: Need to additional checks, so that, after saving new password, user cannot see it
    let ami_password = document.getElementById("ami-password").value;

    currentConfig.ami_manager = {
        port: parseInt(server_port, 10),
        host: server_address,
        username: ami_login,
        password: ami_password,
        events: true
    }

    let operator_number = document.getElementById("operator-number").value;
    let operator_context = document.getElementById("operator-context").value;

    console.log("Operator number at save", operator_number);
    console.log("Operator context at save", operator_context);

    currentConfig.ami_operator = {
        operator_number: parseInt(operator_number),
        context: operator_context
    }

    await invoke("save_asterisk_config", {
        amiOperator: currentConfig.ami_operator,
        managerOptions: currentConfig.ami_manager
    })
}

/**
* Handle modal button for showing or hiding add engineer menu
 */
function renderEngineers() {
    const list = document.getElementById('engineers-list');
    list.innerHTML = '';

    currentConfig.engineers.forEach((eng, index) => {
        const div = document.createElement('div');
        div.className = `engineer-item px-6 py-4 flex items-center justify-between cursor-pointer transition-colors ${currentConfig.current_duty_index === index ? 'selected' : 'hover:bg-slate-800/50'}`;
        div.onclick = () => selectEngineer(index);
        div.innerHTML = engineerCard(eng, index);

        list.appendChild(div);
    });

    if (currentConfig.engineers.length === 0) {
        list.innerHTML = '<div class="p-12 text-center text-slate-600 text-sm italic">Список інженерів порожній</div>';
    }
}

/**
* Render all data for Asterisk set up
* */
function renderAsteriskSettings() {
    let server_address = document.getElementById("server-address");
    let server_port = document.getElementById("server-port");
    let ami_login = document.getElementById("ami-login");
    // Need to additional checks, so that, after saving new password, user cannot see it
    let ami_password = document.getElementById("ami-password")

    let operator_number = document.getElementById("operator-number");
    let operator_context = document.getElementById("operator-context");

    // Populate input fields values from loaded config
    if (currentConfig.ami_manager) {
        server_address.value = currentConfig.ami_manager.host;
        server_port.value = currentConfig.ami_manager.port;
        ami_login.value = currentConfig.ami_manager.username;
        // TODO: Add checks for value and disable view button and rewrite when hower
        ami_password.value = currentConfig.ami_manager.password;
    }

    if (currentConfig.ami_operator) {
        operator_number.value = currentConfig.ami_operator.operator_number;
        operator_context.value = currentConfig.ami_operator.context;
    }
}

/**
* Function to toggle modals in UI
* */
function toggleModal(modal, show) {
    if (show) {
        modal.classList.remove('hidden');
    } else {
        modal.classList.add('hidden');
    }
}

/**
* Function for updating current selected user
* */
function selectEngineer(id) {
    currentConfig.current_duty_index = id;
    saveConfig();
    renderEngineers();
}

/**
* Engineer card function
* */
const engineerCard = (eng, index) => `
    <div class="flex items-center">
        <div class="w-10 h-10 rounded-full bg-slate-800 border border-slate-700 flex items-center justify-center text-slate-400 font-bold mr-4">
            ${eng.first_name[0]}${eng.last_name[0]}
        </div>
        <div>
            <div class="text-sm font-semibold text-slate-200">${eng.first_name} ${eng.last_name}</div>
            <div class="text-xs text-slate-500">${eng.phone_number}</div>
        </div>
    </div>
    <div class="flex space-x-1" data-index="${index}">
        <button data-method="edit" class="p-2 text-slate-600 hover:text-blue-500 transition-colors">
            <i class="fa-solid fa-gear"></i>
        </button>
        <button data-method="delete" class="p-2 text-slate-600 hover:text-red-500 transition-colors">
            <i class="fas fa-trash-can"></i>
        </button>
    </div>
`;

/**
* Function for changing a current menu in main window
* */
function showView(viewName) {
    document.querySelectorAll('.sidebar-item').forEach(item => {
        item.classList.remove('active', 'text-white');
        item.classList.add('text-slate-400', 'hover:bg-slate-800');
    });

    const activeItem = document.getElementById(`menu-${viewName}`);
    activeItem.classList.add('active');
    activeItem.classList.remove('text-slate-400', 'hover:bg-slate-800');

    document.querySelectorAll('.view-content').forEach(view => {
        view.classList.remove('active');
    });

    document.getElementById(`view-${viewName}`).classList.add('active');
}

/**
* Function for creating a new engineer to list with updating current engineer
* */
async function addEngineer() {
    const first_name = document.getElementById('new-first-name').value;
    const last_name = document.getElementById('new-last-name').value;
    const phone_number = document.getElementById('new-phone').value;

    if (!first_name || !last_name || !phone_number) {
        alert('Заповність усі поля');
        return;
    }

    const newEng = {
        first_name,
        last_name,
        phone_number
    };

    currentConfig.engineers.push(newEng);
    currentConfig.current_duty_index = currentConfig.engineers.length - 1;

    // Reset and close
    document.getElementById('new-first-name').value = '';
    document.getElementById('new-last-name').value = '';
    document.getElementById('new-phone').value = '';
    const modal = document.getElementById('add-modal');
    toggleModal(modal, false);
    saveConfig();
    renderEngineers();
}

/**
 * Function, that used for editing engineers data
 * */
async function openEditPanel(index) {
    const modal = document.getElementById('edit-modal');
    toggleModal(modal, true);
}

/**
* Delete engineer from the config
 */
async function confirmDelete(index) {
    const totalEngineers = currentConfig.engineers.length;
    if (totalEngineers === 1) {
        alert("В програмі не може бути менше за 1 інженера");
    }
    if (index === currentConfig.current_duty_index && index === 0) {
        currentConfig.engineers.splice(index, 1);
    } else if (index === currentConfig.current_duty_index) {
        currentConfig.engineers.splice(index, 1);
        currentConfig.current_duty_index -= 1;
    } else {
        currentConfig.engineers.splice(index, 1);
    }
    await saveConfig();
    renderEngineers();
}

/**
 * Formatting a string with mask +38(0XX) XXX-XX-XX
 * @param {string} value
 */
function formatPhoneNumber(value) {
    if (!value) return value;

    // Leave only numbers
    const phoneNumber = value.replace(/[^\d]/g, '');
    // If used delete ll, don't return an operator code
    if (phoneNumber.length === 0) return "";

    // If number starts with 38, cut in for convenience
    const core = phoneNumber.startsWith('38') ? phoneNumber.slice(2) : phoneNumber;

    // ormatting parts
    const part1 = core.slice(0, 3); // Operator code
    const part2 = core.slice(3, 6); // first part of number
    const part3 = core.slice(6, 8); // second part
    const part4 = core.slice(8, 10); // tail

    if (core.length > 8) return `+38(${part1})-${part2}-${part3}-${part4}`;
    if (core.length > 6) return `+38(${part1})-${part2}-${part3}`;
    if (core.length > 3) return `+38(${part1})-${part2}`;
    return `+38(${part1}`;
}

// Call one more time when initializing
function initPhoneFields() {
    const phoneInputs = document.querySelectorAll('#new-phone, #edit-phone');

    phoneInputs.forEach(input => {
        input.addEventListener('input', (_) => {
            let selectionStart = input.selectionStart;
            const oldLength = input.value.length;

            input.value = formatPhoneNumber(input.value);

            const newLength = input.value.length;
            const diff = newLength - oldLength;

            selectionStart += diff;

            input.setSelectionRange(selectionStart, selectionStart);
        });

        // If the field is empty, we immediately insert the beginning upon focus
        input.addEventListener('focus', () => {
            if (input.value.length < 4) {
                input.value = '+38(';
            }
        });
    });
}

// Starting point of code executing
document.addEventListener("DOMContentLoaded", async () => {
    // Retrieve current config state of app
    await getCurrentConfig();
    console.log(currentConfig);

    // Add closing apportunity for window
    document
        .getElementById('closeButton')
        ?.addEventListener('click', () => {
            console.log("Close button clicked");
            appWindow.close();
        });

    // Add minimize button handler
    document
        .getElementById('minimizeButton')
        ?.addEventListener('click', () => {
            appWindow.minimize();
        });

    // Add maximize button
    document
        .getElementById('toggleMaximizeButton')
        ?.addEventListener('click', () => {
           appWindow.toggleMaximize();
        });

    // Handle modal tag
    document.addEventListener('click', async (e) => {

        // Handle sidepanel
        if (e.target.closest('[data-view-engineers]')) {
            showView('engineers');
        } else if (e.target.closest('[data-view-telephony]')) {
            showView('telephony')
        }

        if (e.target.closest('#addEngineerButton')) {
            await addEngineer();
        }

        // If pressed open button
        const openId = e.target.closest('[data-modal-toggle]')?.dataset.modalToggle;
        if (openId) {
            const modal = document.getElementById(openId);
            toggleModal(modal, true);
        }

        // If pressed close button (inside modal)
        if (e.target.closest('[data-modal-close]')) {
            e.target.closest('.fixed')?.classList.add('hidden');
        }
    });

    document.getElementById('save-config').addEventListener("click", (e) => {
        e.preventDefault();

        saveAsteriskConfig();
    })

    initPhoneFields();

    // These methods actually fill up HTML documents
    renderEngineers();
    renderAsteriskSettings();

    // Add once a listener for list objects
    document.getElementById('engineers-list').addEventListener("click", (e) => {
        const btn = e.target.closest('button[data-method]');
        if (!btn) {
            const row = e.target.closest('.engineer-item');
            if (row) {
                const index = parseInt(row.querySelector('[data-index]').dataset.index);
                selectEngineer(index);
            }
            return;
        }

        e.stopPropagation()

        const method = btn.dataset.method;
        const parentContainer = btn.closest('[data-index]');
        const index = parseInt(parentContainer.dataset.index);

        if (method === "edit") {
            openEditPanel(index);
        } else if (method === "delete") {
            confirmDelete(index);
        }
    });
});

