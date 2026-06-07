(() => {
  const setStatus = (button, message, state) => {
    const statusId = button.dataset.copyStatus || button.dataset.selectStatus;
    const status = statusId ? document.getElementById(statusId) : null;
    if (!status) {
      return;
    }
    status.textContent = message;
    status.dataset.state = state;
  };

  const selectTarget = (target) => {
    target.focus();
    target.select();
    target.setSelectionRange(0, target.value.length);
  };

  const fallbackCopy = (target) => {
    selectTarget(target);
    return document.execCommand("copy");
  };

  const copyTarget = async (button) => {
    const target = document.getElementById(button.dataset.copyTarget);
    if (!target) {
      return;
    }

    const text = target.value || target.textContent || "";
    button.disabled = true;

    try {
      if (navigator.clipboard && window.isSecureContext) {
        await navigator.clipboard.writeText(text);
        setStatus(button, "已复制", "success");
        return;
      }

      if (fallbackCopy(target)) {
        setStatus(button, "已复制", "success");
        return;
      }

      setStatus(button, "浏览器限制，请按 Ctrl+C 手动复制", "error");
    } catch (_) {
      if (fallbackCopy(target)) {
        setStatus(button, "已复制", "success");
      } else {
        setStatus(button, "浏览器限制，请按 Ctrl+C 手动复制", "error");
      }
    } finally {
      button.disabled = false;
    }
  };

  document.addEventListener("click", (event) => {
    const copyButton = event.target.closest("[data-copy-target]");
    if (copyButton) {
      copyTarget(copyButton);
      return;
    }

    const selectButton = event.target.closest("[data-select-target]");
    if (selectButton) {
      const target = document.getElementById(selectButton.dataset.selectTarget);
      if (target) {
        selectTarget(target);
        setStatus(selectButton, "已全选，按 Ctrl+C 可复制", "success");
      }
    }
  });
})();
