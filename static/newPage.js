function changePageTitle(title) {
  const pageTitle = document.querySelector(".title");
  pageTitle.innerText = title;
}

const titleInput = document.getElementById("title-edit");

titleInput.onchange = function(e) {
  changePageTitle(e.target.value);
}
