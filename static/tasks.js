(function () {
  const SORT_DIR = {
    ASC: "ascending",
    DESC: "descending",
  };

  // Event Listeners
  // =======================================================================================
  const headerRowSelector = "thead > tr >";
  // const bodyRowSelector = "tbody tr >";
  const statusHeader = document.querySelector(
    `${headerRowSelector} th:nth-child(2)`
  );
  const form = document.querySelector(".task-header form");
  form.addEventListener("submit", addTask);
  statusHeader.addEventListener("click", sortBy(status));
  const prioHeader = document.querySelector(
    `${headerRowSelector} th:nth-child(3)`
  );
  prioHeader.addEventListener("click", sortBy(priority));
  for (const row of document.querySelectorAll("tbody tr")) {
    setupRowEventHandlers(row);
  }

  // Edit functions
  // ==========================================================================================
  async function addTask(e) {
    e.preventDefault();
    const data = new FormData(this);
    const task = data.get("task");
    if (task === "") {
      return;
    }
    const body = {
      content: `${formatDate()} ${task}`,
    };
    try {
      const request = await fetch("/tasks/create", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        credentials: "same-origin",
        body: JSON.stringify(body),
      });
      if (request.status === 200) {
        const response = await request.json();
        const rowParser = document.createElement("tbody");
        rowParser.innerHTML = response;
        const rowWrapper = document.querySelector("tbody");
        const firstRow = rowWrapper.querySelector(":first-child");
        for (const row of document.querySelectorAll("tbody tr")) {
          const idx = parseInt(row.getAttribute("data-idx"));
          row.setAttribute("data-idx", idx + 1);
        }
        const newRow = rowParser.querySelector(":first-child");
        setupRowEventHandlers(newRow);
        rowWrapper.insertBefore(newRow, firstRow);
        // reset the form
        this.elements["task"].value = "";
      } else {
        const errorMsg = document.createElement("span");
        errorMsg.innerHTML = `Could not create task: ${request.statusText}`;
        errorMsg.style.color = "red";
        this.parentNode.appendChild(errorMsg);
      }
    } catch (e) {
      const errorMsg = document.createElement("span");
      errorMsg.innerHTML = `Could not create task: ${e}`;
      errorMsg.style.color = "red";
      this.parentNode.appendChild(errorMsg);
    }
  }
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

  async function deleteTask() {
    const dataIdx = this.parentNode.getAttribute("data-idx");
    if (!dataIdx) {
      throw new Error("All cells should render with a data index.");
    }
    try {
      const request = await fetch(`/tasks/delete/${dataIdx}`, {
        method: "DELETE",
        credentials: "same-origin",
      });
      const response = await request.json();

      for (const remainingRow of document.querySelectorAll("tbody tr")) {
        const idx = parseInt(remainingRow.getAttribute("data-idx"));
        if (idx === response) {
          remainingRow.parentNode.removeChild(remainingRow);
        }
        if (idx > response) {
          remainingRow.setAttribute("data-idx", idx - 1);
        }
      }
    } catch (e) {
      console.error(e);
    }
  }

  const alphaPattern = /[a-zA-Z]/;
  async function changePriority(e) {
    const {
      target: { value },
    } = e;
    const display = this.parentNode.querySelector("span");
    if (value === display.innerText) return;
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
    if (value === display.innerText) return;
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
      return {
        completed: {
          done: true,
          date: formatDate(),
        },
      };
    }
  }

  // Util functions
  // ===========================================================================================
  function formatDate() {
    const today = new Date();
    const month = today.getMonth() + 1;
    const day = today.getDay();
    return `${today.getFullYear()}-${month > 10 ? month : `0${month}`}-${
      day > 10 ? day : `0${day}`
    }`;
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

  function setupRowEventHandlers(row) {
    const deleteCell = row.querySelector("td:first-child");
    deleteCell.addEventListener("click", deleteTask);
    const statusCell = row.querySelector("td:nth-child(2)");
    statusCell.addEventListener("click", updateCellStatus);
    const prioCell = row.querySelector("td:nth-child(3)");
    prioCell.addEventListener("click", editCell);
    const contentCell = row.querySelector("td:nth-child(5)");
    contentCell.addEventListener("click", editCell);
    const metadataCell = row.querySelector("td:nth-child(6)");
    metadataCell.addEventListener("click", editCell);

    const priorityInputCell = row.querySelector("td:nth-child(3) > input");
    priorityInputCell.addEventListener("blur", blurCell);
    priorityInputCell.addEventListener("change", changePriority);
    const contentInputCell = row.querySelector("td:nth-child(5) > input");
    contentInputCell.addEventListener("blur", blurCell);
    contentInputCell.addEventListener("change", changeContent);
    const metadataInputCell = row.querySelector("td:nth-child(6) > input");
    metadataInputCell.addEventListener("blur", blurCell);
    metadataInputCell.addEventListener("change", changeMetadata);
  }
})();
