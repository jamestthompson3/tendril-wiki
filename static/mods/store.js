// TODO add debugging so you can label mutations to the store to track down what functions are changing your data
export function observableStore(initialValues = {}) {
  const store = initialValues;
  const listeners = new Map();
  const handler = {
    get: function (prop) {
      // specific events
      if (listeners.get("access")?.has(prop)) {
        const callbacks = listeners.get("access").get(prop);
        for (const listener of callbacks) {
          listener.cb(value);
          if (listener.once) {
            callbacks.delete(listener);
          }
        }
      }
      // '*' or all events
      if (listeners.get("access")?.has("*")) {
        const callbacks = listeners.get("access").get("*");
        for (const listener of callbacks) {
          listener.cb(value);
          if (listener.once) {
            callbacks.delete(listener);
          }
        }
      }
      return Reflect.get(...arguments);
    },
    set: function (_, prop, value) {
      Reflect.set(...arguments);
      // specific events
      if (listeners.get("update")?.has(prop)) {
        const callbacks = listeners.get("update").get(prop);
        for (const listener of callbacks) {
          listener.cb(value);
          if (listener.once) {
            callbacks.delete(listener);
          }
        }
      }
      // '*' or all events
      if (listeners.get("update")?.has("*")) {
        const callbacks = listeners.get("update").get("*");
        for (const listener of callbacks) {
          listener.cb(value);
          if (listener.once) {
            callbacks.delete(listener);
          }
        }
      }
      return true;
    },
  };
  const proxied = new Proxy(store, handler);
  const createListener = (event, key, cb, once) => {
    const eventMap = listeners.get(event); // for example, if we have set up listeners for the "update" event
    if (eventMap) {
      const eventListenerSet = eventMap.get(key); // if there is a listener for "update" events on a specific key
      if (eventListenerSet) {
        // appened event listener to the existing set.
        eventListenerSet.add({ cb, once });
      } else {
        eventMap.set(key, new Set([{ cb, once }]));
      }
    } else {
      // create a new entry
      const newEventMap = new Map();
      newEventMap.set(key, new Set([{ cb, once }]));
      listeners.set(event, newEventMap);
    }
  };
  return {
    set(key, value) {
      proxied[key] = value;
    },
    /**
     * @param event {StoreEvents}
     * @param key {string}
     * @param cb {fn(): void}
     */
    on(event, key, cb) {
      createListener(event, key, cb, false);
    },
    // TODO make this return sub stores
    get(key) {
      return getNestedKeys(proxied, key);
    },
    select(key) {
      return observableStore(getNestedKeys(proxied, key));
    },
    once(event, key, cb) {
      createListener(event, key, cb, true);
    },
    keys() {
      return Object.keys(store);
    },
  };
}

function getNestedKeys(reference, keyString) {
  const nestedKeys = keyString.split(".");
  let finalProp = reference;
  for (const key of nestedKeys) {
    if (finalProp[key] === undefined) {
      break;
    }
    finalProp = finalProp[key];
  }
  return finalProp;
}
