(function () {
  const SORT_DIR = {
    ASC: "ascending",
    DESC: "descending",
  };

  // Event Listeners
  // =======================================================================================
  const headerRowSelector = "thead > tr >";
  const bodyRowSelector = "tbody tr >";
  const statusHeader = document.querySelector(
    `${headerRowSelector} th:first-child`
  );
  statusHeader.addEventListener("click", sortBy(status));
  const prioHeader = document.querySelector(
    `${headerRowSelector} th:nth-child(2)`
  );
  prioHeader.addEventListener("click", sortBy(priority));
  const statusCells = document.querySelectorAll(
    `${bodyRowSelector} td:first-child`
  );
  for (const statusCell of statusCells) {
    statusCell.addEventListener("click", updateCellStatus);
  }
  const priorityCells = document.querySelectorAll(
    `${bodyRowSelector} td:nth-child(2)`
  );
  for (const prioCell of priorityCells) {
    prioCell.addEventListener("click", editCell);
  }
  const contentCells = document.querySelectorAll(
    `${bodyRowSelector} td:nth-child(4)`
  );
  for (const contentCell of contentCells) {
    contentCell.addEventListener("click", editCell);
  }
  const metadataCells = document.querySelectorAll(
    `${bodyRowSelector} td:nth-child(5)`
  );
  for (const metadataCell of metadataCells) {
    metadataCell.addEventListener("click", editCell);
  }
  // INPUT HANDLERS
  // ===============================================================================================
  const priorityInputCells = document.querySelectorAll(
    `${bodyRowSelector} td:nth-child(2) > input`
  );
  for (const prioCell of priorityInputCells) {
    prioCell.addEventListener("blur", blurCell);
    prioCell.addEventListener("change", changePriority);
  }
  const contentInputCells = document.querySelectorAll(
    `${bodyRowSelector} td:nth-child(4) > input`
  );
  for (const contentCell of contentInputCells) {
    contentCell.addEventListener("blur", blurCell);
    contentCell.addEventListener("change", changeContent);
  }
  const metadataInputCells = document.querySelectorAll(
    `${bodyRowSelector} td:nth-child(5) > input`
  );
  for (const metadataCell of metadataInputCells) {
    metadataCell.addEventListener("blur", blurCell);
    metadataCell.addEventListener("change", changeMetadata);
  }

  // Edit functions
  // ==========================================================================================
  function editCell() {
    const input = this.querySelector("input");
    const display = this.querySelector("span");
    input.classList.remove("hidden");
    display.classList.add("hidden");
    if (document.activeElement !== input) {
      input.focus();
      input.selectionStart = input.selectionEnd = input.value.length;
    }
  }

  function blurCell() {
    const display = this.parentNode.querySelector("span");
    display.classList.remove("hidden");
    this.classList.add("hidden");
  }

  const alphaPattern = /[a-zA-Z]/;
  async function changePriority(e) {
    const {
      target: { value },
    } = e;
    const display = this.parentNode.querySelector("span");
    if (alphaPattern.test(value)) {
      const dataIdx = this.parentNode.parentNode.getAttribute("data-idx");
      if (!dataIdx) {
        throw new Error("All cells should render with a data index.");
      }
      try {
        const response = await updateTask({
          id: dataIdx,
          data: {
            priority: value.toUpperCase(),
          },
        });
        const text = await response.json();
        display.innerText = text;
      } catch (e) {
        console.error(e);
        e.target.value = display.innerText;
      }
    } else {
      e.target.value = display.innerText;
    }
  }

  async function changeContent(e) {
    const {
      target: { value },
    } = e;
    const display = this.parentNode.querySelector("span");
    const dataIdx = this.parentNode.parentNode.getAttribute("data-idx");
    if (!dataIdx) {
      throw new Error("All cells should render with a data index.");
    }
    try {
      const response = await updateTask({
        id: dataIdx,
        data: {
          content: value,
        },
      });
      const text = await response.json();
      display.innerHTML = `${text}`;
    } catch (e) {
      console.error(e);
      e.target.value = display.innerText;
    }
  }

  async function changeMetadata(e) {
    const {
      target: { value },
    } = e;
    const display = this.parentNode.querySelector("span");
    const dataIdx = this.parentNode.parentNode.getAttribute("data-idx");
    if (!dataIdx) {
      throw new Error("All cells should render with a data index.");
    }

    try {
      const response = await updateTask({
        id: dataIdx,
        data: {
          metadata: value,
        },
      });
      const text = await response.json();
      display.innerHTML = `${text}`;
    } catch (e) {
      console.error(e);
      e.target.value = display.innerText;
    }
  }

  async function updateCellStatus() {
    const dataIdx = this.parentNode.getAttribute("data-idx");
    if (!dataIdx) {
      throw new Error("All cells should render with a data index.");
    }
    const checkedStatus = this.getAttribute("aria-checked");
    const data = getStatusPayload(checkedStatus);

    try {
      const response = await updateTask({
        id: dataIdx,
        data,
      });
      const text = await response.json();
      this.innerHTML = text;
      checkedStatus === "true"
        ? this.setAttribute("aria-checked", "false")
        : this.setAttribute("aria-checked", "true");
    } catch (e) {
      console.error(e);
    }
  }

  function getStatusPayload(checkedStatus) {
    if (checkedStatus === "true") {
      return { completed: { done: false, date: undefined } };
    } else {
      const today = new Date();
      const month = today.getMonth() + 1;
      const day = today.getDay();
      return {
        completed: {
          done: true,
          date: `${today.getFullYear()}-${month > 10 ? month : `0${month}`}-${
            day > 10 ? day : `0${day}`
          }`,
        },
      };
    }
  }

  // Util functions
  // ===========================================================================================
  function sortBy(sortFn) {
    return function (_) {
      // clear aria sort roles on other sortable headers;
      for (const el of document.querySelectorAll("[aria-sort*='ending']")) {
        el !== this && el.setAttribute("aria-sort", undefined);
      }
      let dir;
      const sortDir = this.getAttribute("aria-sort");
      switch (sortDir) {
        case SORT_DIR.ASC:
          dir = SORT_DIR.DESC;
          this.setAttribute("aria-sort", SORT_DIR.DESC);
          break;
        case SORT_DIR.DESC:
          dir = SORT_DIR.ASC;
          this.setAttribute("aria-sort", SORT_DIR.ASC);
          break;
        default:
          dir = SORT_DIR.DESC;
          this.setAttribute("aria-sort", SORT_DIR.DESC);
          break;
      }
      const rowWrapper = document.querySelector("tbody");
      const taskList = Array.from(
        document.querySelectorAll("[role='row']").values()
      ).sort(sortFn(dir));
      for (const task of taskList) {
        rowWrapper.appendChild(task);
      }
    };
  }

  /**
   * @param task TaskRecord { id: number, data: Record<String, String> }
   */
  async function updateTask(task) {
    return fetch(`/tasks/update/${task.id}`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "same-origin",
      body: JSON.stringify(task.data),
    });
  }

  // Sort functions
  // All take a <tr> element as the sort element
  function status(direction) {
    return function (a, b) {
      const statusA = a.querySelector("td:first-child").innerText;
      const statusB = b.querySelector("td:first-child").innerText;
      const dateComp = sortDate(a, b);
      if (direction === SORT_DIR.ASC) {
        if (statusA > statusB) return 1;
        if (statusA < statusB) return -1;
        return 0 + dateComp;
      }
      if (statusB > statusA) return 1;
      if (statusB < statusA) return -1;
      return 0 + dateComp;
    };
  }

  function priority(direction) {
    return function (a, b) {
      const prioA = a.querySelector("td:nth-child(2)").innerText;
      const prioB = b.querySelector("td:nth-child(2)").innerText;
      const dateComp = sortDate(a, b);
      if (direction === SORT_DIR.ASC) {
        if (prioA > prioB) return 1;
        if (prioA < prioB) return -1;
        return 0 + dateComp;
      }
      if (prioA > prioB) return -1;
      if (prioA < prioB) return 1;
      return 0 + dateComp;
    };
  }

  function sortDate(a, b) {
    const dateA = a.querySelector("td:nth-child(3)").innerText;
    const dateB = b.querySelector("td:nth-child(3)").innerText;
    switch (true) {
      case !dateA && !dateB:
        return 0;
      case dateA && !dateB:
        return -1;
      case !dateA && dateB:
        return 1;
      case Boolean(dateA) && Boolean(dateB): {
        const toDateA = new Date(dateA);
        const toDateB = new Date(dateB);
        if (toDateA > toDateB) return -1;
        if (toDateA < toDateB) return 1;
        return 0;
      }
      default:
        return 0;
    }
  }
})();
