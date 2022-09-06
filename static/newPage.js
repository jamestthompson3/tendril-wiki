function changePageTitle() {
  document.title = this.innerText;
}

const titleInput = document.querySelector(".title");

// titleInput.focus();
// titleInput.select();
titleInput.addEventListener("change", changePageTitle);
