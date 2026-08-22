function showToast(message) {
  const toast = document.getElementById('toast');
  if (!toast) return;
  toast.textContent = message;
  toast.classList.add('visible');
  setTimeout(function () {
    toast.classList.remove('visible');
  }, 4000);
}

function initToast() {
  document.body.addEventListener('showToast', function (evt) {
    showToast(evt.detail.message);
  });

  document.body.addEventListener('htmx:responseError', function (evt) {
    showToast('Request failed: ' + evt.detail.error);
  });
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initToast);
} else {
  initToast();
}
