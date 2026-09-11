// Seul pont de démarrage généré/JS : l'interface et ses interactions sont en Rust.
const status = document.querySelector('#load-status');
try {
  const { default: initialize } = await import('/chatpurp_web.js');
  await initialize();
  status.textContent = '';
  status.dataset.state = 'ready';
} catch {
  status.textContent = 'Échec du chargement WebAssembly. Relancez le build de l’interface.';
  status.dataset.state = 'error';
}
