function moveCaretToEnd(el) {
  if (typeof el.selectionStart == "number") {
    el.selectionStart = el.selectionEnd = el.value.length;
  } else if (typeof el.createTextRange != "undefined") {
    el.focus();
    var range = el.createTextRange();
    range.collapse(false);
    range.select();
  }
}

function moveCaretToStart(el) {
  if (typeof el.selectionStart == "number") {
    el.selectionStart = el.selectionEnd = 0;
  } else if (typeof el.createTextRange != "undefined") {
    el.focus();
    var range = el.createTextRange();
    range.collapse(false);
    range.select();
  }
}

function edit() {
  const editElement = document.getElementById("edit");
  if (editElement) {
    // sometimes the page might not be editable
    editElement.checked = true;
  }
}

const textarea = document.getElementById("body");
if (textarea) {
  textarea.onkeydown = function (e) {
    switch (e.key) {
      case "Home":
        moveCaretToStart(textarea);
        break;
      case "End":
        moveCaretToEnd(textarea);
        break;
      default:
        break;
    }
  };
}
document.onkeydown = function (e) {
  console.log(e.ctrlKey);
  switch (e.key) {
    case "e":
      edit();
      break;
    default:
      break;
  }
};
