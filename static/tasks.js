(function () {
  const SORT_DIR = {
    ASC: "ascending",
    DESC: "descending",
  };

  // Event Listeners
  // =======================================================================================
  const statusHeader = document.querySelector("thead > tr > th:first-child");
  statusHeader.addEventListener("click", sortBy(status));
  const prioHeader = document.querySelector("thead > tr > th:nth-child(2)");
  prioHeader.addEventListener("click", sortBy(priority));
  const statusCells = document.querySelectorAll("tbody tr > td:first-child");
  for (const statusCell of statusCells) {
    statusCell.addEventListener("click", updateCellStatus);
  }

  // Util functions
  // ===========================================================================================
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

  async function updateCellStatus() {
    const dataIdx = this.parentNode.getAttribute("data-idx");
    if (!dataIdx) {
      throw new Error("All cells should render with a data index.");
    }
    if (this.getAttribute("aria-checked") === "true") {
      const response = await updateTask({
        id: dataIdx,
        data: { completed: { done: false, date: undefined } },
      });
      const text = await response.json();
      this.innerHTML = text;
      this.setAttribute("aria-checked", "false");
    } else {
      const today = new Date();
      const month = today.getMonth() + 1;
      const day = today.getDay();
      const response = await updateTask({
        id: dataIdx,
        data: {
          completed: {
            done: true,
            date: `${today.getFullYear()}-${month > 10 ? month : `0${month}`}-${
              day > 10 ? day : `0${day}`
            }`,
          },
        },
      });
      const text = await response.json();
      this.innerHTML = text;
      this.setAttribute("aria-checked", "true");
    }
  }
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
