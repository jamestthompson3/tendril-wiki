(function () {
  const linkTo = document.querySelectorAll("#linkto");
  const del = document.querySelectorAll("#delete-note");
  for (const item of linkTo) {
    item.remove();
  }
  for (const item of del) {
    item.remove();
  }
})();
