let currentFocusedElement;

export function deleteBlock(el) {
  el.remove();
}

export function setAsFocused(el) {
  if (currentFocusedElement) {
    currentFocusedElement.blur();
  }
  currentFocusedElement = el;
  el.focus();
  el.selectionStart = el.textContent?.length;
  updateInputHeight(el);
}

export function updateInputHeight(el) {
  // Let's handle text overflows
  const scrollHeight = el.scrollHeight;
  const clientHeight = el.clientHeight;
  const scrollTop = el.scrollTop;
  const heightDiff = scrollHeight - scrollTop - clientHeight;
  if (heightDiff > 0 || scrollHeight > clientHeight) {
    el.style.height = `${scrollHeight}px`;
  }
}
