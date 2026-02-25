const { invoke } = window.__TAURI__.core;

const form = document.getElementById("timer-form");
const actionInput = document.getElementById("action");
const targetTimeInput = document.getElementById("target-time");
const messageWrap = document.getElementById("message-wrap");
const messageInput = document.getElementById("message");
const timersEl = document.getElementById("timers");
const statusEl = document.getElementById("status");
const refreshBtn = document.getElementById("refresh");

const showStatus = (text, isError = false) => {
  statusEl.textContent = text;
  statusEl.style.color = isError ? "#c30e2e" : "#475569";
};

const toggleMessage = () => {
  const isPopup = actionInput.value === "popup";
  messageWrap.style.display = isPopup ? "grid" : "none";
  messageInput.required = isPopup;
};

const fmtDate = (iso) => new Date(iso).toLocaleString();

const fmtRemaining = (iso) => {
  const ms = new Date(iso).getTime() - Date.now();
  if (ms <= 0) {
    return "due now";
  }

  const total = Math.floor(ms / 1000);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  return `${hours}h ${minutes}m ${seconds}s`;
};

const renderTimers = (timers) => {
  timersEl.innerHTML = "";

  if (!timers.length) {
    const empty = document.createElement("li");
    empty.className = "empty";
    empty.textContent = "No active timers.";
    timersEl.appendChild(empty);
    return;
  }

  for (const timer of timers) {
    const item = document.createElement("li");
    item.className = "timer-item";

    const top = document.createElement("div");
    top.className = "timer-top";

    const title = document.createElement("strong");
    title.textContent = timer.action.toUpperCase();

    const cancelBtn = document.createElement("button");
    cancelBtn.className = "danger";
    cancelBtn.textContent = "Cancel";
    cancelBtn.addEventListener("click", async () => {
      try {
        await invoke("cancel_timer", { id: timer.id });
        await loadTimers();
        showStatus("Timer canceled.");
      } catch (err) {
        showStatus(String(err), true);
      }
    });

    top.append(title, cancelBtn);

    const when = document.createElement("div");
    when.className = "timer-meta";
    when.textContent = `Runs at ${fmtDate(timer.targetTime)} (${fmtRemaining(timer.targetTime)})`;

    item.append(top, when);

    if (timer.action === "popup" && timer.message) {
      const msg = document.createElement("div");
      msg.className = "timer-meta";
      msg.textContent = `Message: ${timer.message}`;
      item.append(msg);
    }

    timersEl.append(item);
  }
};

const loadTimers = async () => {
  try {
    const timers = await invoke("list_timers");
    renderTimers(timers);
  } catch (err) {
    showStatus(String(err), true);
  }
};

form.addEventListener("submit", async (event) => {
  event.preventDefault();

  if (!targetTimeInput.value) {
    showStatus("Choose a valid time.", true);
    return;
  }

  const request = {
    action: actionInput.value,
    targetTime: new Date(targetTimeInput.value).toISOString(),
    message: actionInput.value === "popup" ? messageInput.value : null,
  };

  try {
    await invoke("create_timer", { request });
    form.reset();
    toggleMessage();
    showStatus("Timer created.");
    await loadTimers();
  } catch (err) {
    showStatus(String(err), true);
  }
});

refreshBtn.addEventListener("click", loadTimers);
actionInput.addEventListener("change", toggleMessage);

setInterval(loadTimers, 1000);

toggleMessage();
loadTimers();
