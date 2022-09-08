function changePageTitle() {
  document.title = this.value;
}

const title = document.querySelector(".title");

title.click();
const titleInput = document.querySelector('input[type="text"].title');

titleInput.focus();
titleInput.select();
titleInput.addEventListener("blur", changePageTitle);
