/*
 *   This content is licensed according to the W3C Software License at
 *   https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
 */

"use strict";

/**
 * @namespace aria
 */
var aria = aria || {};

/**
 * @description
 *  Values for aria-sort
 */
aria.SortType = {
  ASCENDING: "ascending",
  DESCENDING: "descending",
  NONE: "none",
};

/**
 * @description
 *  DOM Selectors to find the grid components
 */
aria.GridSelector = {
  ROW: 'tr, [role="row"]',
  CELL: 'th, td, [role="gridcell"]',
  SCROLL_ROW: 'tr:not([data-fixed]), [role="row"]',
  SORT_HEADER: "th[aria-sort]",
  TABBABLE: '[tabindex="0"]',
};

/**
 * @description
 *  CSS Class names
 */
aria.CSSClass = {
  HIDDEN: "hidden",
};

/**
 * @class
 * @description
 *  Grid object representing the state and interactions for a grid widget
 *
 *  Assumptions:
 *  All focusable cells initially have tabindex="-1"
 *  Produces a fully filled in mxn grid (with no holes)
 * @param gridNode
 *  The DOM node pointing to the grid
 */
aria.Grid = function (gridNode) {
  this.navigationDisabled = false;
  this.gridNode = gridNode;
  this.paginationEnabled = this.gridNode.hasAttribute("data-per-page");
  this.shouldWrapCols = this.gridNode.hasAttribute("data-wrap-cols");
  this.shouldWrapRows = this.gridNode.hasAttribute("data-wrap-rows");
  this.shouldRestructure = this.gridNode.hasAttribute("data-restructure");
  this.topIndex = 0;

  this.keysIndicator = document.getElementById("arrow-keys-indicator");

  aria.Utils.bindMethods(
    this,
    "checkFocusChange",
    "checkPageChange",
    "checkRestructureGrid",
    "delegateButtonHandler",
    "focusClickedCell",
    "restructureGrid",
    "showKeysIndicator",
    "hideKeysIndicator"
  );
  this.setupFocusGrid();
  this.setFocusPointer(0, 0);

  if (this.paginationEnabled) {
    this.setupPagination();
  } else {
    this.perPage = this.grid.length;
  }

  this.registerEvents();
};

/**
 * @description
 *  Creates a 2D array of the focusable cells in the grid.
 */
aria.Grid.prototype.setupFocusGrid = function () {
  this.grid = [];

  Array.prototype.forEach.call(
    this.gridNode.querySelectorAll(aria.GridSelector.ROW),
    function (row) {
      var rowCells = [];

      Array.prototype.forEach.call(
        row.querySelectorAll(aria.GridSelector.CELL),
        function (cell) {
          var focusableSelector = "[tabindex]";

          if (aria.Utils.matches(cell, focusableSelector)) {
            rowCells.push(cell);
          } else {
            var focusableCell = cell.querySelector(focusableSelector);

            if (focusableCell) {
              rowCells.push(focusableCell);
            }
          }
        }.bind(this)
      );

      if (rowCells.length) {
        this.grid.push(rowCells);
      }
    }.bind(this)
  );

  if (this.paginationEnabled) {
    this.setupIndices();
  }
};

/**
 * @description
 *  If possible, set focus pointer to the cell with the specified coordinates
 * @param row
 *  The index of the cell's row
 * @param col
 *  The index of the cell's column
 * @returns {boolean}
 *  Returns whether or not the focus could be set on the cell.
 */
aria.Grid.prototype.setFocusPointer = function (row, col) {
  if (!this.isValidCell(row, col)) {
    return false;
  }

  if (this.isHidden(row, col)) {
    return false;
  }

  if (!isNaN(this.focusedRow) && !isNaN(this.focusedCol)) {
    this.grid[this.focusedRow][this.focusedCol].setAttribute("tabindex", -1);
  }

  this.grid[row][col].removeEventListener("focus", this.showKeysIndicator);
  this.grid[row][col].removeEventListener("blur", this.hideKeysIndicator);

  // Disable navigation if focused on an input
  this.navigationDisabled = aria.Utils.matches(this.grid[row][col], "input");

  this.grid[row][col].setAttribute("tabindex", 0);
  this.focusedRow = row;
  this.focusedCol = col;

  this.grid[row][col].addEventListener("focus", this.showKeysIndicator);
  this.grid[row][col].addEventListener("blur", this.hideKeysIndicator);

  return true;
};

/**
 * @param row
 *  The index of the cell's row
 * @param col
 *  The index of the cell's column
 * @returns {boolean}
 *  Returns whether or not the coordinates are within the grid's boundaries.
 */
aria.Grid.prototype.isValidCell = function (row, col) {
  return (
    !isNaN(row) &&
    !isNaN(col) &&
    row >= 0 &&
    col >= 0 &&
    this.grid &&
    this.grid.length &&
    row < this.grid.length &&
    col < this.grid[row].length
  );
};

/**
 * @param row
 *  The index of the cell's row
 * @param col
 *  The index of the cell's column
 * @returns {boolean}
 *  Returns whether or not the cell has been hidden.
 */
aria.Grid.prototype.isHidden = function (row, col) {
  var cell = this.gridNode
    .querySelectorAll(aria.GridSelector.ROW)
    [row].querySelectorAll(aria.GridSelector.CELL)[col];
  return aria.Utils.hasClass(cell, aria.CSSClass.HIDDEN);
};

/**
 * @description
 *  Clean up grid events
 */
aria.Grid.prototype.clearEvents = function () {
  this.gridNode.removeEventListener("keydown", this.checkFocusChange);
  this.gridNode.removeEventListener("keydown", this.delegateButtonHandler);
  this.gridNode.removeEventListener("click", this.focusClickedCell);
  this.gridNode.removeEventListener("click", this.delegateButtonHandler);

  if (this.paginationEnabled) {
    this.gridNode.removeEventListener("keydown", this.checkPageChange);
  }

  if (this.shouldRestructure) {
    window.removeEventListener("resize", this.checkRestructureGrid);
  }

  this.grid[this.focusedRow][this.focusedCol].removeEventListener(
    "focus",
    this.showKeysIndicator
  );
  this.grid[this.focusedRow][this.focusedCol].removeEventListener(
    "blur",
    this.hideKeysIndicator
  );
};

/**
 * @description
 *  Register grid events
 */
aria.Grid.prototype.registerEvents = function () {
  this.clearEvents();

  this.gridNode.addEventListener("keydown", this.checkFocusChange);
  this.gridNode.addEventListener("keydown", this.delegateButtonHandler);
  this.gridNode.addEventListener("click", this.focusClickedCell);
  this.gridNode.addEventListener("click", this.delegateButtonHandler);

  if (this.paginationEnabled) {
    this.gridNode.addEventListener("keydown", this.checkPageChange);
  }

  if (this.shouldRestructure) {
    window.addEventListener("resize", this.checkRestructureGrid);
  }
};

/**
 * @description
 *  Focus on the cell in the specified row and column
 * @param row
 *  The index of the cell's row
 * @param col
 *  The index of the cell's column
 */
aria.Grid.prototype.focusCell = function (row, col) {
  if (this.setFocusPointer(row, col)) {
    this.grid[row][col].focus();
  }
};

aria.Grid.prototype.showKeysIndicator = function () {
  if (this.keysIndicator) {
    aria.Utils.removeClass(this.keysIndicator, "hidden");
  }
};

aria.Grid.prototype.hideKeysIndicator = function () {
  if (
    this.keysIndicator &&
    this.grid[this.focusedRow][this.focusedCol].tabIndex === 0
  ) {
    aria.Utils.addClass(this.keysIndicator, "hidden");
  }
};

/**
 * @description
 *  Triggered on keydown. Checks if an arrow key was pressed, and (if possible)
 *  moves focus to the next valid cell in the direction of the arrow key.
 * @param event
 *  Keydown event
 */
aria.Grid.prototype.checkFocusChange = function (event) {
  if (!event || this.navigationDisabled) {
    return;
  }

  this.findFocusedItem(event.target);

  var key = event.which || event.keyCode;
  var rowCaret = this.focusedRow;
  var colCaret = this.focusedCol;
  var nextCell;

  switch (key) {
    case aria.KeyCode.UP:
      nextCell = this.getNextVisibleCell(0, -1);
      rowCaret = nextCell.row;
      colCaret = nextCell.col;
      break;
    case aria.KeyCode.DOWN:
      nextCell = this.getNextVisibleCell(0, 1);
      rowCaret = nextCell.row;
      colCaret = nextCell.col;
      break;
    case aria.KeyCode.LEFT:
      nextCell = this.getNextVisibleCell(-1, 0);
      rowCaret = nextCell.row;
      colCaret = nextCell.col;
      break;
    case aria.KeyCode.RIGHT:
      nextCell = this.getNextVisibleCell(1, 0);
      rowCaret = nextCell.row;
      colCaret = nextCell.col;
      break;
    case aria.KeyCode.HOME:
      if (event.ctrlKey) {
        rowCaret = 0;
      }
      colCaret = 0;
      break;
    case aria.KeyCode.END:
      if (event.ctrlKey) {
        rowCaret = this.grid.length - 1;
      }
      colCaret = this.grid[this.focusedRow].length - 1;
      break;
    default:
      return;
  }

  if (this.paginationEnabled) {
    if (rowCaret < this.topIndex) {
      this.showFromRow(rowCaret, true);
    }

    if (rowCaret >= this.topIndex + this.perPage) {
      this.showFromRow(rowCaret, false);
    }
  }

  this.focusCell(rowCaret, colCaret);
  event.preventDefault();
};

/**
 * @description
 *  Reset focused row and col if it doesn't match focusedRow and focusedCol
 * @param focusedTarget
 *  Element that is currently focused by browser
 */
aria.Grid.prototype.findFocusedItem = function (focusedTarget) {
  var focusedCell = this.grid[this.focusedRow][this.focusedCol];

  if (focusedCell === focusedTarget || focusedCell.contains(focusedTarget)) {
    return;
  }

  for (var i = 0; i < this.grid.length; i++) {
    for (var j = 0; j < this.grid[i].length; j++) {
      if (
        this.grid[i][j] === focusedTarget ||
        this.grid[i][j].contains(focusedTarget)
      ) {
        this.setFocusPointer(i, j);
        return;
      }
    }
  }
};

/**
 * @description
 *  Triggered on click. Finds the cell that was clicked on and focuses on it.
 * @param event
 *  Keydown event
 */
aria.Grid.prototype.focusClickedCell = function (event) {
  var clickedGridCell = this.findClosest(event.target, "[tabindex]");

  for (var row = 0; row < this.grid.length; row++) {
    for (var col = 0; col < this.grid[row].length; col++) {
      if (this.grid[row][col] === clickedGridCell) {
        this.setFocusPointer(row, col);

        if (!aria.Utils.matches(clickedGridCell, "button[aria-haspopup]")) {
          // Don't focus if it's a menu button (focus should be set to menu)
          this.focusCell(row, col);
        }

        return;
      }
    }
  }
};

/**
 * @description
 *  Triggered on click. Checks if user clicked on a header with aria-sort.
 *  If so, it sorts the column based on the aria-sort attribute.
 * @param event
 *  Keydown event
 */
aria.Grid.prototype.delegateButtonHandler = function (event) {
  var key = event.which || event.keyCode;
  var target = event.target;
  var isClickEvent = event.type === "click";

  if (!target) {
    return;
  }

  if (
    target.parentNode &&
    target.parentNode.matches("th[aria-sort]") &&
    (isClickEvent || key === aria.KeyCode.SPACE || key === aria.KeyCode.RETURN)
  ) {
    event.preventDefault();
    this.handleSort(target.parentNode);
  }

  if (
    aria.Utils.matches(target, ".editable-text, .edit-text-button") &&
    (isClickEvent || key === aria.KeyCode.RETURN)
  ) {
    event.preventDefault();
    this.toggleEditMode(this.findClosest(target, ".editable-text"), true, true);
  }

  if (
    aria.Utils.matches(target, ".edit-text-input") &&
    (key === aria.KeyCode.RETURN || key === aria.KeyCode.ESC)
  ) {
    event.preventDefault();
    this.toggleEditMode(
      this.findClosest(target, ".editable-text"),
      false,
      key === aria.KeyCode.RETURN
    );
  }
};

/**
 * @description
 *  Toggles the mode of an editable cell between displaying the edit button
 *  and displaying the editable input.
 * @param editCell
 *  Cell to toggle
 * @param toggleOn
 *  Whether to show or hide edit input
 * @param updateText
 *  Whether or not to update the button text with the input text
 */
aria.Grid.prototype.toggleEditMode = function (editCell, toggleOn, updateText) {
  var onClassName = toggleOn ? "edit-text-input" : "edit-text-button";
  var offClassName = toggleOn ? "edit-text-button" : "edit-text-input";
  var onNode = editCell.querySelector("." + onClassName);
  var offNode = editCell.querySelector("." + offClassName);

  if (toggleOn) {
    onNode.value = offNode.innerText;
  } else if (updateText) {
    onNode.innerText = offNode.value;
  }

  aria.Utils.addClass(offNode, aria.CSSClass.HIDDEN);
  aria.Utils.removeClass(onNode, aria.CSSClass.HIDDEN);
  offNode.setAttribute("tabindex", -1);
  onNode.setAttribute("tabindex", 0);
  onNode.focus();
  this.grid[this.focusedRow][this.focusedCol] = onNode;
  this.navigationDisabled = toggleOn;
};

/**
 * @description
 *  Sorts the column below the header node, based on the aria-sort attribute.
 *  aria-sort="none" => aria-sort="ascending"
 *  aria-sort="ascending" => aria-sort="descending"
 *  All other headers with aria-sort are reset to "none"
 *
 *  Note: This implementation assumes that there is no pagination on the grid.
 * @param headerNode
 *  Header DOM node
 */
aria.Grid.prototype.handleSort = function (headerNode) {
  var columnIndex = headerNode.cellIndex;
  var sortType = headerNode.getAttribute("aria-sort");

  if (sortType === aria.SortType.ASCENDING) {
    sortType = aria.SortType.DESCENDING;
  } else {
    sortType = aria.SortType.ASCENDING;
  }

  var comparator = function (row1, row2) {
    var row1Text = row1.children[columnIndex].innerText;
    var row2Text = row2.children[columnIndex].innerText;
    var row1Value = parseInt(row1Text.replace(/[^0-9.]+/g, ""));
    var row2Value = parseInt(row2Text.replace(/[^0-9.]+/g, ""));

    if (sortType === aria.SortType.ASCENDING) {
      return row1Value - row2Value;
    } else {
      return row2Value - row1Value;
    }
  };

  this.sortRows(comparator);
  this.setupFocusGrid();

  Array.prototype.forEach.call(
    this.gridNode.querySelectorAll(aria.GridSelector.SORT_HEADER),
    function (headerCell) {
      headerCell.setAttribute("aria-sort", aria.SortType.NONE);
    }
  );

  headerNode.setAttribute("aria-sort", sortType);
};

/**
 * @description
 *  Sorts the grid's rows according to the specified compareFn
 * @param compareFn
 *  Comparison function to sort the rows
 */
aria.Grid.prototype.sortRows = function (compareFn) {
  var rows = this.gridNode.querySelectorAll(aria.GridSelector.ROW);
  var rowWrapper = rows[0].parentNode;
  var dataRows = Array.prototype.slice.call(rows, 1);

  dataRows.sort(compareFn);

  dataRows.forEach(
    function (row) {
      rowWrapper.appendChild(row);
    }.bind(this)
  );
};

/**
 * @description
 *  Adds aria-rowindex and aria-colindex to the cells in the grid
 */
aria.Grid.prototype.setupIndices = function () {
  var rows = this.gridNode.querySelectorAll(aria.GridSelector.ROW);

  for (var row = 0; row < rows.length; row++) {
    var cols = rows[row].querySelectorAll(aria.GridSelector.CELL);
    rows[row].setAttribute("aria-rowindex", row + 1);

    for (var col = 0; col < cols.length; col++) {
      cols[col].setAttribute("aria-colindex", col + 1);
    }
  }
};

/**
 * @description
 *  Determines the per page attribute of the grid, and shows/hides rows
 *  accordingly.
 */
aria.Grid.prototype.setupPagination = function () {
  this.onPaginationChange = this.onPaginationChange || function () {};
  this.perPage = parseInt(this.gridNode.getAttribute("data-per-page"));
  this.showFromRow(0, true);
};

aria.Grid.prototype.setPaginationChangeHandler = function (onPaginationChange) {
  this.onPaginationChange = onPaginationChange;
};

/**
 * @description
 *  Check if page up or page down was pressed, and show the next page if so.
 * @param event
 *  Keydown event
 */
aria.Grid.prototype.checkPageChange = function (event) {
  if (!event) {
    return;
  }

  var key = event.which || event.keyCode;

  if (key === aria.KeyCode.PAGE_UP) {
    event.preventDefault();
    this.movePageUp();
  } else if (key === aria.KeyCode.PAGE_DOWN) {
    event.preventDefault();
    this.movePageDown();
  }
};

aria.Grid.prototype.movePageUp = function () {
  var startIndex = Math.max(this.perPage - 1, this.topIndex - 1);
  this.showFromRow(startIndex, false);
  this.focusCell(startIndex, this.focusedCol);
};

aria.Grid.prototype.movePageDown = function () {
  var startIndex = this.topIndex + this.perPage;
  this.showFromRow(startIndex, true);
  this.focusCell(startIndex, this.focusedCol);
};

/**
 * @description
 *  Scroll the specified row into view in the specified direction
 * @param startIndex
 *  Row index to use as the start index
 * @param scrollDown
 *  Whether to scroll the new page above or below the row index
 */
aria.Grid.prototype.showFromRow = function (startIndex, scrollDown) {
  var dataRows = this.gridNode.querySelectorAll(aria.GridSelector.SCROLL_ROW);
  var reachedTop = false;
  var firstIndex = -1;
  var endIndex = -1;

  if (startIndex < 0 || startIndex >= dataRows.length) {
    return;
  }

  for (var i = 0; i < dataRows.length; i++) {
    if (
      (scrollDown && i >= startIndex && i < startIndex + this.perPage) ||
      (!scrollDown && i <= startIndex && i > startIndex - this.perPage)
    ) {
      aria.Utils.removeClass(dataRows[i], aria.CSSClass.HIDDEN);

      if (!reachedTop) {
        this.topIndex = i;
        reachedTop = true;
      }

      if (firstIndex < 0) {
        firstIndex = i;
      }
      endIndex = i;
    } else {
      aria.Utils.addClass(dataRows[i], aria.CSSClass.HIDDEN);
    }
  }
  this.onPaginationChange(firstIndex, endIndex);
};

/**
 * @description
 *  Throttle restructuring to only happen every 300ms
 */
aria.Grid.prototype.checkRestructureGrid = function () {
  if (this.waitingToRestructure) {
    return;
  }

  this.waitingToRestructure = true;

  setTimeout(this.restructureGrid, 300);
};

/**
 * @description
 *  Restructure grid based on the size.
 */
aria.Grid.prototype.restructureGrid = function () {
  this.waitingToRestructure = false;

  var gridWidth = this.gridNode.offsetWidth;
  var cells = this.gridNode.querySelectorAll(aria.GridSelector.CELL);
  var currentWidth = 0;

  var focusedElement = this.gridNode.querySelector(aria.GridSelector.TABBABLE);
  var shouldRefocus = document.activeElement === focusedElement;
  var focusedIndex = this.focusedRow * this.grid[0].length + this.focusedCol;

  var newRow = document.createElement("div");
  newRow.setAttribute("role", "row");
  this.gridNode.innerHTML = "";
  this.gridNode.append(newRow);

  cells.forEach(function (cell) {
    var cellWidth = cell.offsetWidth;

    if (currentWidth > 0 && currentWidth >= gridWidth - cellWidth) {
      newRow = document.createElement("div");
      newRow.setAttribute("role", "row");
      this.gridNode.append(newRow);
      currentWidth = 0;
    }

    newRow.append(cell);
    currentWidth += cellWidth;
  });

  this.setupFocusGrid();

  this.focusedRow = Math.floor(focusedIndex / this.grid[0].length);
  this.focusedCol = focusedIndex % this.grid[0].length;

  if (shouldRefocus) {
    this.focusCell(this.focusedRow, this.focusedCol);
  }
};

/**
 * @description
 * Get next cell to the right or left (direction) of the focused
 * cell.
 * @param currRow
 * Row index to start searching from
 * @param currCol
 * Column index to start searching from
 * @param directionX
 * X direction for where to check for cells. +1 to check to the right, -1 to
 * check to the left
 * @param directionY
 * @returns {false | object}
 * Indices of the next cell in the specified direction. Returns the focused
 * cell if none are found.
 */
aria.Grid.prototype.getNextCell = function (
  currRow,
  currCol,
  directionX,
  directionY
) {
  var row = currRow + directionY;
  var col = currCol + directionX;
  var rowCount = this.grid.length;
  var isLeftRight = directionX !== 0;

  if (!rowCount) {
    return false;
  }

  var colCount = this.grid[0].length;

  if (this.shouldWrapCols && isLeftRight) {
    if (col < 0) {
      col = colCount - 1;
      row--;
    }

    if (col >= colCount) {
      col = 0;
      row++;
    }
  }

  if (this.shouldWrapRows && !isLeftRight) {
    if (row < 0) {
      col--;
      row = rowCount - 1;
      if (this.grid[row] && col >= 0 && !this.grid[row][col]) {
        // Sometimes the bottom row is not completely filled in. In this case,
        // jump to the next filled in cell.
        row--;
      }
    } else if (row >= rowCount || !this.grid[row][col]) {
      row = 0;
      col++;
    }
  }

  if (this.isValidCell(row, col)) {
    return {
      row: row,
      col: col,
    };
  } else if (this.isValidCell(currRow, currCol)) {
    return {
      row: currRow,
      col: currCol,
    };
  } else {
    return false;
  }
};

/**
 * @param directionX
 * @param directionY
 * @description
Get next visible column to the right or left (direction) of the focused
 * cell.
 * @returns {false | object}
 * Indices of the next visible cell in the specified direction. If no visible
 * cells are found, returns false if the current cell is hidden and returns
 * the current cell if it is not hidden.
 */
aria.Grid.prototype.getNextVisibleCell = function (directionX, directionY) {
  var nextCell = this.getNextCell(
    this.focusedRow,
    this.focusedCol,
    directionX,
    directionY
  );

  if (!nextCell) {
    return false;
  }

  while (this.isHidden(nextCell.row, nextCell.col)) {
    var currRow = nextCell.row;
    var currCol = nextCell.col;

    nextCell = this.getNextCell(currRow, currCol, directionX, directionY);

    if (currRow === nextCell.row && currCol === nextCell.col) {
      // There are no more cells to try if getNextCell returns the current cell
      return false;
    }
  }

  return nextCell;
};

/**
 * @description
 *  Show or hide the cells in the specified column
 * @param columnIndex
 *  Index of the column to toggle
 * @param isShown
 *  Whether or not to show the column
 */
aria.Grid.prototype.toggleColumn = function (columnIndex, isShown) {
  var cellSelector = '[aria-colindex="' + columnIndex + '"]';
  var columnCells = this.gridNode.querySelectorAll(cellSelector);

  Array.prototype.forEach.call(columnCells, function (cell) {
    if (isShown) {
      aria.Utils.removeClass(cell, aria.CSSClass.HIDDEN);
    } else {
      aria.Utils.addClass(cell, aria.CSSClass.HIDDEN);
    }
  });

  if (!isShown && this.focusedCol === columnIndex - 1) {
    // If focus was set on the hidden column, shift focus to the right
    var nextCell = this.getNextVisibleCell(1, 0);
    if (nextCell) {
      this.setFocusPointer(nextCell.row, nextCell.col);
    }
  }
};

/**
 * @description
 *  Find the closest element matching the selector. Only checks parent and
 *  direct children.
 * @param element
 *  Element to start searching from
 * @param selector
 *  Index of the column to toggle
 * @returns {object} matching element
 */
aria.Grid.prototype.findClosest = function (element, selector) {
  if (aria.Utils.matches(element, selector)) {
    return element;
  }

  if (aria.Utils.matches(element.parentNode, selector)) {
    return element.parentNode;
  }

  return element.querySelector(selector);
};
/*
 *   This content is licensed according to the W3C Software License at
 *   https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
 */

/* global aria */

("use strict");

/**
 * ARIA Grid Examples
 *
 * @function onload
 * @description Initialize the grid examples once the page has loaded
 */

window.addEventListener("load", function () {
  // Initialize Example 1 Grid (if it is present in the DOM)
  var ex1GridElement = document.getElementById("ex1-grid");
  if (ex1GridElement) {
    new aria.Grid(ex1GridElement);
  }

  // Initialize Example 2 Grid (if it is present in the DOM)
  var ex2GridElement = document.getElementById("ex2-grid");
  if (ex2GridElement) {
    new aria.Grid(ex2GridElement);
  }

  // Initialize Example 3 Grid (if it is present in the DOM)
  var ex3GridElement = document.getElementById("ex3-grid");
  if (ex3GridElement) {
    var ex3Grid = new aria.Grid(ex3GridElement);
    var toggleButton = document.getElementById("toggle_column_btn");
    var toggledOn = true;

    toggleButton.addEventListener("click", function () {
      toggledOn = !toggledOn;

      ex3Grid.toggleColumn(2, toggledOn);
      ex3Grid.toggleColumn(4, toggledOn);

      if (toggledOn) {
        toggleButton.innerText = "Hide Type and Category";
      } else {
        toggleButton.innerText = "Show Type and Category";
      }
    });
  }
});
/*
 *   This content is licensed according to the W3C Software License at
 *   https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
 *
 */

("use strict");

/**
 * ARIA Menu Button example
 *
 * @function onload
 * @description  after page has loaded initialize all menu buttons based on the selector "[aria-haspopup][aria-controls]"
 */

window.addEventListener("load", function () {
  var menuButtons = document.querySelectorAll("[aria-haspopup][aria-controls]");

  [].forEach.call(menuButtons, function (menuButton) {
    if (
      (menuButton && menuButton.tagName.toLowerCase() === "button") ||
      menuButton.getAttribute("role").toLowerCase() === "button"
    ) {
      var mb = new aria.widget.MenuButton(menuButton);
      mb.initMenuButton();
    }
  });
});

/**
 * @namespace aria
 */

var aria = aria || {};

/* ---------------------------------------------------------------- */
/*                  ARIA Utils Namespace                        */
/* ---------------------------------------------------------------- */

/**
 * @class Menu
 * @memberof aria.Utils
 * @description  Computes absolute position of an element
 */

aria.Utils = aria.Utils || {};

aria.Utils.findPos = function (element) {
  var xPosition = 0;
  var yPosition = 0;

  while (element) {
    xPosition += element.offsetLeft - element.scrollLeft + element.clientLeft;
    yPosition += element.offsetTop - element.scrollTop + element.clientTop;
    element = element.offsetParent;
  }
  return { x: xPosition, y: yPosition };
};

/* ---------------------------------------------------------------- */
/*                  ARIA Widget Namespace                        */
/* ---------------------------------------------------------------- */

aria.widget = aria.widget || {};

/* ---------------------------------------------------------------- */
/*                  Menu Button Widget                           */
/* ---------------------------------------------------------------- */

/**
 * @class Menu
 * @memberof aria.Widget
 * @description  Creates a Menu Button widget using ARIA
 */

aria.widget.Menu = function (node, menuButton) {
  this.keyCode = Object.freeze({
    TAB: 9,
    RETURN: 13,
    ESC: 27,
    SPACE: 32,
    PAGEUP: 33,
    PAGEDOWN: 34,
    END: 35,
    HOME: 36,
    LEFT: 37,
    UP: 38,
    RIGHT: 39,
    DOWN: 40,
  });

  // Check fo DOM element node
  if (typeof node !== "object" || !node.getElementsByClassName) {
    return false;
  }

  this.menuNode = node;
  node.tabIndex = -1;

  this.menuButton = menuButton;

  this.firstMenuItem = false;
  this.lastMenuItem = false;
};

/**
 * @function initMenuButton
 * @memberof aria.widget.Menu
 * @description  Adds event handlers to button elements
 */

aria.widget.Menu.prototype.initMenu = function () {
  var self = this;

  var cn = this.menuNode.firstChild;

  while (cn) {
    if (cn.nodeType === Node.ELEMENT_NODE) {
      if (cn.getAttribute("role") === "menuitem") {
        cn.tabIndex = -1;
        if (!this.firstMenuItem) {
          this.firstMenuItem = cn;
        }
        this.lastMenuItem = cn;

        var eventKeyDown = function (event) {
          self.eventKeyDown(event, self);
        };
        cn.addEventListener("keydown", eventKeyDown);

        var eventMouseClick = function (event) {
          self.eventMouseClick(event, self);
        };
        cn.addEventListener("click", eventMouseClick);

        var eventBlur = function (event) {
          self.eventBlur(event, self);
        };
        cn.addEventListener("blur", eventBlur);

        var eventFocus = function (event) {
          self.eventFocus(event, self);
        };
        cn.addEventListener("focus", eventFocus);
      }
    }
    cn = cn.nextSibling;
  }
};

/**
 * @function nextMenuItem
 * @memberof aria.widget.Menu
 * @description  Moves focus to next menuItem
 */

aria.widget.Menu.prototype.nextMenuItem = function (currentMenuItem) {
  var mi = currentMenuItem.nextSibling;

  while (mi) {
    if (
      mi.nodeType === Node.ELEMENT_NODE &&
      mi.getAttribute("role") === "menuitem"
    ) {
      mi.focus();
      break;
    }
    mi = mi.nextSibling;
  }

  if (!mi && this.firstMenuItem) {
    this.firstMenuItem.focus();
  }
};

/**
 * @function previousMenuItem
 * @memberof aria.widget.Menu
 * @description  Moves focus to previous menuItem
 */

aria.widget.Menu.prototype.previousMenuItem = function (currentMenuItem) {
  var mi = currentMenuItem.previousSibling;

  while (mi) {
    if (
      mi.nodeType === Node.ELEMENT_NODE &&
      mi.getAttribute("role") === "menuitem"
    ) {
      mi.focus();
      break;
    }
    mi = mi.previousSibling;
  }

  if (!mi && this.lastMenuItem) {
    this.lastMenuItem.focus();
  }
};

/**
 * @function eventKeyDown
 * @memberof aria.widget.Menu
 * @description  Keydown event handler for Menu Object
 *        NOTE: The menu parameter is needed to provide a reference to the specific
 *               menu
 */

aria.widget.Menu.prototype.eventKeyDown = function (event, menu) {
  var ct = event.currentTarget;
  var flag = false;

  switch (event.keyCode) {
    case menu.keyCode.SPACE:
    case menu.keyCode.RETURN:
      menu.eventMouseClick(event, menu);
      menu.menuButton.closeMenu(true);
      flag = true;
      break;

    case menu.keyCode.ESC:
      menu.menuButton.closeMenu(true);
      menu.menuButton.buttonNode.focus();
      flag = true;
      break;

    case menu.keyCode.UP:
    case menu.keyCode.LEFT:
      menu.previousMenuItem(ct);
      flag = true;
      break;

    case menu.keyCode.DOWN:
    case menu.keyCode.RIGHT:
      menu.nextMenuItem(ct);
      flag = true;
      break;

    case menu.keyCode.TAB:
      menu.menuButton.closeMenu(true, false);
      break;

    default:
      break;
  }

  if (flag) {
    event.stopPropagation();
    event.preventDefault();
  }
};

/**
 * @function eventMouseClick
 * @memberof aria.widget.Menu
 * @description  onclick event handler for Menu Object
 *        NOTE: The menu parameter is needed to provide a reference to the specific
 *               menu
 */

aria.widget.Menu.prototype.eventMouseClick = function (event, menu) {
  var clickedItemText = event.target.innerText;
  this.menuButton.buttonNode.innerText = clickedItemText;
  menu.menuButton.closeMenu(true);
};

/**
 * @param event
 * @param menu
 * @function eventBlur
 * @memberof aria.widget.Menu
 * @description eventBlur event handler for Menu Object
 * NOTE: The menu parameter is needed to provide a reference to the specific
 * menu
 */
aria.widget.Menu.prototype.eventBlur = function (event, menu) {
  menu.menuHasFocus = false;
  setTimeout(function () {
    if (!menu.menuHasFocus) {
      menu.menuButton.closeMenu(false, false);
    }
  }, 200);
};

/**
 * @param event
 * @param menu
 * @function eventFocus
 * @memberof aria.widget.Menu
 * @description eventFocus event handler for Menu Object
 * NOTE: The menu parameter is needed to provide a reference to the specific
 * menu
 */
aria.widget.Menu.prototype.eventFocus = function (event, menu) {
  menu.menuHasFocus = true;
};

/* ---------------------------------------------------------------- */
/*                  Menu Button Widget                           */
/* ---------------------------------------------------------------- */

/**
 * @class Menu Button
 * @memberof aria.Widget
 * @description  Creates a Menu Button widget using ARIA
 */

aria.widget.MenuButton = function (node) {
  this.keyCode = Object.freeze({
    TAB: 9,
    RETURN: 13,
    ESC: 27,
    SPACE: 32,
    UP: 38,
    DOWN: 40,
  });

  // Check fo DOM element node
  if (typeof node !== "object" || !node.getElementsByClassName) {
    return false;
  }

  this.done = true;
  this.mouseInMouseButton = false;
  this.menuHasFocus = false;
  this.buttonNode = node;
  this.isLink = false;

  if (node.tagName.toLowerCase() === "a") {
    var url = node.getAttribute("href");
    if (url && url.length && url.length > 0) {
      this.isLink = true;
    }
  }
};

/**
 * @function initMenuButton
 * @memberof aria.widget.MenuButton
 * @description  Adds event handlers to button elements
 */

aria.widget.MenuButton.prototype.initMenuButton = function () {
  var id = this.buttonNode.getAttribute("aria-controls");

  if (id) {
    this.menuNode = document.getElementById(id);

    if (this.menuNode) {
      this.menu = new aria.widget.Menu(this.menuNode, this);
      this.menu.initMenu();
      this.menuShowing = false;
    }
  }

  this.closeMenu(false, false);

  var self = this;

  var eventKeyDown = function (event) {
    self.eventKeyDown(event, self);
  };
  this.buttonNode.addEventListener("keydown", eventKeyDown);

  var eventMouseClick = function (event) {
    self.eventMouseClick(event, self);
  };
  this.buttonNode.addEventListener("click", eventMouseClick);
};

/**
 * @function openMenu
 * @memberof aria.widget.MenuButton
 * @description  Opens the menu
 */

aria.widget.MenuButton.prototype.openMenu = function () {
  if (this.menuNode) {
    this.menuNode.style.display = "block";
    this.menuShowing = true;
  }
};

/**
 * @function closeMenu
 * @memberof aria.widget.MenuButton
 * @description  Close the menu
 */

aria.widget.MenuButton.prototype.closeMenu = function (force, focusMenuButton) {
  if (typeof force !== "boolean") {
    force = false;
  }
  if (typeof focusMenuButton !== "boolean") {
    focusMenuButton = true;
  }

  if (
    force ||
    (!this.mouseInMenuButton &&
      this.menuNode &&
      !this.menu.mouseInMenu &&
      !this.menu.menuHasFocus)
  ) {
    this.menuNode.style.display = "none";
    if (focusMenuButton) {
      this.buttonNode.focus();
    }
    this.menuShowing = false;
  }
};

/**
 * @function toggleMenu
 * @memberof aria.widget.MenuButton
 * @description  Close or open the menu depending on current state
 */

aria.widget.MenuButton.prototype.toggleMenu = function () {
  if (this.menuNode) {
    if (this.menuNode.style.display === "block") {
      this.menuNode.style.display = "none";
    } else {
      this.menuNode.style.display = "block";
    }
  }
};

/**
 * @function moveFocusToFirstMenuItem
 * @memberof aria.widget.MenuButton
 * @description  Move keyboard focus to first menu item
 */

aria.widget.MenuButton.prototype.moveFocusToFirstMenuItem = function () {
  if (this.menu.firstMenuItem) {
    this.openMenu();
    this.menu.firstMenuItem.focus();
  }
};

/**
 * @function moveFocusToLastMenuItem
 * @memberof aria.widget.MenuButton
 * @description  Move keyboard focus to last menu item
 */

aria.widget.MenuButton.prototype.moveFocusToLastMenuItem = function () {
  if (this.menu.lastMenuItem) {
    this.openMenu();
    this.menu.lastMenuItem.focus();
  }
};

/**
 * @function eventKeyDown
 * @memberof aria.widget.MenuButton
 * @description  Keydown event handler for MenuButton Object
 *        NOTE: The menuButton parameter is needed to provide a reference to the specific
 *               menuButton
 */

aria.widget.MenuButton.prototype.eventKeyDown = function (event, menuButton) {
  var flag = false;

  switch (event.keyCode) {
    case menuButton.keyCode.SPACE:
      menuButton.moveFocusToFirstMenuItem();
      flag = true;
      break;

    case menuButton.keyCode.RETURN:
      menuButton.moveFocusToFirstMenuItem();
      flag = true;
      break;

    case menuButton.keyCode.UP:
      if (this.menuShowing) {
        menuButton.moveFocusToLastMenuItem();
        flag = true;
      }
      break;

    case menuButton.keyCode.DOWN:
      if (this.menuShowing) {
        menuButton.moveFocusToFirstMenuItem();
        flag = true;
      }
      break;

    case menuButton.keyCode.TAB:
      menuButton.closeMenu(true, false);
      break;

    default:
      break;
  }

  if (flag) {
    event.stopPropagation();
    event.preventDefault();
  }
};

/**
 * @param event
 * @param menuButton
 * @function eventMouseClick
 * @memberof aria.widget.MenuButton
 * @description Click event handler for MenuButton Object
 * NOTE: The menuButton parameter is needed to provide a reference to the specific
 * menuButton
 */
aria.widget.MenuButton.prototype.eventMouseClick = function (
  event,
  menuButton
) {
  menuButton.moveFocusToFirstMenuItem();
};
("use strict");
/**
 * @namespace aria
 */

var aria = aria || {};

/**
 * @description
 *  Key code constants
 */
aria.KeyCode = {
  BACKSPACE: 8,
  TAB: 9,
  RETURN: 13,
  SHIFT: 16,
  ESC: 27,
  SPACE: 32,
  PAGE_UP: 33,
  PAGE_DOWN: 34,
  END: 35,
  HOME: 36,
  LEFT: 37,
  UP: 38,
  RIGHT: 39,
  DOWN: 40,
  DELETE: 46,
};

aria.Utils = aria.Utils || {};

// Polyfill src https://developer.mozilla.org/en-US/docs/Web/API/Element/matches
aria.Utils.matches = function (element, selector) {
  if (!Element.prototype.matches) {
    Element.prototype.matches =
      Element.prototype.matchesSelector ||
      Element.prototype.mozMatchesSelector ||
      Element.prototype.msMatchesSelector ||
      Element.prototype.oMatchesSelector ||
      Element.prototype.webkitMatchesSelector ||
      function (s) {
        var matches = element.parentNode.querySelectorAll(s);
        var i = matches.length;
        while (--i >= 0 && matches.item(i) !== this) {
          // empty
        }
        return i > -1;
      };
  }

  return element.matches(selector);
};

aria.Utils.remove = function (item) {
  if (item.remove && typeof item.remove === "function") {
    return item.remove();
  }
  if (
    item.parentNode &&
    item.parentNode.removeChild &&
    typeof item.parentNode.removeChild === "function"
  ) {
    return item.parentNode.removeChild(item);
  }
  return false;
};

aria.Utils.isFocusable = function (element) {
  if (element.tabIndex < 0) {
    return false;
  }

  if (element.disabled) {
    return false;
  }

  switch (element.nodeName) {
    case "A":
      return !!element.href && element.rel != "ignore";
    case "INPUT":
      return element.type != "hidden";
    case "BUTTON":
    case "SELECT":
    case "TEXTAREA":
      return true;
    default:
      return false;
  }
};

aria.Utils.getAncestorBySelector = function (element, selector) {
  if (!aria.Utils.matches(element, selector + " " + element.tagName)) {
    // Element is not inside an element that matches selector
    return null;
  }

  // Move up the DOM tree until a parent matching the selector is found
  var currentNode = element;
  var ancestor = null;
  while (ancestor === null) {
    if (aria.Utils.matches(currentNode.parentNode, selector)) {
      ancestor = currentNode.parentNode;
    } else {
      currentNode = currentNode.parentNode;
    }
  }

  return ancestor;
};

aria.Utils.hasClass = function (element, className) {
  return new RegExp("(\\s|^)" + className + "(\\s|$)").test(element.className);
};

aria.Utils.addClass = function (element, className) {
  if (!aria.Utils.hasClass(element, className)) {
    element.className += " " + className;
  }
};

aria.Utils.removeClass = function (element, className) {
  var classRegex = new RegExp("(\\s|^)" + className + "(\\s|$)");
  element.className = element.className.replace(classRegex, " ").trim();
};

aria.Utils.bindMethods = function (object /* , ...methodNames */) {
  var methodNames = Array.prototype.slice.call(arguments, 1);
  methodNames.forEach(function (method) {
    object[method] = object[method].bind(object);
  });
};
