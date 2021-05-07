function changePageTitle(title) {
  const pageTitle = document.querySelector(".title");
  pageTitle.innerText = title;
  document.title = title;
}

const titleInput = document.getElementById("title-edit");

titleInput.focus();
titleInput.select();

titleInput.onchange = function (e) {
  changePageTitle(e.target.value);
};
