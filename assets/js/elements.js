var UR = Object.defineProperty;
var GR = (t, r, i) => r in t ? UR(t, r, { enumerable: !0, configurable: !0, writable: !0, value: i }) : t[r] = i;
var Hf = (t, r, i) => GR(t, typeof r != "symbol" ? r + "" : r, i);
const Wb = "data-search";
class VR extends HTMLElement {
  constructor() {
    super(...arguments);
    Hf(this, "input");
  }
  connectedCallback() {
    this.dataset.enhanced !== "true" && (this.dataset.enhanced = "true", this.render());
  }
  get catalog() {
    const i = this.getAttribute("for");
    return i ? document.getElementById(i) : null;
  }
  render() {
    const i = document.createElement("label");
    i.className = "collection-search";
    const u = document.createElement("input");
    u.type = "search", u.placeholder = this.getAttribute("placeholder") ?? "Filter by name, author, tag, loop shape, issue type...", u.setAttribute("aria-label", "Filter collections"), u.addEventListener("input", () => this.applyFilter(u.value)), this.input = u;
    const l = document.createElement("button");
    l.type = "button", l.className = "collection-view-toggle", l.addEventListener("click", () => this.toggleView(l));
    const s = document.createElement("span");
    s.className = "collection-search-count", s.setAttribute("role", "status"), i.append(u, l, s), this.append(i), this.syncToggleLabel(l), this.applyFilter("");
  }
  currentView() {
    var i;
    return ((i = this.catalog) == null ? void 0 : i.dataset.view) === "table" ? "table" : "cards";
  }
  syncToggleLabel(i) {
    const u = this.currentView() === "cards" ? "table" : "cards";
    i.textContent = `View as ${u}`, i.setAttribute("aria-label", `Switch to ${u} view`);
  }
  toggleView(i) {
    var l;
    const u = this.catalog;
    u && (u.dataset.view = this.currentView() === "cards" ? "table" : "cards", this.syncToggleLabel(i), this.applyFilter(((l = this.input) == null ? void 0 : l.value) ?? ""));
  }
  applyFilter(i) {
    const u = this.catalog;
    if (!u)
      return;
    const l = i.toLowerCase().split(/\s+/).filter(Boolean), s = u.querySelector(`[data-view-target="${this.currentView()}"]`);
    let c = 0, f = 0;
    for (const h of u.querySelectorAll(`[${Wb}]`)) {
      const v = (h.getAttribute(Wb) ?? "").toLowerCase(), y = l.every((m) => v.includes(m));
      h.hidden = !y, s != null && s.contains(h) && (f += 1, y && (c += 1));
    }
    const g = this.querySelector(".collection-search-count");
    g && (g.textContent = c === f ? `${f} collections` : `${c} of ${f} collections`), u.dataset.empty = c === 0 ? "true" : "false";
  }
}
const YR = "operator-collection-search";
var bo = typeof globalThis < "u" ? globalThis : typeof window < "u" ? window : typeof global < "u" ? global : typeof self < "u" ? self : {};
function Ly(t) {
  return t && t.__esModule && Object.prototype.hasOwnProperty.call(t, "default") ? t.default : t;
}
var jf = { exports: {} }, yu = {};
/**
 * @license React
 * react-jsx-runtime.production.js
 *
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
var e_;
function kR() {
  if (e_) return yu;
  e_ = 1;
  var t = Symbol.for("react.transitional.element"), r = Symbol.for("react.fragment");
  function i(u, l, s) {
    var c = null;
    if (s !== void 0 && (c = "" + s), l.key !== void 0 && (c = "" + l.key), "key" in l) {
      s = {};
      for (var f in l)
        f !== "key" && (s[f] = l[f]);
    } else s = l;
    return l = s.ref, {
      $$typeof: t,
      type: u,
      key: c,
      ref: l !== void 0 ? l : null,
      props: s
    };
  }
  return yu.Fragment = r, yu.jsx = i, yu.jsxs = i, yu;
}
var t_;
function XR() {
  return t_ || (t_ = 1, jf.exports = kR()), jf.exports;
}
var k = XR(), Bf = { exports: {} }, we = {};
/**
 * @license React
 * react.production.js
 *
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
var n_;
function IR() {
  if (n_) return we;
  n_ = 1;
  var t = Symbol.for("react.transitional.element"), r = Symbol.for("react.portal"), i = Symbol.for("react.fragment"), u = Symbol.for("react.strict_mode"), l = Symbol.for("react.profiler"), s = Symbol.for("react.consumer"), c = Symbol.for("react.context"), f = Symbol.for("react.forward_ref"), g = Symbol.for("react.suspense"), h = Symbol.for("react.memo"), v = Symbol.for("react.lazy"), y = Symbol.for("react.activity"), m = Symbol.iterator;
  function b(O) {
    return O === null || typeof O != "object" ? null : (O = m && O[m] || O["@@iterator"], typeof O == "function" ? O : null);
  }
  var _ = {
    isMounted: function() {
      return !1;
    },
    enqueueForceUpdate: function() {
    },
    enqueueReplaceState: function() {
    },
    enqueueSetState: function() {
    }
  }, A = Object.assign, w = {};
  function E(O, G, ie) {
    this.props = O, this.context = G, this.refs = w, this.updater = ie || _;
  }
  E.prototype.isReactComponent = {}, E.prototype.setState = function(O, G) {
    if (typeof O != "object" && typeof O != "function" && O != null)
      throw Error(
        "takes an object of state variables to update or a function which returns an object of state variables."
      );
    this.updater.enqueueSetState(this, O, G, "setState");
  }, E.prototype.forceUpdate = function(O) {
    this.updater.enqueueForceUpdate(this, O, "forceUpdate");
  };
  function M() {
  }
  M.prototype = E.prototype;
  function S(O, G, ie) {
    this.props = O, this.context = G, this.refs = w, this.updater = ie || _;
  }
  var T = S.prototype = new M();
  T.constructor = S, A(T, E.prototype), T.isPureReactComponent = !0;
  var N = Array.isArray;
  function R() {
  }
  var q = { H: null, A: null, T: null, S: null }, L = Object.prototype.hasOwnProperty;
  function j(O, G, ie) {
    var H = ie.ref;
    return {
      $$typeof: t,
      type: O,
      key: G,
      ref: H !== void 0 ? H : null,
      props: ie
    };
  }
  function Z(O, G) {
    return j(O.type, G, O.props);
  }
  function Y(O) {
    return typeof O == "object" && O !== null && O.$$typeof === t;
  }
  function J(O) {
    var G = { "=": "=0", ":": "=2" };
    return "$" + O.replace(/[=:]/g, function(ie) {
      return G[ie];
    });
  }
  var Q = /\/+/g;
  function D(O, G) {
    return typeof O == "object" && O !== null && O.key != null ? J("" + O.key) : G.toString(36);
  }
  function X(O) {
    switch (O.status) {
      case "fulfilled":
        return O.value;
      case "rejected":
        throw O.reason;
      default:
        switch (typeof O.status == "string" ? O.then(R, R) : (O.status = "pending", O.then(
          function(G) {
            O.status === "pending" && (O.status = "fulfilled", O.value = G);
          },
          function(G) {
            O.status === "pending" && (O.status = "rejected", O.reason = G);
          }
        )), O.status) {
          case "fulfilled":
            return O.value;
          case "rejected":
            throw O.reason;
        }
    }
    throw O;
  }
  function z(O, G, ie, H, K) {
    var P = typeof O;
    (P === "undefined" || P === "boolean") && (O = null);
    var ue = !1;
    if (O === null) ue = !0;
    else
      switch (P) {
        case "bigint":
        case "string":
        case "number":
          ue = !0;
          break;
        case "object":
          switch (O.$$typeof) {
            case t:
            case r:
              ue = !0;
              break;
            case v:
              return ue = O._init, z(
                ue(O._payload),
                G,
                ie,
                H,
                K
              );
          }
      }
    if (ue)
      return K = K(O), ue = H === "" ? "." + D(O, 0) : H, N(K) ? (ie = "", ue != null && (ie = ue.replace(Q, "$&/") + "/"), z(K, G, ie, "", function(de) {
        return de;
      })) : K != null && (Y(K) && (K = Z(
        K,
        ie + (K.key == null || O && O.key === K.key ? "" : ("" + K.key).replace(
          Q,
          "$&/"
        ) + "/") + ue
      )), G.push(K)), 1;
    ue = 0;
    var W = H === "" ? "." : H + ":";
    if (N(O))
      for (var se = 0; se < O.length; se++)
        H = O[se], P = W + D(H, se), ue += z(
          H,
          G,
          ie,
          P,
          K
        );
    else if (se = b(O), typeof se == "function")
      for (O = se.call(O), se = 0; !(H = O.next()).done; )
        H = H.value, P = W + D(H, se++), ue += z(
          H,
          G,
          ie,
          P,
          K
        );
    else if (P === "object") {
      if (typeof O.then == "function")
        return z(
          X(O),
          G,
          ie,
          H,
          K
        );
      throw G = String(O), Error(
        "Objects are not valid as a React child (found: " + (G === "[object Object]" ? "object with keys {" + Object.keys(O).join(", ") + "}" : G) + "). If you meant to render a collection of children, use an array instead."
      );
    }
    return ue;
  }
  function B(O, G, ie) {
    if (O == null) return O;
    var H = [], K = 0;
    return z(O, H, "", "", function(P) {
      return G.call(ie, P, K++);
    }), H;
  }
  function $(O) {
    if (O._status === -1) {
      var G = O._result;
      G = G(), G.then(
        function(ie) {
          (O._status === 0 || O._status === -1) && (O._status = 1, O._result = ie);
        },
        function(ie) {
          (O._status === 0 || O._status === -1) && (O._status = 2, O._result = ie);
        }
      ), O._status === -1 && (O._status = 0, O._result = G);
    }
    if (O._status === 1) return O._result.default;
    throw O._result;
  }
  var U = typeof reportError == "function" ? reportError : function(O) {
    if (typeof window == "object" && typeof window.ErrorEvent == "function") {
      var G = new window.ErrorEvent("error", {
        bubbles: !0,
        cancelable: !0,
        message: typeof O == "object" && O !== null && typeof O.message == "string" ? String(O.message) : String(O),
        error: O
      });
      if (!window.dispatchEvent(G)) return;
    } else if (typeof process == "object" && typeof process.emit == "function") {
      process.emit("uncaughtException", O);
      return;
    }
    console.error(O);
  }, ae = {
    map: B,
    forEach: function(O, G, ie) {
      B(
        O,
        function() {
          G.apply(this, arguments);
        },
        ie
      );
    },
    count: function(O) {
      var G = 0;
      return B(O, function() {
        G++;
      }), G;
    },
    toArray: function(O) {
      return B(O, function(G) {
        return G;
      }) || [];
    },
    only: function(O) {
      if (!Y(O))
        throw Error(
          "React.Children.only expected to receive a single React element child."
        );
      return O;
    }
  };
  return we.Activity = y, we.Children = ae, we.Component = E, we.Fragment = i, we.Profiler = l, we.PureComponent = S, we.StrictMode = u, we.Suspense = g, we.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE = q, we.__COMPILER_RUNTIME = {
    __proto__: null,
    c: function(O) {
      return q.H.useMemoCache(O);
    }
  }, we.cache = function(O) {
    return function() {
      return O.apply(null, arguments);
    };
  }, we.cacheSignal = function() {
    return null;
  }, we.cloneElement = function(O, G, ie) {
    if (O == null)
      throw Error(
        "The argument must be a React element, but you passed " + O + "."
      );
    var H = A({}, O.props), K = O.key;
    if (G != null)
      for (P in G.key !== void 0 && (K = "" + G.key), G)
        !L.call(G, P) || P === "key" || P === "__self" || P === "__source" || P === "ref" && G.ref === void 0 || (H[P] = G[P]);
    var P = arguments.length - 2;
    if (P === 1) H.children = ie;
    else if (1 < P) {
      for (var ue = Array(P), W = 0; W < P; W++)
        ue[W] = arguments[W + 2];
      H.children = ue;
    }
    return j(O.type, K, H);
  }, we.createContext = function(O) {
    return O = {
      $$typeof: c,
      _currentValue: O,
      _currentValue2: O,
      _threadCount: 0,
      Provider: null,
      Consumer: null
    }, O.Provider = O, O.Consumer = {
      $$typeof: s,
      _context: O
    }, O;
  }, we.createElement = function(O, G, ie) {
    var H, K = {}, P = null;
    if (G != null)
      for (H in G.key !== void 0 && (P = "" + G.key), G)
        L.call(G, H) && H !== "key" && H !== "__self" && H !== "__source" && (K[H] = G[H]);
    var ue = arguments.length - 2;
    if (ue === 1) K.children = ie;
    else if (1 < ue) {
      for (var W = Array(ue), se = 0; se < ue; se++)
        W[se] = arguments[se + 2];
      K.children = W;
    }
    if (O && O.defaultProps)
      for (H in ue = O.defaultProps, ue)
        K[H] === void 0 && (K[H] = ue[H]);
    return j(O, P, K);
  }, we.createRef = function() {
    return { current: null };
  }, we.forwardRef = function(O) {
    return { $$typeof: f, render: O };
  }, we.isValidElement = Y, we.lazy = function(O) {
    return {
      $$typeof: v,
      _payload: { _status: -1, _result: O },
      _init: $
    };
  }, we.memo = function(O, G) {
    return {
      $$typeof: h,
      type: O,
      compare: G === void 0 ? null : G
    };
  }, we.startTransition = function(O) {
    var G = q.T, ie = {};
    q.T = ie;
    try {
      var H = O(), K = q.S;
      K !== null && K(ie, H), typeof H == "object" && H !== null && typeof H.then == "function" && H.then(R, U);
    } catch (P) {
      U(P);
    } finally {
      G !== null && ie.types !== null && (G.types = ie.types), q.T = G;
    }
  }, we.unstable_useCacheRefresh = function() {
    return q.H.useCacheRefresh();
  }, we.use = function(O) {
    return q.H.use(O);
  }, we.useActionState = function(O, G, ie) {
    return q.H.useActionState(O, G, ie);
  }, we.useCallback = function(O, G) {
    return q.H.useCallback(O, G);
  }, we.useContext = function(O) {
    return q.H.useContext(O);
  }, we.useDebugValue = function() {
  }, we.useDeferredValue = function(O, G) {
    return q.H.useDeferredValue(O, G);
  }, we.useEffect = function(O, G) {
    return q.H.useEffect(O, G);
  }, we.useEffectEvent = function(O) {
    return q.H.useEffectEvent(O);
  }, we.useId = function() {
    return q.H.useId();
  }, we.useImperativeHandle = function(O, G, ie) {
    return q.H.useImperativeHandle(O, G, ie);
  }, we.useInsertionEffect = function(O, G) {
    return q.H.useInsertionEffect(O, G);
  }, we.useLayoutEffect = function(O, G) {
    return q.H.useLayoutEffect(O, G);
  }, we.useMemo = function(O, G) {
    return q.H.useMemo(O, G);
  }, we.useOptimistic = function(O, G) {
    return q.H.useOptimistic(O, G);
  }, we.useReducer = function(O, G, ie) {
    return q.H.useReducer(O, G, ie);
  }, we.useRef = function(O) {
    return q.H.useRef(O);
  }, we.useState = function(O) {
    return q.H.useState(O);
  }, we.useSyncExternalStore = function(O, G, ie) {
    return q.H.useSyncExternalStore(
      O,
      G,
      ie
    );
  }, we.useTransition = function() {
    return q.H.useTransition();
  }, we.version = "19.2.8", we;
}
var r_;
function ju() {
  return r_ || (r_ = 1, Bf.exports = IR()), Bf.exports;
}
var re = ju();
const ZR = /* @__PURE__ */ Ly(re);
var Uf = { exports: {} }, mu = {}, Gf = { exports: {} }, Vf = {};
/**
 * @license React
 * scheduler.production.js
 *
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
var a_;
function KR() {
  return a_ || (a_ = 1, (function(t) {
    function r(z, B) {
      var $ = z.length;
      z.push(B);
      e: for (; 0 < $; ) {
        var U = $ - 1 >>> 1, ae = z[U];
        if (0 < l(ae, B))
          z[U] = B, z[$] = ae, $ = U;
        else break e;
      }
    }
    function i(z) {
      return z.length === 0 ? null : z[0];
    }
    function u(z) {
      if (z.length === 0) return null;
      var B = z[0], $ = z.pop();
      if ($ !== B) {
        z[0] = $;
        e: for (var U = 0, ae = z.length, O = ae >>> 1; U < O; ) {
          var G = 2 * (U + 1) - 1, ie = z[G], H = G + 1, K = z[H];
          if (0 > l(ie, $))
            H < ae && 0 > l(K, ie) ? (z[U] = K, z[H] = $, U = H) : (z[U] = ie, z[G] = $, U = G);
          else if (H < ae && 0 > l(K, $))
            z[U] = K, z[H] = $, U = H;
          else break e;
        }
      }
      return B;
    }
    function l(z, B) {
      var $ = z.sortIndex - B.sortIndex;
      return $ !== 0 ? $ : z.id - B.id;
    }
    if (t.unstable_now = void 0, typeof performance == "object" && typeof performance.now == "function") {
      var s = performance;
      t.unstable_now = function() {
        return s.now();
      };
    } else {
      var c = Date, f = c.now();
      t.unstable_now = function() {
        return c.now() - f;
      };
    }
    var g = [], h = [], v = 1, y = null, m = 3, b = !1, _ = !1, A = !1, w = !1, E = typeof setTimeout == "function" ? setTimeout : null, M = typeof clearTimeout == "function" ? clearTimeout : null, S = typeof setImmediate < "u" ? setImmediate : null;
    function T(z) {
      for (var B = i(h); B !== null; ) {
        if (B.callback === null) u(h);
        else if (B.startTime <= z)
          u(h), B.sortIndex = B.expirationTime, r(g, B);
        else break;
        B = i(h);
      }
    }
    function N(z) {
      if (A = !1, T(z), !_)
        if (i(g) !== null)
          _ = !0, R || (R = !0, J());
        else {
          var B = i(h);
          B !== null && X(N, B.startTime - z);
        }
    }
    var R = !1, q = -1, L = 5, j = -1;
    function Z() {
      return w ? !0 : !(t.unstable_now() - j < L);
    }
    function Y() {
      if (w = !1, R) {
        var z = t.unstable_now();
        j = z;
        var B = !0;
        try {
          e: {
            _ = !1, A && (A = !1, M(q), q = -1), b = !0;
            var $ = m;
            try {
              t: {
                for (T(z), y = i(g); y !== null && !(y.expirationTime > z && Z()); ) {
                  var U = y.callback;
                  if (typeof U == "function") {
                    y.callback = null, m = y.priorityLevel;
                    var ae = U(
                      y.expirationTime <= z
                    );
                    if (z = t.unstable_now(), typeof ae == "function") {
                      y.callback = ae, T(z), B = !0;
                      break t;
                    }
                    y === i(g) && u(g), T(z);
                  } else u(g);
                  y = i(g);
                }
                if (y !== null) B = !0;
                else {
                  var O = i(h);
                  O !== null && X(
                    N,
                    O.startTime - z
                  ), B = !1;
                }
              }
              break e;
            } finally {
              y = null, m = $, b = !1;
            }
            B = void 0;
          }
        } finally {
          B ? J() : R = !1;
        }
      }
    }
    var J;
    if (typeof S == "function")
      J = function() {
        S(Y);
      };
    else if (typeof MessageChannel < "u") {
      var Q = new MessageChannel(), D = Q.port2;
      Q.port1.onmessage = Y, J = function() {
        D.postMessage(null);
      };
    } else
      J = function() {
        E(Y, 0);
      };
    function X(z, B) {
      q = E(function() {
        z(t.unstable_now());
      }, B);
    }
    t.unstable_IdlePriority = 5, t.unstable_ImmediatePriority = 1, t.unstable_LowPriority = 4, t.unstable_NormalPriority = 3, t.unstable_Profiling = null, t.unstable_UserBlockingPriority = 2, t.unstable_cancelCallback = function(z) {
      z.callback = null;
    }, t.unstable_forceFrameRate = function(z) {
      0 > z || 125 < z ? console.error(
        "forceFrameRate takes a positive int between 0 and 125, forcing frame rates higher than 125 fps is not supported"
      ) : L = 0 < z ? Math.floor(1e3 / z) : 5;
    }, t.unstable_getCurrentPriorityLevel = function() {
      return m;
    }, t.unstable_next = function(z) {
      switch (m) {
        case 1:
        case 2:
        case 3:
          var B = 3;
          break;
        default:
          B = m;
      }
      var $ = m;
      m = B;
      try {
        return z();
      } finally {
        m = $;
      }
    }, t.unstable_requestPaint = function() {
      w = !0;
    }, t.unstable_runWithPriority = function(z, B) {
      switch (z) {
        case 1:
        case 2:
        case 3:
        case 4:
        case 5:
          break;
        default:
          z = 3;
      }
      var $ = m;
      m = z;
      try {
        return B();
      } finally {
        m = $;
      }
    }, t.unstable_scheduleCallback = function(z, B, $) {
      var U = t.unstable_now();
      switch (typeof $ == "object" && $ !== null ? ($ = $.delay, $ = typeof $ == "number" && 0 < $ ? U + $ : U) : $ = U, z) {
        case 1:
          var ae = -1;
          break;
        case 2:
          ae = 250;
          break;
        case 5:
          ae = 1073741823;
          break;
        case 4:
          ae = 1e4;
          break;
        default:
          ae = 5e3;
      }
      return ae = $ + ae, z = {
        id: v++,
        callback: B,
        priorityLevel: z,
        startTime: $,
        expirationTime: ae,
        sortIndex: -1
      }, $ > U ? (z.sortIndex = $, r(h, z), i(g) === null && z === i(h) && (A ? (M(q), q = -1) : A = !0, X(N, $ - U))) : (z.sortIndex = ae, r(g, z), _ || b || (_ = !0, R || (R = !0, J()))), z;
    }, t.unstable_shouldYield = Z, t.unstable_wrapCallback = function(z) {
      var B = m;
      return function() {
        var $ = m;
        m = B;
        try {
          return z.apply(this, arguments);
        } finally {
          m = $;
        }
      };
    };
  })(Vf)), Vf;
}
var i_;
function QR() {
  return i_ || (i_ = 1, Gf.exports = KR()), Gf.exports;
}
var Yf = { exports: {} }, At = {};
/**
 * @license React
 * react-dom.production.js
 *
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
var u_;
function $R() {
  if (u_) return At;
  u_ = 1;
  var t = ju();
  function r(g) {
    var h = "https://react.dev/errors/" + g;
    if (1 < arguments.length) {
      h += "?args[]=" + encodeURIComponent(arguments[1]);
      for (var v = 2; v < arguments.length; v++)
        h += "&args[]=" + encodeURIComponent(arguments[v]);
    }
    return "Minified React error #" + g + "; visit " + h + " for the full message or use the non-minified dev environment for full errors and additional helpful warnings.";
  }
  function i() {
  }
  var u = {
    d: {
      f: i,
      r: function() {
        throw Error(r(522));
      },
      D: i,
      C: i,
      L: i,
      m: i,
      X: i,
      S: i,
      M: i
    },
    p: 0,
    findDOMNode: null
  }, l = Symbol.for("react.portal");
  function s(g, h, v) {
    var y = 3 < arguments.length && arguments[3] !== void 0 ? arguments[3] : null;
    return {
      $$typeof: l,
      key: y == null ? null : "" + y,
      children: g,
      containerInfo: h,
      implementation: v
    };
  }
  var c = t.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE;
  function f(g, h) {
    if (g === "font") return "";
    if (typeof h == "string")
      return h === "use-credentials" ? h : "";
  }
  return At.__DOM_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE = u, At.createPortal = function(g, h) {
    var v = 2 < arguments.length && arguments[2] !== void 0 ? arguments[2] : null;
    if (!h || h.nodeType !== 1 && h.nodeType !== 9 && h.nodeType !== 11)
      throw Error(r(299));
    return s(g, h, null, v);
  }, At.flushSync = function(g) {
    var h = c.T, v = u.p;
    try {
      if (c.T = null, u.p = 2, g) return g();
    } finally {
      c.T = h, u.p = v, u.d.f();
    }
  }, At.preconnect = function(g, h) {
    typeof g == "string" && (h ? (h = h.crossOrigin, h = typeof h == "string" ? h === "use-credentials" ? h : "" : void 0) : h = null, u.d.C(g, h));
  }, At.prefetchDNS = function(g) {
    typeof g == "string" && u.d.D(g);
  }, At.preinit = function(g, h) {
    if (typeof g == "string" && h && typeof h.as == "string") {
      var v = h.as, y = f(v, h.crossOrigin), m = typeof h.integrity == "string" ? h.integrity : void 0, b = typeof h.fetchPriority == "string" ? h.fetchPriority : void 0;
      v === "style" ? u.d.S(
        g,
        typeof h.precedence == "string" ? h.precedence : void 0,
        {
          crossOrigin: y,
          integrity: m,
          fetchPriority: b
        }
      ) : v === "script" && u.d.X(g, {
        crossOrigin: y,
        integrity: m,
        fetchPriority: b,
        nonce: typeof h.nonce == "string" ? h.nonce : void 0
      });
    }
  }, At.preinitModule = function(g, h) {
    if (typeof g == "string")
      if (typeof h == "object" && h !== null) {
        if (h.as == null || h.as === "script") {
          var v = f(
            h.as,
            h.crossOrigin
          );
          u.d.M(g, {
            crossOrigin: v,
            integrity: typeof h.integrity == "string" ? h.integrity : void 0,
            nonce: typeof h.nonce == "string" ? h.nonce : void 0
          });
        }
      } else h == null && u.d.M(g);
  }, At.preload = function(g, h) {
    if (typeof g == "string" && typeof h == "object" && h !== null && typeof h.as == "string") {
      var v = h.as, y = f(v, h.crossOrigin);
      u.d.L(g, v, {
        crossOrigin: y,
        integrity: typeof h.integrity == "string" ? h.integrity : void 0,
        nonce: typeof h.nonce == "string" ? h.nonce : void 0,
        type: typeof h.type == "string" ? h.type : void 0,
        fetchPriority: typeof h.fetchPriority == "string" ? h.fetchPriority : void 0,
        referrerPolicy: typeof h.referrerPolicy == "string" ? h.referrerPolicy : void 0,
        imageSrcSet: typeof h.imageSrcSet == "string" ? h.imageSrcSet : void 0,
        imageSizes: typeof h.imageSizes == "string" ? h.imageSizes : void 0,
        media: typeof h.media == "string" ? h.media : void 0
      });
    }
  }, At.preloadModule = function(g, h) {
    if (typeof g == "string")
      if (h) {
        var v = f(h.as, h.crossOrigin);
        u.d.m(g, {
          as: typeof h.as == "string" && h.as !== "script" ? h.as : void 0,
          crossOrigin: v,
          integrity: typeof h.integrity == "string" ? h.integrity : void 0
        });
      } else u.d.m(g);
  }, At.requestFormReset = function(g) {
    u.d.r(g);
  }, At.unstable_batchedUpdates = function(g, h) {
    return g(h);
  }, At.useFormState = function(g, h, v) {
    return c.H.useFormState(g, h, v);
  }, At.useFormStatus = function() {
    return c.H.useHostTransitionStatus();
  }, At.version = "19.2.8", At;
}
var l_;
function kA() {
  if (l_) return Yf.exports;
  l_ = 1;
  function t() {
    if (!(typeof __REACT_DEVTOOLS_GLOBAL_HOOK__ > "u" || typeof __REACT_DEVTOOLS_GLOBAL_HOOK__.checkDCE != "function"))
      try {
        __REACT_DEVTOOLS_GLOBAL_HOOK__.checkDCE(t);
      } catch (r) {
        console.error(r);
      }
  }
  return t(), Yf.exports = $R(), Yf.exports;
}
/**
 * @license React
 * react-dom-client.production.js
 *
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
var o_;
function FR() {
  if (o_) return mu;
  o_ = 1;
  var t = QR(), r = ju(), i = kA();
  function u(e) {
    var n = "https://react.dev/errors/" + e;
    if (1 < arguments.length) {
      n += "?args[]=" + encodeURIComponent(arguments[1]);
      for (var a = 2; a < arguments.length; a++)
        n += "&args[]=" + encodeURIComponent(arguments[a]);
    }
    return "Minified React error #" + e + "; visit " + n + " for the full message or use the non-minified dev environment for full errors and additional helpful warnings.";
  }
  function l(e) {
    return !(!e || e.nodeType !== 1 && e.nodeType !== 9 && e.nodeType !== 11);
  }
  function s(e) {
    var n = e, a = e;
    if (e.alternate) for (; n.return; ) n = n.return;
    else {
      e = n;
      do
        n = e, (n.flags & 4098) !== 0 && (a = n.return), e = n.return;
      while (e);
    }
    return n.tag === 3 ? a : null;
  }
  function c(e) {
    if (e.tag === 13) {
      var n = e.memoizedState;
      if (n === null && (e = e.alternate, e !== null && (n = e.memoizedState)), n !== null) return n.dehydrated;
    }
    return null;
  }
  function f(e) {
    if (e.tag === 31) {
      var n = e.memoizedState;
      if (n === null && (e = e.alternate, e !== null && (n = e.memoizedState)), n !== null) return n.dehydrated;
    }
    return null;
  }
  function g(e) {
    if (s(e) !== e)
      throw Error(u(188));
  }
  function h(e) {
    var n = e.alternate;
    if (!n) {
      if (n = s(e), n === null) throw Error(u(188));
      return n !== e ? null : e;
    }
    for (var a = e, o = n; ; ) {
      var d = a.return;
      if (d === null) break;
      var p = d.alternate;
      if (p === null) {
        if (o = d.return, o !== null) {
          a = o;
          continue;
        }
        break;
      }
      if (d.child === p.child) {
        for (p = d.child; p; ) {
          if (p === a) return g(d), e;
          if (p === o) return g(d), n;
          p = p.sibling;
        }
        throw Error(u(188));
      }
      if (a.return !== o.return) a = d, o = p;
      else {
        for (var x = !1, C = d.child; C; ) {
          if (C === a) {
            x = !0, a = d, o = p;
            break;
          }
          if (C === o) {
            x = !0, o = d, a = p;
            break;
          }
          C = C.sibling;
        }
        if (!x) {
          for (C = p.child; C; ) {
            if (C === a) {
              x = !0, a = p, o = d;
              break;
            }
            if (C === o) {
              x = !0, o = p, a = d;
              break;
            }
            C = C.sibling;
          }
          if (!x) throw Error(u(189));
        }
      }
      if (a.alternate !== o) throw Error(u(190));
    }
    if (a.tag !== 3) throw Error(u(188));
    return a.stateNode.current === a ? e : n;
  }
  function v(e) {
    var n = e.tag;
    if (n === 5 || n === 26 || n === 27 || n === 6) return e;
    for (e = e.child; e !== null; ) {
      if (n = v(e), n !== null) return n;
      e = e.sibling;
    }
    return null;
  }
  var y = Object.assign, m = Symbol.for("react.element"), b = Symbol.for("react.transitional.element"), _ = Symbol.for("react.portal"), A = Symbol.for("react.fragment"), w = Symbol.for("react.strict_mode"), E = Symbol.for("react.profiler"), M = Symbol.for("react.consumer"), S = Symbol.for("react.context"), T = Symbol.for("react.forward_ref"), N = Symbol.for("react.suspense"), R = Symbol.for("react.suspense_list"), q = Symbol.for("react.memo"), L = Symbol.for("react.lazy"), j = Symbol.for("react.activity"), Z = Symbol.for("react.memo_cache_sentinel"), Y = Symbol.iterator;
  function J(e) {
    return e === null || typeof e != "object" ? null : (e = Y && e[Y] || e["@@iterator"], typeof e == "function" ? e : null);
  }
  var Q = Symbol.for("react.client.reference");
  function D(e) {
    if (e == null) return null;
    if (typeof e == "function")
      return e.$$typeof === Q ? null : e.displayName || e.name || null;
    if (typeof e == "string") return e;
    switch (e) {
      case A:
        return "Fragment";
      case E:
        return "Profiler";
      case w:
        return "StrictMode";
      case N:
        return "Suspense";
      case R:
        return "SuspenseList";
      case j:
        return "Activity";
    }
    if (typeof e == "object")
      switch (e.$$typeof) {
        case _:
          return "Portal";
        case S:
          return e.displayName || "Context";
        case M:
          return (e._context.displayName || "Context") + ".Consumer";
        case T:
          var n = e.render;
          return e = e.displayName, e || (e = n.displayName || n.name || "", e = e !== "" ? "ForwardRef(" + e + ")" : "ForwardRef"), e;
        case q:
          return n = e.displayName || null, n !== null ? n : D(e.type) || "Memo";
        case L:
          n = e._payload, e = e._init;
          try {
            return D(e(n));
          } catch {
          }
      }
    return null;
  }
  var X = Array.isArray, z = r.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE, B = i.__DOM_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE, $ = {
    pending: !1,
    data: null,
    method: null,
    action: null
  }, U = [], ae = -1;
  function O(e) {
    return { current: e };
  }
  function G(e) {
    0 > ae || (e.current = U[ae], U[ae] = null, ae--);
  }
  function ie(e, n) {
    ae++, U[ae] = e.current, e.current = n;
  }
  var H = O(null), K = O(null), P = O(null), ue = O(null);
  function W(e, n) {
    switch (ie(P, n), ie(K, e), ie(H, null), n.nodeType) {
      case 9:
      case 11:
        e = (e = n.documentElement) && (e = e.namespaceURI) ? Sb(e) : 0;
        break;
      default:
        if (e = n.tagName, n = n.namespaceURI)
          n = Sb(n), e = Eb(n, e);
        else
          switch (e) {
            case "svg":
              e = 1;
              break;
            case "math":
              e = 2;
              break;
            default:
              e = 0;
          }
    }
    G(H), ie(H, e);
  }
  function se() {
    G(H), G(K), G(P);
  }
  function de(e) {
    e.memoizedState !== null && ie(ue, e);
    var n = H.current, a = Eb(n, e.type);
    n !== a && (ie(K, e), ie(H, a));
  }
  function ve(e) {
    K.current === e && (G(H), G(K)), ue.current === e && (G(ue), hu._currentValue = $);
  }
  var ye, he;
  function me(e) {
    if (ye === void 0)
      try {
        throw Error();
      } catch (a) {
        var n = a.stack.trim().match(/\n( *(at )?)/);
        ye = n && n[1] || "", he = -1 < a.stack.indexOf(`
    at`) ? " (<anonymous>)" : -1 < a.stack.indexOf("@") ? "@unknown:0:0" : "";
      }
    return `
` + ye + e + he;
  }
  var ge = !1;
  function Ae(e, n) {
    if (!e || ge) return "";
    ge = !0;
    var a = Error.prepareStackTrace;
    Error.prepareStackTrace = void 0;
    try {
      var o = {
        DetermineComponentFrameRoot: function() {
          try {
            if (n) {
              var fe = function() {
                throw Error();
              };
              if (Object.defineProperty(fe.prototype, "props", {
                set: function() {
                  throw Error();
                }
              }), typeof Reflect == "object" && Reflect.construct) {
                try {
                  Reflect.construct(fe, []);
                } catch (le) {
                  var ne = le;
                }
                Reflect.construct(e, [], fe);
              } else {
                try {
                  fe.call();
                } catch (le) {
                  ne = le;
                }
                e.call(fe.prototype);
              }
            } else {
              try {
                throw Error();
              } catch (le) {
                ne = le;
              }
              (fe = e()) && typeof fe.catch == "function" && fe.catch(function() {
              });
            }
          } catch (le) {
            if (le && ne && typeof le.stack == "string")
              return [le.stack, ne.stack];
          }
          return [null, null];
        }
      };
      o.DetermineComponentFrameRoot.displayName = "DetermineComponentFrameRoot";
      var d = Object.getOwnPropertyDescriptor(
        o.DetermineComponentFrameRoot,
        "name"
      );
      d && d.configurable && Object.defineProperty(
        o.DetermineComponentFrameRoot,
        "name",
        { value: "DetermineComponentFrameRoot" }
      );
      var p = o.DetermineComponentFrameRoot(), x = p[0], C = p[1];
      if (x && C) {
        var V = x.split(`
`), te = C.split(`
`);
        for (d = o = 0; o < V.length && !V[o].includes("DetermineComponentFrameRoot"); )
          o++;
        for (; d < te.length && !te[d].includes(
          "DetermineComponentFrameRoot"
        ); )
          d++;
        if (o === V.length || d === te.length)
          for (o = V.length - 1, d = te.length - 1; 1 <= o && 0 <= d && V[o] !== te[d]; )
            d--;
        for (; 1 <= o && 0 <= d; o--, d--)
          if (V[o] !== te[d]) {
            if (o !== 1 || d !== 1)
              do
                if (o--, d--, 0 > d || V[o] !== te[d]) {
                  var oe = `
` + V[o].replace(" at new ", " at ");
                  return e.displayName && oe.includes("<anonymous>") && (oe = oe.replace("<anonymous>", e.displayName)), oe;
                }
              while (1 <= o && 0 <= d);
            break;
          }
      }
    } finally {
      ge = !1, Error.prepareStackTrace = a;
    }
    return (a = e ? e.displayName || e.name : "") ? me(a) : "";
  }
  function xe(e, n) {
    switch (e.tag) {
      case 26:
      case 27:
      case 5:
        return me(e.type);
      case 16:
        return me("Lazy");
      case 13:
        return e.child !== n && n !== null ? me("Suspense Fallback") : me("Suspense");
      case 19:
        return me("SuspenseList");
      case 0:
      case 15:
        return Ae(e.type, !1);
      case 11:
        return Ae(e.type.render, !1);
      case 1:
        return Ae(e.type, !0);
      case 31:
        return me("Activity");
      default:
        return "";
    }
  }
  function Pe(e) {
    try {
      var n = "", a = null;
      do
        n += xe(e, a), a = e, e = e.return;
      while (e);
      return n;
    } catch (o) {
      return `
Error generating stack: ` + o.message + `
` + o.stack;
    }
  }
  var tt = Object.prototype.hasOwnProperty, xt = t.unstable_scheduleCallback, gt = t.unstable_cancelCallback, St = t.unstable_shouldYield, Ke = t.unstable_requestPaint, ke = t.unstable_now, vt = t.unstable_getCurrentPriorityLevel, qt = t.unstable_ImmediatePriority, Tt = t.unstable_UserBlockingPriority, ft = t.unstable_NormalPriority, tr = t.unstable_LowPriority, qn = t.unstable_IdlePriority, _i = t.log, va = t.unstable_setDisableYieldValue, Et = null, Je = null;
  function mn(e) {
    if (typeof _i == "function" && va(e), Je && typeof Je.setStrictMode == "function")
      try {
        Je.setStrictMode(Et, e);
      } catch {
      }
  }
  var Mt = Math.clz32 ? Math.clz32 : ws, Ss = Math.log, Es = Math.LN2;
  function ws(e) {
    return e >>>= 0, e === 0 ? 32 : 31 - (Ss(e) / Es | 0) | 0;
  }
  var pa = 256, ya = 262144, ma = 4194304;
  function Rn(e) {
    var n = e & 42;
    if (n !== 0) return n;
    switch (e & -e) {
      case 1:
        return 1;
      case 2:
        return 2;
      case 4:
        return 4;
      case 8:
        return 8;
      case 16:
        return 16;
      case 32:
        return 32;
      case 64:
        return 64;
      case 128:
        return 128;
      case 256:
      case 512:
      case 1024:
      case 2048:
      case 4096:
      case 8192:
      case 16384:
      case 32768:
      case 65536:
      case 131072:
        return e & 261888;
      case 262144:
      case 524288:
      case 1048576:
      case 2097152:
        return e & 3932160;
      case 4194304:
      case 8388608:
      case 16777216:
      case 33554432:
        return e & 62914560;
      case 67108864:
        return 67108864;
      case 134217728:
        return 134217728;
      case 268435456:
        return 268435456;
      case 536870912:
        return 536870912;
      case 1073741824:
        return 0;
      default:
        return e;
    }
  }
  function ba(e, n, a) {
    var o = e.pendingLanes;
    if (o === 0) return 0;
    var d = 0, p = e.suspendedLanes, x = e.pingedLanes;
    e = e.warmLanes;
    var C = o & 134217727;
    return C !== 0 ? (o = C & ~p, o !== 0 ? d = Rn(o) : (x &= C, x !== 0 ? d = Rn(x) : a || (a = C & ~e, a !== 0 && (d = Rn(a))))) : (C = o & ~p, C !== 0 ? d = Rn(C) : x !== 0 ? d = Rn(x) : a || (a = o & ~e, a !== 0 && (d = Rn(a)))), d === 0 ? 0 : n !== 0 && n !== d && (n & p) === 0 && (p = d & -d, a = n & -n, p >= a || p === 32 && (a & 4194048) !== 0) ? n : d;
  }
  function Hr(e, n) {
    return (e.pendingLanes & ~(e.suspendedLanes & ~e.pingedLanes) & n) === 0;
  }
  function As(e, n) {
    switch (e) {
      case 1:
      case 2:
      case 4:
      case 8:
      case 64:
        return n + 250;
      case 16:
      case 32:
      case 128:
      case 256:
      case 512:
      case 1024:
      case 2048:
      case 4096:
      case 8192:
      case 16384:
      case 32768:
      case 65536:
      case 131072:
      case 262144:
      case 524288:
      case 1048576:
      case 2097152:
        return n + 5e3;
      case 4194304:
      case 8388608:
      case 16777216:
      case 33554432:
        return -1;
      case 67108864:
      case 134217728:
      case 268435456:
      case 536870912:
      case 1073741824:
        return -1;
      default:
        return -1;
    }
  }
  function Fu() {
    var e = ma;
    return ma <<= 1, (ma & 62914560) === 0 && (ma = 4194304), e;
  }
  function xi(e) {
    for (var n = [], a = 0; 31 > a; a++) n.push(e);
    return n;
  }
  function jr(e, n) {
    e.pendingLanes |= n, n !== 268435456 && (e.suspendedLanes = 0, e.pingedLanes = 0, e.warmLanes = 0);
  }
  function Ts(e, n, a, o, d, p) {
    var x = e.pendingLanes;
    e.pendingLanes = a, e.suspendedLanes = 0, e.pingedLanes = 0, e.warmLanes = 0, e.expiredLanes &= a, e.entangledLanes &= a, e.errorRecoveryDisabledLanes &= a, e.shellSuspendCounter = 0;
    var C = e.entanglements, V = e.expirationTimes, te = e.hiddenUpdates;
    for (a = x & ~a; 0 < a; ) {
      var oe = 31 - Mt(a), fe = 1 << oe;
      C[oe] = 0, V[oe] = -1;
      var ne = te[oe];
      if (ne !== null)
        for (te[oe] = null, oe = 0; oe < ne.length; oe++) {
          var le = ne[oe];
          le !== null && (le.lane &= -536870913);
        }
      a &= ~fe;
    }
    o !== 0 && Ju(e, o, 0), p !== 0 && d === 0 && e.tag !== 0 && (e.suspendedLanes |= p & ~(x & ~n));
  }
  function Ju(e, n, a) {
    e.pendingLanes |= n, e.suspendedLanes &= ~n;
    var o = 31 - Mt(n);
    e.entangledLanes |= n, e.entanglements[o] = e.entanglements[o] | 1073741824 | a & 261930;
  }
  function Pu(e, n) {
    var a = e.entangledLanes |= n;
    for (e = e.entanglements; a; ) {
      var o = 31 - Mt(a), d = 1 << o;
      d & n | e[o] & n && (e[o] |= n), a &= ~d;
    }
  }
  function Wu(e, n) {
    var a = n & -n;
    return a = (a & 42) !== 0 ? 1 : Si(a), (a & (e.suspendedLanes | n)) !== 0 ? 0 : a;
  }
  function Si(e) {
    switch (e) {
      case 2:
        e = 1;
        break;
      case 8:
        e = 4;
        break;
      case 32:
        e = 16;
        break;
      case 256:
      case 512:
      case 1024:
      case 2048:
      case 4096:
      case 8192:
      case 16384:
      case 32768:
      case 65536:
      case 131072:
      case 262144:
      case 524288:
      case 1048576:
      case 2097152:
      case 4194304:
      case 8388608:
      case 16777216:
      case 33554432:
        e = 128;
        break;
      case 268435456:
        e = 134217728;
        break;
      default:
        e = 0;
    }
    return e;
  }
  function Ei(e) {
    return e &= -e, 2 < e ? 8 < e ? (e & 134217727) !== 0 ? 32 : 268435456 : 8 : 2;
  }
  function el() {
    var e = B.p;
    return e !== 0 ? e : (e = window.event, e === void 0 ? 32 : Zb(e.type));
  }
  function tl(e, n) {
    var a = B.p;
    try {
      return B.p = e, n();
    } finally {
      B.p = a;
    }
  }
  var bn = Math.random().toString(36).slice(2), dt = "__reactFiber$" + bn, wt = "__reactProps$" + bn, Nn = "__reactContainer$" + bn, _a = "__reactEvents$" + bn, nl = "__reactListeners$" + bn, Ms = "__reactHandles$" + bn, rl = "__reactResources$" + bn, Br = "__reactMarker$" + bn;
  function wi(e) {
    delete e[dt], delete e[wt], delete e[_a], delete e[nl], delete e[Ms];
  }
  function nr(e) {
    var n = e[dt];
    if (n) return n;
    for (var a = e.parentNode; a; ) {
      if (n = a[Nn] || a[dt]) {
        if (a = n.alternate, n.child !== null || a !== null && a.child !== null)
          for (e = Rb(e); e !== null; ) {
            if (a = e[dt]) return a;
            e = Rb(e);
          }
        return n;
      }
      e = a, a = e.parentNode;
    }
    return null;
  }
  function rr(e) {
    if (e = e[dt] || e[Nn]) {
      var n = e.tag;
      if (n === 5 || n === 6 || n === 13 || n === 31 || n === 26 || n === 27 || n === 3)
        return e;
    }
    return null;
  }
  function ar(e) {
    var n = e.tag;
    if (n === 5 || n === 26 || n === 27 || n === 6) return e.stateNode;
    throw Error(u(33));
  }
  function ir(e) {
    var n = e[rl];
    return n || (n = e[rl] = { hoistableStyles: /* @__PURE__ */ new Map(), hoistableScripts: /* @__PURE__ */ new Map() }), n;
  }
  function at(e) {
    e[Br] = !0;
  }
  var al = /* @__PURE__ */ new Set(), il = {};
  function zn(e, n) {
    ur(e, n), ur(e + "Capture", n);
  }
  function ur(e, n) {
    for (il[e] = n, e = 0; e < n.length; e++)
      al.add(n[e]);
  }
  var Cs = RegExp(
    "^[:A-Z_a-z\\u00C0-\\u00D6\\u00D8-\\u00F6\\u00F8-\\u02FF\\u0370-\\u037D\\u037F-\\u1FFF\\u200C-\\u200D\\u2070-\\u218F\\u2C00-\\u2FEF\\u3001-\\uD7FF\\uF900-\\uFDCF\\uFDF0-\\uFFFD][:A-Z_a-z\\u00C0-\\u00D6\\u00D8-\\u00F6\\u00F8-\\u02FF\\u0370-\\u037D\\u037F-\\u1FFF\\u200C-\\u200D\\u2070-\\u218F\\u2C00-\\u2FEF\\u3001-\\uD7FF\\uF900-\\uFDCF\\uFDF0-\\uFFFD\\-.0-9\\u00B7\\u0300-\\u036F\\u203F-\\u2040]*$"
  ), ul = {}, Ai = {};
  function qs(e) {
    return tt.call(Ai, e) ? !0 : tt.call(ul, e) ? !1 : Cs.test(e) ? Ai[e] = !0 : (ul[e] = !0, !1);
  }
  function xa(e, n, a) {
    if (qs(n))
      if (a === null) e.removeAttribute(n);
      else {
        switch (typeof a) {
          case "undefined":
          case "function":
          case "symbol":
            e.removeAttribute(n);
            return;
          case "boolean":
            var o = n.toLowerCase().slice(0, 5);
            if (o !== "data-" && o !== "aria-") {
              e.removeAttribute(n);
              return;
            }
        }
        e.setAttribute(n, "" + a);
      }
  }
  function Sa(e, n, a) {
    if (a === null) e.removeAttribute(n);
    else {
      switch (typeof a) {
        case "undefined":
        case "function":
        case "symbol":
        case "boolean":
          e.removeAttribute(n);
          return;
      }
      e.setAttribute(n, "" + a);
    }
  }
  function un(e, n, a, o) {
    if (o === null) e.removeAttribute(a);
    else {
      switch (typeof o) {
        case "undefined":
        case "function":
        case "symbol":
        case "boolean":
          e.removeAttribute(a);
          return;
      }
      e.setAttributeNS(n, a, "" + o);
    }
  }
  function Rt(e) {
    switch (typeof e) {
      case "bigint":
      case "boolean":
      case "number":
      case "string":
      case "undefined":
        return e;
      case "object":
        return e;
      default:
        return "";
    }
  }
  function ll(e) {
    var n = e.type;
    return (e = e.nodeName) && e.toLowerCase() === "input" && (n === "checkbox" || n === "radio");
  }
  function Rs(e, n, a) {
    var o = Object.getOwnPropertyDescriptor(
      e.constructor.prototype,
      n
    );
    if (!e.hasOwnProperty(n) && typeof o < "u" && typeof o.get == "function" && typeof o.set == "function") {
      var d = o.get, p = o.set;
      return Object.defineProperty(e, n, {
        configurable: !0,
        get: function() {
          return d.call(this);
        },
        set: function(x) {
          a = "" + x, p.call(this, x);
        }
      }), Object.defineProperty(e, n, {
        enumerable: o.enumerable
      }), {
        getValue: function() {
          return a;
        },
        setValue: function(x) {
          a = "" + x;
        },
        stopTracking: function() {
          e._valueTracker = null, delete e[n];
        }
      };
    }
  }
  function Ti(e) {
    if (!e._valueTracker) {
      var n = ll(e) ? "checked" : "value";
      e._valueTracker = Rs(
        e,
        n,
        "" + e[n]
      );
    }
  }
  function Ur(e) {
    if (!e) return !1;
    var n = e._valueTracker;
    if (!n) return !0;
    var a = n.getValue(), o = "";
    return e && (o = ll(e) ? e.checked ? "true" : "false" : e.value), e = o, e !== a ? (n.setValue(e), !0) : !1;
  }
  function Ea(e) {
    if (e = e || (typeof document < "u" ? document : void 0), typeof e > "u") return null;
    try {
      return e.activeElement || e.body;
    } catch {
      return e.body;
    }
  }
  var ol = /[\n"\\]/g;
  function Nt(e) {
    return e.replace(
      ol,
      function(n) {
        return "\\" + n.charCodeAt(0).toString(16) + " ";
      }
    );
  }
  function Mi(e, n, a, o, d, p, x, C) {
    e.name = "", x != null && typeof x != "function" && typeof x != "symbol" && typeof x != "boolean" ? e.type = x : e.removeAttribute("type"), n != null ? x === "number" ? (n === 0 && e.value === "" || e.value != n) && (e.value = "" + Rt(n)) : e.value !== "" + Rt(n) && (e.value = "" + Rt(n)) : x !== "submit" && x !== "reset" || e.removeAttribute("value"), n != null ? Ci(e, x, Rt(n)) : a != null ? Ci(e, x, Rt(a)) : o != null && e.removeAttribute("value"), d == null && p != null && (e.defaultChecked = !!p), d != null && (e.checked = d && typeof d != "function" && typeof d != "symbol"), C != null && typeof C != "function" && typeof C != "symbol" && typeof C != "boolean" ? e.name = "" + Rt(C) : e.removeAttribute("name");
  }
  function wa(e, n, a, o, d, p, x, C) {
    if (p != null && typeof p != "function" && typeof p != "symbol" && typeof p != "boolean" && (e.type = p), n != null || a != null) {
      if (!(p !== "submit" && p !== "reset" || n != null)) {
        Ti(e);
        return;
      }
      a = a != null ? "" + Rt(a) : "", n = n != null ? "" + Rt(n) : a, C || n === e.value || (e.value = n), e.defaultValue = n;
    }
    o = o ?? d, o = typeof o != "function" && typeof o != "symbol" && !!o, e.checked = C ? e.checked : !!o, e.defaultChecked = !!o, x != null && typeof x != "function" && typeof x != "symbol" && typeof x != "boolean" && (e.name = x), Ti(e);
  }
  function Ci(e, n, a) {
    n === "number" && Ea(e.ownerDocument) === e || e.defaultValue === "" + a || (e.defaultValue = "" + a);
  }
  function lr(e, n, a, o) {
    if (e = e.options, n) {
      n = {};
      for (var d = 0; d < a.length; d++)
        n["$" + a[d]] = !0;
      for (a = 0; a < e.length; a++)
        d = n.hasOwnProperty("$" + e[a].value), e[a].selected !== d && (e[a].selected = d), d && o && (e[a].defaultSelected = !0);
    } else {
      for (a = "" + Rt(a), n = null, d = 0; d < e.length; d++) {
        if (e[d].value === a) {
          e[d].selected = !0, o && (e[d].defaultSelected = !0);
          return;
        }
        n !== null || e[d].disabled || (n = e[d]);
      }
      n !== null && (n.selected = !0);
    }
  }
  function qi(e, n, a) {
    if (n != null && (n = "" + Rt(n), n !== e.value && (e.value = n), a == null)) {
      e.defaultValue !== n && (e.defaultValue = n);
      return;
    }
    e.defaultValue = a != null ? "" + Rt(a) : "";
  }
  function mm(e, n, a, o) {
    if (n == null) {
      if (o != null) {
        if (a != null) throw Error(u(92));
        if (X(o)) {
          if (1 < o.length) throw Error(u(93));
          o = o[0];
        }
        a = o;
      }
      a == null && (a = ""), n = a;
    }
    a = Rt(n), e.defaultValue = a, o = e.textContent, o === a && o !== "" && o !== null && (e.value = o), Ti(e);
  }
  function Aa(e, n) {
    if (n) {
      var a = e.firstChild;
      if (a && a === e.lastChild && a.nodeType === 3) {
        a.nodeValue = n;
        return;
      }
    }
    e.textContent = n;
  }
  var DC = new Set(
    "animationIterationCount aspectRatio borderImageOutset borderImageSlice borderImageWidth boxFlex boxFlexGroup boxOrdinalGroup columnCount columns flex flexGrow flexPositive flexShrink flexNegative flexOrder gridArea gridRow gridRowEnd gridRowSpan gridRowStart gridColumn gridColumnEnd gridColumnSpan gridColumnStart fontWeight lineClamp lineHeight opacity order orphans scale tabSize widows zIndex zoom fillOpacity floodOpacity stopOpacity strokeDasharray strokeDashoffset strokeMiterlimit strokeOpacity strokeWidth MozAnimationIterationCount MozBoxFlex MozBoxFlexGroup MozLineClamp msAnimationIterationCount msFlex msZoom msFlexGrow msFlexNegative msFlexOrder msFlexPositive msFlexShrink msGridColumn msGridColumnSpan msGridRow msGridRowSpan WebkitAnimationIterationCount WebkitBoxFlex WebKitBoxFlexGroup WebkitBoxOrdinalGroup WebkitColumnCount WebkitColumns WebkitFlex WebkitFlexGrow WebkitFlexPositive WebkitFlexShrink WebkitLineClamp".split(
      " "
    )
  );
  function bm(e, n, a) {
    var o = n.indexOf("--") === 0;
    a == null || typeof a == "boolean" || a === "" ? o ? e.setProperty(n, "") : n === "float" ? e.cssFloat = "" : e[n] = "" : o ? e.setProperty(n, a) : typeof a != "number" || a === 0 || DC.has(n) ? n === "float" ? e.cssFloat = a : e[n] = ("" + a).trim() : e[n] = a + "px";
  }
  function _m(e, n, a) {
    if (n != null && typeof n != "object")
      throw Error(u(62));
    if (e = e.style, a != null) {
      for (var o in a)
        !a.hasOwnProperty(o) || n != null && n.hasOwnProperty(o) || (o.indexOf("--") === 0 ? e.setProperty(o, "") : o === "float" ? e.cssFloat = "" : e[o] = "");
      for (var d in n)
        o = n[d], n.hasOwnProperty(d) && a[d] !== o && bm(e, d, o);
    } else
      for (var p in n)
        n.hasOwnProperty(p) && bm(e, p, n[p]);
  }
  function Ns(e) {
    if (e.indexOf("-") === -1) return !1;
    switch (e) {
      case "annotation-xml":
      case "color-profile":
      case "font-face":
      case "font-face-src":
      case "font-face-uri":
      case "font-face-format":
      case "font-face-name":
      case "missing-glyph":
        return !1;
      default:
        return !0;
    }
  }
  var LC = /* @__PURE__ */ new Map([
    ["acceptCharset", "accept-charset"],
    ["htmlFor", "for"],
    ["httpEquiv", "http-equiv"],
    ["crossOrigin", "crossorigin"],
    ["accentHeight", "accent-height"],
    ["alignmentBaseline", "alignment-baseline"],
    ["arabicForm", "arabic-form"],
    ["baselineShift", "baseline-shift"],
    ["capHeight", "cap-height"],
    ["clipPath", "clip-path"],
    ["clipRule", "clip-rule"],
    ["colorInterpolation", "color-interpolation"],
    ["colorInterpolationFilters", "color-interpolation-filters"],
    ["colorProfile", "color-profile"],
    ["colorRendering", "color-rendering"],
    ["dominantBaseline", "dominant-baseline"],
    ["enableBackground", "enable-background"],
    ["fillOpacity", "fill-opacity"],
    ["fillRule", "fill-rule"],
    ["floodColor", "flood-color"],
    ["floodOpacity", "flood-opacity"],
    ["fontFamily", "font-family"],
    ["fontSize", "font-size"],
    ["fontSizeAdjust", "font-size-adjust"],
    ["fontStretch", "font-stretch"],
    ["fontStyle", "font-style"],
    ["fontVariant", "font-variant"],
    ["fontWeight", "font-weight"],
    ["glyphName", "glyph-name"],
    ["glyphOrientationHorizontal", "glyph-orientation-horizontal"],
    ["glyphOrientationVertical", "glyph-orientation-vertical"],
    ["horizAdvX", "horiz-adv-x"],
    ["horizOriginX", "horiz-origin-x"],
    ["imageRendering", "image-rendering"],
    ["letterSpacing", "letter-spacing"],
    ["lightingColor", "lighting-color"],
    ["markerEnd", "marker-end"],
    ["markerMid", "marker-mid"],
    ["markerStart", "marker-start"],
    ["overlinePosition", "overline-position"],
    ["overlineThickness", "overline-thickness"],
    ["paintOrder", "paint-order"],
    ["panose-1", "panose-1"],
    ["pointerEvents", "pointer-events"],
    ["renderingIntent", "rendering-intent"],
    ["shapeRendering", "shape-rendering"],
    ["stopColor", "stop-color"],
    ["stopOpacity", "stop-opacity"],
    ["strikethroughPosition", "strikethrough-position"],
    ["strikethroughThickness", "strikethrough-thickness"],
    ["strokeDasharray", "stroke-dasharray"],
    ["strokeDashoffset", "stroke-dashoffset"],
    ["strokeLinecap", "stroke-linecap"],
    ["strokeLinejoin", "stroke-linejoin"],
    ["strokeMiterlimit", "stroke-miterlimit"],
    ["strokeOpacity", "stroke-opacity"],
    ["strokeWidth", "stroke-width"],
    ["textAnchor", "text-anchor"],
    ["textDecoration", "text-decoration"],
    ["textRendering", "text-rendering"],
    ["transformOrigin", "transform-origin"],
    ["underlinePosition", "underline-position"],
    ["underlineThickness", "underline-thickness"],
    ["unicodeBidi", "unicode-bidi"],
    ["unicodeRange", "unicode-range"],
    ["unitsPerEm", "units-per-em"],
    ["vAlphabetic", "v-alphabetic"],
    ["vHanging", "v-hanging"],
    ["vIdeographic", "v-ideographic"],
    ["vMathematical", "v-mathematical"],
    ["vectorEffect", "vector-effect"],
    ["vertAdvY", "vert-adv-y"],
    ["vertOriginX", "vert-origin-x"],
    ["vertOriginY", "vert-origin-y"],
    ["wordSpacing", "word-spacing"],
    ["writingMode", "writing-mode"],
    ["xmlnsXlink", "xmlns:xlink"],
    ["xHeight", "x-height"]
  ]), HC = /^[\u0000-\u001F ]*j[\r\n\t]*a[\r\n\t]*v[\r\n\t]*a[\r\n\t]*s[\r\n\t]*c[\r\n\t]*r[\r\n\t]*i[\r\n\t]*p[\r\n\t]*t[\r\n\t]*:/i;
  function sl(e) {
    return HC.test("" + e) ? "javascript:throw new Error('React has blocked a javascript: URL as a security precaution.')" : e;
  }
  function On() {
  }
  var zs = null;
  function Os(e) {
    return e = e.target || e.srcElement || window, e.correspondingUseElement && (e = e.correspondingUseElement), e.nodeType === 3 ? e.parentNode : e;
  }
  var Ta = null, Ma = null;
  function xm(e) {
    var n = rr(e);
    if (n && (e = n.stateNode)) {
      var a = e[wt] || null;
      e: switch (e = n.stateNode, n.type) {
        case "input":
          if (Mi(
            e,
            a.value,
            a.defaultValue,
            a.defaultValue,
            a.checked,
            a.defaultChecked,
            a.type,
            a.name
          ), n = a.name, a.type === "radio" && n != null) {
            for (a = e; a.parentNode; ) a = a.parentNode;
            for (a = a.querySelectorAll(
              'input[name="' + Nt(
                "" + n
              ) + '"][type="radio"]'
            ), n = 0; n < a.length; n++) {
              var o = a[n];
              if (o !== e && o.form === e.form) {
                var d = o[wt] || null;
                if (!d) throw Error(u(90));
                Mi(
                  o,
                  d.value,
                  d.defaultValue,
                  d.defaultValue,
                  d.checked,
                  d.defaultChecked,
                  d.type,
                  d.name
                );
              }
            }
            for (n = 0; n < a.length; n++)
              o = a[n], o.form === e.form && Ur(o);
          }
          break e;
        case "textarea":
          qi(e, a.value, a.defaultValue);
          break e;
        case "select":
          n = a.value, n != null && lr(e, !!a.multiple, n, !1);
      }
    }
  }
  var Ds = !1;
  function Sm(e, n, a) {
    if (Ds) return e(n, a);
    Ds = !0;
    try {
      var o = e(n);
      return o;
    } finally {
      if (Ds = !1, (Ta !== null || Ma !== null) && (Fl(), Ta && (n = Ta, e = Ma, Ma = Ta = null, xm(n), e)))
        for (n = 0; n < e.length; n++) xm(e[n]);
    }
  }
  function Ri(e, n) {
    var a = e.stateNode;
    if (a === null) return null;
    var o = a[wt] || null;
    if (o === null) return null;
    a = o[n];
    e: switch (n) {
      case "onClick":
      case "onClickCapture":
      case "onDoubleClick":
      case "onDoubleClickCapture":
      case "onMouseDown":
      case "onMouseDownCapture":
      case "onMouseMove":
      case "onMouseMoveCapture":
      case "onMouseUp":
      case "onMouseUpCapture":
      case "onMouseEnter":
        (o = !o.disabled) || (e = e.type, o = !(e === "button" || e === "input" || e === "select" || e === "textarea")), e = !o;
        break e;
      default:
        e = !1;
    }
    if (e) return null;
    if (a && typeof a != "function")
      throw Error(
        u(231, n, typeof a)
      );
    return a;
  }
  var Dn = !(typeof window > "u" || typeof window.document > "u" || typeof window.document.createElement > "u"), Ls = !1;
  if (Dn)
    try {
      var Ni = {};
      Object.defineProperty(Ni, "passive", {
        get: function() {
          Ls = !0;
        }
      }), window.addEventListener("test", Ni, Ni), window.removeEventListener("test", Ni, Ni);
    } catch {
      Ls = !1;
    }
  var or = null, Hs = null, cl = null;
  function Em() {
    if (cl) return cl;
    var e, n = Hs, a = n.length, o, d = "value" in or ? or.value : or.textContent, p = d.length;
    for (e = 0; e < a && n[e] === d[e]; e++) ;
    var x = a - e;
    for (o = 1; o <= x && n[a - o] === d[p - o]; o++) ;
    return cl = d.slice(e, 1 < o ? 1 - o : void 0);
  }
  function fl(e) {
    var n = e.keyCode;
    return "charCode" in e ? (e = e.charCode, e === 0 && n === 13 && (e = 13)) : e = n, e === 10 && (e = 13), 32 <= e || e === 13 ? e : 0;
  }
  function dl() {
    return !0;
  }
  function wm() {
    return !1;
  }
  function zt(e) {
    function n(a, o, d, p, x) {
      this._reactName = a, this._targetInst = d, this.type = o, this.nativeEvent = p, this.target = x, this.currentTarget = null;
      for (var C in e)
        e.hasOwnProperty(C) && (a = e[C], this[C] = a ? a(p) : p[C]);
      return this.isDefaultPrevented = (p.defaultPrevented != null ? p.defaultPrevented : p.returnValue === !1) ? dl : wm, this.isPropagationStopped = wm, this;
    }
    return y(n.prototype, {
      preventDefault: function() {
        this.defaultPrevented = !0;
        var a = this.nativeEvent;
        a && (a.preventDefault ? a.preventDefault() : typeof a.returnValue != "unknown" && (a.returnValue = !1), this.isDefaultPrevented = dl);
      },
      stopPropagation: function() {
        var a = this.nativeEvent;
        a && (a.stopPropagation ? a.stopPropagation() : typeof a.cancelBubble != "unknown" && (a.cancelBubble = !0), this.isPropagationStopped = dl);
      },
      persist: function() {
      },
      isPersistent: dl
    }), n;
  }
  var Gr = {
    eventPhase: 0,
    bubbles: 0,
    cancelable: 0,
    timeStamp: function(e) {
      return e.timeStamp || Date.now();
    },
    defaultPrevented: 0,
    isTrusted: 0
  }, hl = zt(Gr), zi = y({}, Gr, { view: 0, detail: 0 }), jC = zt(zi), js, Bs, Oi, gl = y({}, zi, {
    screenX: 0,
    screenY: 0,
    clientX: 0,
    clientY: 0,
    pageX: 0,
    pageY: 0,
    ctrlKey: 0,
    shiftKey: 0,
    altKey: 0,
    metaKey: 0,
    getModifierState: Gs,
    button: 0,
    buttons: 0,
    relatedTarget: function(e) {
      return e.relatedTarget === void 0 ? e.fromElement === e.srcElement ? e.toElement : e.fromElement : e.relatedTarget;
    },
    movementX: function(e) {
      return "movementX" in e ? e.movementX : (e !== Oi && (Oi && e.type === "mousemove" ? (js = e.screenX - Oi.screenX, Bs = e.screenY - Oi.screenY) : Bs = js = 0, Oi = e), js);
    },
    movementY: function(e) {
      return "movementY" in e ? e.movementY : Bs;
    }
  }), Am = zt(gl), BC = y({}, gl, { dataTransfer: 0 }), UC = zt(BC), GC = y({}, zi, { relatedTarget: 0 }), Us = zt(GC), VC = y({}, Gr, {
    animationName: 0,
    elapsedTime: 0,
    pseudoElement: 0
  }), YC = zt(VC), kC = y({}, Gr, {
    clipboardData: function(e) {
      return "clipboardData" in e ? e.clipboardData : window.clipboardData;
    }
  }), XC = zt(kC), IC = y({}, Gr, { data: 0 }), Tm = zt(IC), ZC = {
    Esc: "Escape",
    Spacebar: " ",
    Left: "ArrowLeft",
    Up: "ArrowUp",
    Right: "ArrowRight",
    Down: "ArrowDown",
    Del: "Delete",
    Win: "OS",
    Menu: "ContextMenu",
    Apps: "ContextMenu",
    Scroll: "ScrollLock",
    MozPrintableKey: "Unidentified"
  }, KC = {
    8: "Backspace",
    9: "Tab",
    12: "Clear",
    13: "Enter",
    16: "Shift",
    17: "Control",
    18: "Alt",
    19: "Pause",
    20: "CapsLock",
    27: "Escape",
    32: " ",
    33: "PageUp",
    34: "PageDown",
    35: "End",
    36: "Home",
    37: "ArrowLeft",
    38: "ArrowUp",
    39: "ArrowRight",
    40: "ArrowDown",
    45: "Insert",
    46: "Delete",
    112: "F1",
    113: "F2",
    114: "F3",
    115: "F4",
    116: "F5",
    117: "F6",
    118: "F7",
    119: "F8",
    120: "F9",
    121: "F10",
    122: "F11",
    123: "F12",
    144: "NumLock",
    145: "ScrollLock",
    224: "Meta"
  }, QC = {
    Alt: "altKey",
    Control: "ctrlKey",
    Meta: "metaKey",
    Shift: "shiftKey"
  };
  function $C(e) {
    var n = this.nativeEvent;
    return n.getModifierState ? n.getModifierState(e) : (e = QC[e]) ? !!n[e] : !1;
  }
  function Gs() {
    return $C;
  }
  var FC = y({}, zi, {
    key: function(e) {
      if (e.key) {
        var n = ZC[e.key] || e.key;
        if (n !== "Unidentified") return n;
      }
      return e.type === "keypress" ? (e = fl(e), e === 13 ? "Enter" : String.fromCharCode(e)) : e.type === "keydown" || e.type === "keyup" ? KC[e.keyCode] || "Unidentified" : "";
    },
    code: 0,
    location: 0,
    ctrlKey: 0,
    shiftKey: 0,
    altKey: 0,
    metaKey: 0,
    repeat: 0,
    locale: 0,
    getModifierState: Gs,
    charCode: function(e) {
      return e.type === "keypress" ? fl(e) : 0;
    },
    keyCode: function(e) {
      return e.type === "keydown" || e.type === "keyup" ? e.keyCode : 0;
    },
    which: function(e) {
      return e.type === "keypress" ? fl(e) : e.type === "keydown" || e.type === "keyup" ? e.keyCode : 0;
    }
  }), JC = zt(FC), PC = y({}, gl, {
    pointerId: 0,
    width: 0,
    height: 0,
    pressure: 0,
    tangentialPressure: 0,
    tiltX: 0,
    tiltY: 0,
    twist: 0,
    pointerType: 0,
    isPrimary: 0
  }), Mm = zt(PC), WC = y({}, zi, {
    touches: 0,
    targetTouches: 0,
    changedTouches: 0,
    altKey: 0,
    metaKey: 0,
    ctrlKey: 0,
    shiftKey: 0,
    getModifierState: Gs
  }), eq = zt(WC), tq = y({}, Gr, {
    propertyName: 0,
    elapsedTime: 0,
    pseudoElement: 0
  }), nq = zt(tq), rq = y({}, gl, {
    deltaX: function(e) {
      return "deltaX" in e ? e.deltaX : "wheelDeltaX" in e ? -e.wheelDeltaX : 0;
    },
    deltaY: function(e) {
      return "deltaY" in e ? e.deltaY : "wheelDeltaY" in e ? -e.wheelDeltaY : "wheelDelta" in e ? -e.wheelDelta : 0;
    },
    deltaZ: 0,
    deltaMode: 0
  }), aq = zt(rq), iq = y({}, Gr, {
    newState: 0,
    oldState: 0
  }), uq = zt(iq), lq = [9, 13, 27, 32], Vs = Dn && "CompositionEvent" in window, Di = null;
  Dn && "documentMode" in document && (Di = document.documentMode);
  var oq = Dn && "TextEvent" in window && !Di, Cm = Dn && (!Vs || Di && 8 < Di && 11 >= Di), qm = " ", Rm = !1;
  function Nm(e, n) {
    switch (e) {
      case "keyup":
        return lq.indexOf(n.keyCode) !== -1;
      case "keydown":
        return n.keyCode !== 229;
      case "keypress":
      case "mousedown":
      case "focusout":
        return !0;
      default:
        return !1;
    }
  }
  function zm(e) {
    return e = e.detail, typeof e == "object" && "data" in e ? e.data : null;
  }
  var Ca = !1;
  function sq(e, n) {
    switch (e) {
      case "compositionend":
        return zm(n);
      case "keypress":
        return n.which !== 32 ? null : (Rm = !0, qm);
      case "textInput":
        return e = n.data, e === qm && Rm ? null : e;
      default:
        return null;
    }
  }
  function cq(e, n) {
    if (Ca)
      return e === "compositionend" || !Vs && Nm(e, n) ? (e = Em(), cl = Hs = or = null, Ca = !1, e) : null;
    switch (e) {
      case "paste":
        return null;
      case "keypress":
        if (!(n.ctrlKey || n.altKey || n.metaKey) || n.ctrlKey && n.altKey) {
          if (n.char && 1 < n.char.length)
            return n.char;
          if (n.which) return String.fromCharCode(n.which);
        }
        return null;
      case "compositionend":
        return Cm && n.locale !== "ko" ? null : n.data;
      default:
        return null;
    }
  }
  var fq = {
    color: !0,
    date: !0,
    datetime: !0,
    "datetime-local": !0,
    email: !0,
    month: !0,
    number: !0,
    password: !0,
    range: !0,
    search: !0,
    tel: !0,
    text: !0,
    time: !0,
    url: !0,
    week: !0
  };
  function Om(e) {
    var n = e && e.nodeName && e.nodeName.toLowerCase();
    return n === "input" ? !!fq[e.type] : n === "textarea";
  }
  function Dm(e, n, a, o) {
    Ta ? Ma ? Ma.push(o) : Ma = [o] : Ta = o, n = ro(n, "onChange"), 0 < n.length && (a = new hl(
      "onChange",
      "change",
      null,
      a,
      o
    ), e.push({ event: a, listeners: n }));
  }
  var Li = null, Hi = null;
  function dq(e) {
    pb(e, 0);
  }
  function vl(e) {
    var n = ar(e);
    if (Ur(n)) return e;
  }
  function Lm(e, n) {
    if (e === "change") return n;
  }
  var Hm = !1;
  if (Dn) {
    var Ys;
    if (Dn) {
      var ks = "oninput" in document;
      if (!ks) {
        var jm = document.createElement("div");
        jm.setAttribute("oninput", "return;"), ks = typeof jm.oninput == "function";
      }
      Ys = ks;
    } else Ys = !1;
    Hm = Ys && (!document.documentMode || 9 < document.documentMode);
  }
  function Bm() {
    Li && (Li.detachEvent("onpropertychange", Um), Hi = Li = null);
  }
  function Um(e) {
    if (e.propertyName === "value" && vl(Hi)) {
      var n = [];
      Dm(
        n,
        Hi,
        e,
        Os(e)
      ), Sm(dq, n);
    }
  }
  function hq(e, n, a) {
    e === "focusin" ? (Bm(), Li = n, Hi = a, Li.attachEvent("onpropertychange", Um)) : e === "focusout" && Bm();
  }
  function gq(e) {
    if (e === "selectionchange" || e === "keyup" || e === "keydown")
      return vl(Hi);
  }
  function vq(e, n) {
    if (e === "click") return vl(n);
  }
  function pq(e, n) {
    if (e === "input" || e === "change")
      return vl(n);
  }
  function yq(e, n) {
    return e === n && (e !== 0 || 1 / e === 1 / n) || e !== e && n !== n;
  }
  var Gt = typeof Object.is == "function" ? Object.is : yq;
  function ji(e, n) {
    if (Gt(e, n)) return !0;
    if (typeof e != "object" || e === null || typeof n != "object" || n === null)
      return !1;
    var a = Object.keys(e), o = Object.keys(n);
    if (a.length !== o.length) return !1;
    for (o = 0; o < a.length; o++) {
      var d = a[o];
      if (!tt.call(n, d) || !Gt(e[d], n[d]))
        return !1;
    }
    return !0;
  }
  function Gm(e) {
    for (; e && e.firstChild; ) e = e.firstChild;
    return e;
  }
  function Vm(e, n) {
    var a = Gm(e);
    e = 0;
    for (var o; a; ) {
      if (a.nodeType === 3) {
        if (o = e + a.textContent.length, e <= n && o >= n)
          return { node: a, offset: n - e };
        e = o;
      }
      e: {
        for (; a; ) {
          if (a.nextSibling) {
            a = a.nextSibling;
            break e;
          }
          a = a.parentNode;
        }
        a = void 0;
      }
      a = Gm(a);
    }
  }
  function Ym(e, n) {
    return e && n ? e === n ? !0 : e && e.nodeType === 3 ? !1 : n && n.nodeType === 3 ? Ym(e, n.parentNode) : "contains" in e ? e.contains(n) : e.compareDocumentPosition ? !!(e.compareDocumentPosition(n) & 16) : !1 : !1;
  }
  function km(e) {
    e = e != null && e.ownerDocument != null && e.ownerDocument.defaultView != null ? e.ownerDocument.defaultView : window;
    for (var n = Ea(e.document); n instanceof e.HTMLIFrameElement; ) {
      try {
        var a = typeof n.contentWindow.location.href == "string";
      } catch {
        a = !1;
      }
      if (a) e = n.contentWindow;
      else break;
      n = Ea(e.document);
    }
    return n;
  }
  function Xs(e) {
    var n = e && e.nodeName && e.nodeName.toLowerCase();
    return n && (n === "input" && (e.type === "text" || e.type === "search" || e.type === "tel" || e.type === "url" || e.type === "password") || n === "textarea" || e.contentEditable === "true");
  }
  var mq = Dn && "documentMode" in document && 11 >= document.documentMode, qa = null, Is = null, Bi = null, Zs = !1;
  function Xm(e, n, a) {
    var o = a.window === a ? a.document : a.nodeType === 9 ? a : a.ownerDocument;
    Zs || qa == null || qa !== Ea(o) || (o = qa, "selectionStart" in o && Xs(o) ? o = { start: o.selectionStart, end: o.selectionEnd } : (o = (o.ownerDocument && o.ownerDocument.defaultView || window).getSelection(), o = {
      anchorNode: o.anchorNode,
      anchorOffset: o.anchorOffset,
      focusNode: o.focusNode,
      focusOffset: o.focusOffset
    }), Bi && ji(Bi, o) || (Bi = o, o = ro(Is, "onSelect"), 0 < o.length && (n = new hl(
      "onSelect",
      "select",
      null,
      n,
      a
    ), e.push({ event: n, listeners: o }), n.target = qa)));
  }
  function Vr(e, n) {
    var a = {};
    return a[e.toLowerCase()] = n.toLowerCase(), a["Webkit" + e] = "webkit" + n, a["Moz" + e] = "moz" + n, a;
  }
  var Ra = {
    animationend: Vr("Animation", "AnimationEnd"),
    animationiteration: Vr("Animation", "AnimationIteration"),
    animationstart: Vr("Animation", "AnimationStart"),
    transitionrun: Vr("Transition", "TransitionRun"),
    transitionstart: Vr("Transition", "TransitionStart"),
    transitioncancel: Vr("Transition", "TransitionCancel"),
    transitionend: Vr("Transition", "TransitionEnd")
  }, Ks = {}, Im = {};
  Dn && (Im = document.createElement("div").style, "AnimationEvent" in window || (delete Ra.animationend.animation, delete Ra.animationiteration.animation, delete Ra.animationstart.animation), "TransitionEvent" in window || delete Ra.transitionend.transition);
  function Yr(e) {
    if (Ks[e]) return Ks[e];
    if (!Ra[e]) return e;
    var n = Ra[e], a;
    for (a in n)
      if (n.hasOwnProperty(a) && a in Im)
        return Ks[e] = n[a];
    return e;
  }
  var Zm = Yr("animationend"), Km = Yr("animationiteration"), Qm = Yr("animationstart"), bq = Yr("transitionrun"), _q = Yr("transitionstart"), xq = Yr("transitioncancel"), $m = Yr("transitionend"), Fm = /* @__PURE__ */ new Map(), Qs = "abort auxClick beforeToggle cancel canPlay canPlayThrough click close contextMenu copy cut drag dragEnd dragEnter dragExit dragLeave dragOver dragStart drop durationChange emptied encrypted ended error gotPointerCapture input invalid keyDown keyPress keyUp load loadedData loadedMetadata loadStart lostPointerCapture mouseDown mouseMove mouseOut mouseOver mouseUp paste pause play playing pointerCancel pointerDown pointerMove pointerOut pointerOver pointerUp progress rateChange reset resize seeked seeking stalled submit suspend timeUpdate touchCancel touchEnd touchStart volumeChange scroll toggle touchMove waiting wheel".split(
    " "
  );
  Qs.push("scrollEnd");
  function ln(e, n) {
    Fm.set(e, n), zn(n, [e]);
  }
  var pl = typeof reportError == "function" ? reportError : function(e) {
    if (typeof window == "object" && typeof window.ErrorEvent == "function") {
      var n = new window.ErrorEvent("error", {
        bubbles: !0,
        cancelable: !0,
        message: typeof e == "object" && e !== null && typeof e.message == "string" ? String(e.message) : String(e),
        error: e
      });
      if (!window.dispatchEvent(n)) return;
    } else if (typeof process == "object" && typeof process.emit == "function") {
      process.emit("uncaughtException", e);
      return;
    }
    console.error(e);
  }, $t = [], Na = 0, $s = 0;
  function yl() {
    for (var e = Na, n = $s = Na = 0; n < e; ) {
      var a = $t[n];
      $t[n++] = null;
      var o = $t[n];
      $t[n++] = null;
      var d = $t[n];
      $t[n++] = null;
      var p = $t[n];
      if ($t[n++] = null, o !== null && d !== null) {
        var x = o.pending;
        x === null ? d.next = d : (d.next = x.next, x.next = d), o.pending = d;
      }
      p !== 0 && Jm(a, d, p);
    }
  }
  function ml(e, n, a, o) {
    $t[Na++] = e, $t[Na++] = n, $t[Na++] = a, $t[Na++] = o, $s |= o, e.lanes |= o, e = e.alternate, e !== null && (e.lanes |= o);
  }
  function Fs(e, n, a, o) {
    return ml(e, n, a, o), bl(e);
  }
  function kr(e, n) {
    return ml(e, null, null, n), bl(e);
  }
  function Jm(e, n, a) {
    e.lanes |= a;
    var o = e.alternate;
    o !== null && (o.lanes |= a);
    for (var d = !1, p = e.return; p !== null; )
      p.childLanes |= a, o = p.alternate, o !== null && (o.childLanes |= a), p.tag === 22 && (e = p.stateNode, e === null || e._visibility & 1 || (d = !0)), e = p, p = p.return;
    return e.tag === 3 ? (p = e.stateNode, d && n !== null && (d = 31 - Mt(a), e = p.hiddenUpdates, o = e[d], o === null ? e[d] = [n] : o.push(n), n.lane = a | 536870912), p) : null;
  }
  function bl(e) {
    if (50 < uu)
      throw uu = 0, uf = null, Error(u(185));
    for (var n = e.return; n !== null; )
      e = n, n = e.return;
    return e.tag === 3 ? e.stateNode : null;
  }
  var za = {};
  function Sq(e, n, a, o) {
    this.tag = e, this.key = a, this.sibling = this.child = this.return = this.stateNode = this.type = this.elementType = null, this.index = 0, this.refCleanup = this.ref = null, this.pendingProps = n, this.dependencies = this.memoizedState = this.updateQueue = this.memoizedProps = null, this.mode = o, this.subtreeFlags = this.flags = 0, this.deletions = null, this.childLanes = this.lanes = 0, this.alternate = null;
  }
  function Vt(e, n, a, o) {
    return new Sq(e, n, a, o);
  }
  function Js(e) {
    return e = e.prototype, !(!e || !e.isReactComponent);
  }
  function Ln(e, n) {
    var a = e.alternate;
    return a === null ? (a = Vt(
      e.tag,
      n,
      e.key,
      e.mode
    ), a.elementType = e.elementType, a.type = e.type, a.stateNode = e.stateNode, a.alternate = e, e.alternate = a) : (a.pendingProps = n, a.type = e.type, a.flags = 0, a.subtreeFlags = 0, a.deletions = null), a.flags = e.flags & 65011712, a.childLanes = e.childLanes, a.lanes = e.lanes, a.child = e.child, a.memoizedProps = e.memoizedProps, a.memoizedState = e.memoizedState, a.updateQueue = e.updateQueue, n = e.dependencies, a.dependencies = n === null ? null : { lanes: n.lanes, firstContext: n.firstContext }, a.sibling = e.sibling, a.index = e.index, a.ref = e.ref, a.refCleanup = e.refCleanup, a;
  }
  function Pm(e, n) {
    e.flags &= 65011714;
    var a = e.alternate;
    return a === null ? (e.childLanes = 0, e.lanes = n, e.child = null, e.subtreeFlags = 0, e.memoizedProps = null, e.memoizedState = null, e.updateQueue = null, e.dependencies = null, e.stateNode = null) : (e.childLanes = a.childLanes, e.lanes = a.lanes, e.child = a.child, e.subtreeFlags = 0, e.deletions = null, e.memoizedProps = a.memoizedProps, e.memoizedState = a.memoizedState, e.updateQueue = a.updateQueue, e.type = a.type, n = a.dependencies, e.dependencies = n === null ? null : {
      lanes: n.lanes,
      firstContext: n.firstContext
    }), e;
  }
  function _l(e, n, a, o, d, p) {
    var x = 0;
    if (o = e, typeof e == "function") Js(e) && (x = 1);
    else if (typeof e == "string")
      x = MR(
        e,
        a,
        H.current
      ) ? 26 : e === "html" || e === "head" || e === "body" ? 27 : 5;
    else
      e: switch (e) {
        case j:
          return e = Vt(31, a, n, d), e.elementType = j, e.lanes = p, e;
        case A:
          return Xr(a.children, d, p, n);
        case w:
          x = 8, d |= 24;
          break;
        case E:
          return e = Vt(12, a, n, d | 2), e.elementType = E, e.lanes = p, e;
        case N:
          return e = Vt(13, a, n, d), e.elementType = N, e.lanes = p, e;
        case R:
          return e = Vt(19, a, n, d), e.elementType = R, e.lanes = p, e;
        default:
          if (typeof e == "object" && e !== null)
            switch (e.$$typeof) {
              case S:
                x = 10;
                break e;
              case M:
                x = 9;
                break e;
              case T:
                x = 11;
                break e;
              case q:
                x = 14;
                break e;
              case L:
                x = 16, o = null;
                break e;
            }
          x = 29, a = Error(
            u(130, e === null ? "null" : typeof e, "")
          ), o = null;
      }
    return n = Vt(x, a, n, d), n.elementType = e, n.type = o, n.lanes = p, n;
  }
  function Xr(e, n, a, o) {
    return e = Vt(7, e, o, n), e.lanes = a, e;
  }
  function Ps(e, n, a) {
    return e = Vt(6, e, null, n), e.lanes = a, e;
  }
  function Wm(e) {
    var n = Vt(18, null, null, 0);
    return n.stateNode = e, n;
  }
  function Ws(e, n, a) {
    return n = Vt(
      4,
      e.children !== null ? e.children : [],
      e.key,
      n
    ), n.lanes = a, n.stateNode = {
      containerInfo: e.containerInfo,
      pendingChildren: null,
      implementation: e.implementation
    }, n;
  }
  var e0 = /* @__PURE__ */ new WeakMap();
  function Ft(e, n) {
    if (typeof e == "object" && e !== null) {
      var a = e0.get(e);
      return a !== void 0 ? a : (n = {
        value: e,
        source: n,
        stack: Pe(n)
      }, e0.set(e, n), n);
    }
    return {
      value: e,
      source: n,
      stack: Pe(n)
    };
  }
  var Oa = [], Da = 0, xl = null, Ui = 0, Jt = [], Pt = 0, sr = null, _n = 1, xn = "";
  function Hn(e, n) {
    Oa[Da++] = Ui, Oa[Da++] = xl, xl = e, Ui = n;
  }
  function t0(e, n, a) {
    Jt[Pt++] = _n, Jt[Pt++] = xn, Jt[Pt++] = sr, sr = e;
    var o = _n;
    e = xn;
    var d = 32 - Mt(o) - 1;
    o &= ~(1 << d), a += 1;
    var p = 32 - Mt(n) + d;
    if (30 < p) {
      var x = d - d % 5;
      p = (o & (1 << x) - 1).toString(32), o >>= x, d -= x, _n = 1 << 32 - Mt(n) + d | a << d | o, xn = p + e;
    } else
      _n = 1 << p | a << d | o, xn = e;
  }
  function ec(e) {
    e.return !== null && (Hn(e, 1), t0(e, 1, 0));
  }
  function tc(e) {
    for (; e === xl; )
      xl = Oa[--Da], Oa[Da] = null, Ui = Oa[--Da], Oa[Da] = null;
    for (; e === sr; )
      sr = Jt[--Pt], Jt[Pt] = null, xn = Jt[--Pt], Jt[Pt] = null, _n = Jt[--Pt], Jt[Pt] = null;
  }
  function n0(e, n) {
    Jt[Pt++] = _n, Jt[Pt++] = xn, Jt[Pt++] = sr, _n = n.id, xn = n.overflow, sr = e;
  }
  var yt = null, Xe = null, ze = !1, cr = null, Wt = !1, nc = Error(u(519));
  function fr(e) {
    var n = Error(
      u(
        418,
        1 < arguments.length && arguments[1] !== void 0 && arguments[1] ? "text" : "HTML",
        ""
      )
    );
    throw Gi(Ft(n, e)), nc;
  }
  function r0(e) {
    var n = e.stateNode, a = e.type, o = e.memoizedProps;
    switch (n[dt] = e, n[wt] = o, a) {
      case "dialog":
        qe("cancel", n), qe("close", n);
        break;
      case "iframe":
      case "object":
      case "embed":
        qe("load", n);
        break;
      case "video":
      case "audio":
        for (a = 0; a < ou.length; a++)
          qe(ou[a], n);
        break;
      case "source":
        qe("error", n);
        break;
      case "img":
      case "image":
      case "link":
        qe("error", n), qe("load", n);
        break;
      case "details":
        qe("toggle", n);
        break;
      case "input":
        qe("invalid", n), wa(
          n,
          o.value,
          o.defaultValue,
          o.checked,
          o.defaultChecked,
          o.type,
          o.name,
          !0
        );
        break;
      case "select":
        qe("invalid", n);
        break;
      case "textarea":
        qe("invalid", n), mm(n, o.value, o.defaultValue, o.children);
    }
    a = o.children, typeof a != "string" && typeof a != "number" && typeof a != "bigint" || n.textContent === "" + a || o.suppressHydrationWarning === !0 || _b(n.textContent, a) ? (o.popover != null && (qe("beforetoggle", n), qe("toggle", n)), o.onScroll != null && qe("scroll", n), o.onScrollEnd != null && qe("scrollend", n), o.onClick != null && (n.onclick = On), n = !0) : n = !1, n || fr(e, !0);
  }
  function a0(e) {
    for (yt = e.return; yt; )
      switch (yt.tag) {
        case 5:
        case 31:
        case 13:
          Wt = !1;
          return;
        case 27:
        case 3:
          Wt = !0;
          return;
        default:
          yt = yt.return;
      }
  }
  function La(e) {
    if (e !== yt) return !1;
    if (!ze) return a0(e), ze = !0, !1;
    var n = e.tag, a;
    if ((a = n !== 3 && n !== 27) && ((a = n === 5) && (a = e.type, a = !(a !== "form" && a !== "button") || xf(e.type, e.memoizedProps)), a = !a), a && Xe && fr(e), a0(e), n === 13) {
      if (e = e.memoizedState, e = e !== null ? e.dehydrated : null, !e) throw Error(u(317));
      Xe = qb(e);
    } else if (n === 31) {
      if (e = e.memoizedState, e = e !== null ? e.dehydrated : null, !e) throw Error(u(317));
      Xe = qb(e);
    } else
      n === 27 ? (n = Xe, Ar(e.type) ? (e = Tf, Tf = null, Xe = e) : Xe = n) : Xe = yt ? tn(e.stateNode.nextSibling) : null;
    return !0;
  }
  function Ir() {
    Xe = yt = null, ze = !1;
  }
  function rc() {
    var e = cr;
    return e !== null && (Ht === null ? Ht = e : Ht.push.apply(
      Ht,
      e
    ), cr = null), e;
  }
  function Gi(e) {
    cr === null ? cr = [e] : cr.push(e);
  }
  var ac = O(null), Zr = null, jn = null;
  function dr(e, n, a) {
    ie(ac, n._currentValue), n._currentValue = a;
  }
  function Bn(e) {
    e._currentValue = ac.current, G(ac);
  }
  function ic(e, n, a) {
    for (; e !== null; ) {
      var o = e.alternate;
      if ((e.childLanes & n) !== n ? (e.childLanes |= n, o !== null && (o.childLanes |= n)) : o !== null && (o.childLanes & n) !== n && (o.childLanes |= n), e === a) break;
      e = e.return;
    }
  }
  function uc(e, n, a, o) {
    var d = e.child;
    for (d !== null && (d.return = e); d !== null; ) {
      var p = d.dependencies;
      if (p !== null) {
        var x = d.child;
        p = p.firstContext;
        e: for (; p !== null; ) {
          var C = p;
          p = d;
          for (var V = 0; V < n.length; V++)
            if (C.context === n[V]) {
              p.lanes |= a, C = p.alternate, C !== null && (C.lanes |= a), ic(
                p.return,
                a,
                e
              ), o || (x = null);
              break e;
            }
          p = C.next;
        }
      } else if (d.tag === 18) {
        if (x = d.return, x === null) throw Error(u(341));
        x.lanes |= a, p = x.alternate, p !== null && (p.lanes |= a), ic(x, a, e), x = null;
      } else x = d.child;
      if (x !== null) x.return = d;
      else
        for (x = d; x !== null; ) {
          if (x === e) {
            x = null;
            break;
          }
          if (d = x.sibling, d !== null) {
            d.return = x.return, x = d;
            break;
          }
          x = x.return;
        }
      d = x;
    }
  }
  function Ha(e, n, a, o) {
    e = null;
    for (var d = n, p = !1; d !== null; ) {
      if (!p) {
        if ((d.flags & 524288) !== 0) p = !0;
        else if ((d.flags & 262144) !== 0) break;
      }
      if (d.tag === 10) {
        var x = d.alternate;
        if (x === null) throw Error(u(387));
        if (x = x.memoizedProps, x !== null) {
          var C = d.type;
          Gt(d.pendingProps.value, x.value) || (e !== null ? e.push(C) : e = [C]);
        }
      } else if (d === ue.current) {
        if (x = d.alternate, x === null) throw Error(u(387));
        x.memoizedState.memoizedState !== d.memoizedState.memoizedState && (e !== null ? e.push(hu) : e = [hu]);
      }
      d = d.return;
    }
    e !== null && uc(
      n,
      e,
      a,
      o
    ), n.flags |= 262144;
  }
  function Sl(e) {
    for (e = e.firstContext; e !== null; ) {
      if (!Gt(
        e.context._currentValue,
        e.memoizedValue
      ))
        return !0;
      e = e.next;
    }
    return !1;
  }
  function Kr(e) {
    Zr = e, jn = null, e = e.dependencies, e !== null && (e.firstContext = null);
  }
  function mt(e) {
    return i0(Zr, e);
  }
  function El(e, n) {
    return Zr === null && Kr(e), i0(e, n);
  }
  function i0(e, n) {
    var a = n._currentValue;
    if (n = { context: n, memoizedValue: a, next: null }, jn === null) {
      if (e === null) throw Error(u(308));
      jn = n, e.dependencies = { lanes: 0, firstContext: n }, e.flags |= 524288;
    } else jn = jn.next = n;
    return a;
  }
  var Eq = typeof AbortController < "u" ? AbortController : function() {
    var e = [], n = this.signal = {
      aborted: !1,
      addEventListener: function(a, o) {
        e.push(o);
      }
    };
    this.abort = function() {
      n.aborted = !0, e.forEach(function(a) {
        return a();
      });
    };
  }, wq = t.unstable_scheduleCallback, Aq = t.unstable_NormalPriority, it = {
    $$typeof: S,
    Consumer: null,
    Provider: null,
    _currentValue: null,
    _currentValue2: null,
    _threadCount: 0
  };
  function lc() {
    return {
      controller: new Eq(),
      data: /* @__PURE__ */ new Map(),
      refCount: 0
    };
  }
  function Vi(e) {
    e.refCount--, e.refCount === 0 && wq(Aq, function() {
      e.controller.abort();
    });
  }
  var Yi = null, oc = 0, ja = 0, Ba = null;
  function Tq(e, n) {
    if (Yi === null) {
      var a = Yi = [];
      oc = 0, ja = df(), Ba = {
        status: "pending",
        value: void 0,
        then: function(o) {
          a.push(o);
        }
      };
    }
    return oc++, n.then(u0, u0), n;
  }
  function u0() {
    if (--oc === 0 && Yi !== null) {
      Ba !== null && (Ba.status = "fulfilled");
      var e = Yi;
      Yi = null, ja = 0, Ba = null;
      for (var n = 0; n < e.length; n++) (0, e[n])();
    }
  }
  function Mq(e, n) {
    var a = [], o = {
      status: "pending",
      value: null,
      reason: null,
      then: function(d) {
        a.push(d);
      }
    };
    return e.then(
      function() {
        o.status = "fulfilled", o.value = n;
        for (var d = 0; d < a.length; d++) (0, a[d])(n);
      },
      function(d) {
        for (o.status = "rejected", o.reason = d, d = 0; d < a.length; d++)
          (0, a[d])(void 0);
      }
    ), o;
  }
  var l0 = z.S;
  z.S = function(e, n) {
    X1 = ke(), typeof n == "object" && n !== null && typeof n.then == "function" && Tq(e, n), l0 !== null && l0(e, n);
  };
  var Qr = O(null);
  function sc() {
    var e = Qr.current;
    return e !== null ? e : Ye.pooledCache;
  }
  function wl(e, n) {
    n === null ? ie(Qr, Qr.current) : ie(Qr, n.pool);
  }
  function o0() {
    var e = sc();
    return e === null ? null : { parent: it._currentValue, pool: e };
  }
  var Ua = Error(u(460)), cc = Error(u(474)), Al = Error(u(542)), Tl = { then: function() {
  } };
  function s0(e) {
    return e = e.status, e === "fulfilled" || e === "rejected";
  }
  function c0(e, n, a) {
    switch (a = e[a], a === void 0 ? e.push(n) : a !== n && (n.then(On, On), n = a), n.status) {
      case "fulfilled":
        return n.value;
      case "rejected":
        throw e = n.reason, d0(e), e;
      default:
        if (typeof n.status == "string") n.then(On, On);
        else {
          if (e = Ye, e !== null && 100 < e.shellSuspendCounter)
            throw Error(u(482));
          e = n, e.status = "pending", e.then(
            function(o) {
              if (n.status === "pending") {
                var d = n;
                d.status = "fulfilled", d.value = o;
              }
            },
            function(o) {
              if (n.status === "pending") {
                var d = n;
                d.status = "rejected", d.reason = o;
              }
            }
          );
        }
        switch (n.status) {
          case "fulfilled":
            return n.value;
          case "rejected":
            throw e = n.reason, d0(e), e;
        }
        throw Fr = n, Ua;
    }
  }
  function $r(e) {
    try {
      var n = e._init;
      return n(e._payload);
    } catch (a) {
      throw a !== null && typeof a == "object" && typeof a.then == "function" ? (Fr = a, Ua) : a;
    }
  }
  var Fr = null;
  function f0() {
    if (Fr === null) throw Error(u(459));
    var e = Fr;
    return Fr = null, e;
  }
  function d0(e) {
    if (e === Ua || e === Al)
      throw Error(u(483));
  }
  var Ga = null, ki = 0;
  function Ml(e) {
    var n = ki;
    return ki += 1, Ga === null && (Ga = []), c0(Ga, e, n);
  }
  function Xi(e, n) {
    n = n.props.ref, e.ref = n !== void 0 ? n : null;
  }
  function Cl(e, n) {
    throw n.$$typeof === m ? Error(u(525)) : (e = Object.prototype.toString.call(n), Error(
      u(
        31,
        e === "[object Object]" ? "object with keys {" + Object.keys(n).join(", ") + "}" : e
      )
    ));
  }
  function h0(e) {
    function n(F, I) {
      if (e) {
        var ee = F.deletions;
        ee === null ? (F.deletions = [I], F.flags |= 16) : ee.push(I);
      }
    }
    function a(F, I) {
      if (!e) return null;
      for (; I !== null; )
        n(F, I), I = I.sibling;
      return null;
    }
    function o(F) {
      for (var I = /* @__PURE__ */ new Map(); F !== null; )
        F.key !== null ? I.set(F.key, F) : I.set(F.index, F), F = F.sibling;
      return I;
    }
    function d(F, I) {
      return F = Ln(F, I), F.index = 0, F.sibling = null, F;
    }
    function p(F, I, ee) {
      return F.index = ee, e ? (ee = F.alternate, ee !== null ? (ee = ee.index, ee < I ? (F.flags |= 67108866, I) : ee) : (F.flags |= 67108866, I)) : (F.flags |= 1048576, I);
    }
    function x(F) {
      return e && F.alternate === null && (F.flags |= 67108866), F;
    }
    function C(F, I, ee, ce) {
      return I === null || I.tag !== 6 ? (I = Ps(ee, F.mode, ce), I.return = F, I) : (I = d(I, ee), I.return = F, I);
    }
    function V(F, I, ee, ce) {
      var Se = ee.type;
      return Se === A ? oe(
        F,
        I,
        ee.props.children,
        ce,
        ee.key
      ) : I !== null && (I.elementType === Se || typeof Se == "object" && Se !== null && Se.$$typeof === L && $r(Se) === I.type) ? (I = d(I, ee.props), Xi(I, ee), I.return = F, I) : (I = _l(
        ee.type,
        ee.key,
        ee.props,
        null,
        F.mode,
        ce
      ), Xi(I, ee), I.return = F, I);
    }
    function te(F, I, ee, ce) {
      return I === null || I.tag !== 4 || I.stateNode.containerInfo !== ee.containerInfo || I.stateNode.implementation !== ee.implementation ? (I = Ws(ee, F.mode, ce), I.return = F, I) : (I = d(I, ee.children || []), I.return = F, I);
    }
    function oe(F, I, ee, ce, Se) {
      return I === null || I.tag !== 7 ? (I = Xr(
        ee,
        F.mode,
        ce,
        Se
      ), I.return = F, I) : (I = d(I, ee), I.return = F, I);
    }
    function fe(F, I, ee) {
      if (typeof I == "string" && I !== "" || typeof I == "number" || typeof I == "bigint")
        return I = Ps(
          "" + I,
          F.mode,
          ee
        ), I.return = F, I;
      if (typeof I == "object" && I !== null) {
        switch (I.$$typeof) {
          case b:
            return ee = _l(
              I.type,
              I.key,
              I.props,
              null,
              F.mode,
              ee
            ), Xi(ee, I), ee.return = F, ee;
          case _:
            return I = Ws(
              I,
              F.mode,
              ee
            ), I.return = F, I;
          case L:
            return I = $r(I), fe(F, I, ee);
        }
        if (X(I) || J(I))
          return I = Xr(
            I,
            F.mode,
            ee,
            null
          ), I.return = F, I;
        if (typeof I.then == "function")
          return fe(F, Ml(I), ee);
        if (I.$$typeof === S)
          return fe(
            F,
            El(F, I),
            ee
          );
        Cl(F, I);
      }
      return null;
    }
    function ne(F, I, ee, ce) {
      var Se = I !== null ? I.key : null;
      if (typeof ee == "string" && ee !== "" || typeof ee == "number" || typeof ee == "bigint")
        return Se !== null ? null : C(F, I, "" + ee, ce);
      if (typeof ee == "object" && ee !== null) {
        switch (ee.$$typeof) {
          case b:
            return ee.key === Se ? V(F, I, ee, ce) : null;
          case _:
            return ee.key === Se ? te(F, I, ee, ce) : null;
          case L:
            return ee = $r(ee), ne(F, I, ee, ce);
        }
        if (X(ee) || J(ee))
          return Se !== null ? null : oe(F, I, ee, ce, null);
        if (typeof ee.then == "function")
          return ne(
            F,
            I,
            Ml(ee),
            ce
          );
        if (ee.$$typeof === S)
          return ne(
            F,
            I,
            El(F, ee),
            ce
          );
        Cl(F, ee);
      }
      return null;
    }
    function le(F, I, ee, ce, Se) {
      if (typeof ce == "string" && ce !== "" || typeof ce == "number" || typeof ce == "bigint")
        return F = F.get(ee) || null, C(I, F, "" + ce, Se);
      if (typeof ce == "object" && ce !== null) {
        switch (ce.$$typeof) {
          case b:
            return F = F.get(
              ce.key === null ? ee : ce.key
            ) || null, V(I, F, ce, Se);
          case _:
            return F = F.get(
              ce.key === null ? ee : ce.key
            ) || null, te(I, F, ce, Se);
          case L:
            return ce = $r(ce), le(
              F,
              I,
              ee,
              ce,
              Se
            );
        }
        if (X(ce) || J(ce))
          return F = F.get(ee) || null, oe(I, F, ce, Se, null);
        if (typeof ce.then == "function")
          return le(
            F,
            I,
            ee,
            Ml(ce),
            Se
          );
        if (ce.$$typeof === S)
          return le(
            F,
            I,
            ee,
            El(I, ce),
            Se
          );
        Cl(I, ce);
      }
      return null;
    }
    function pe(F, I, ee, ce) {
      for (var Se = null, De = null, be = I, Me = I = 0, Ne = null; be !== null && Me < ee.length; Me++) {
        be.index > Me ? (Ne = be, be = null) : Ne = be.sibling;
        var Le = ne(
          F,
          be,
          ee[Me],
          ce
        );
        if (Le === null) {
          be === null && (be = Ne);
          break;
        }
        e && be && Le.alternate === null && n(F, be), I = p(Le, I, Me), De === null ? Se = Le : De.sibling = Le, De = Le, be = Ne;
      }
      if (Me === ee.length)
        return a(F, be), ze && Hn(F, Me), Se;
      if (be === null) {
        for (; Me < ee.length; Me++)
          be = fe(F, ee[Me], ce), be !== null && (I = p(
            be,
            I,
            Me
          ), De === null ? Se = be : De.sibling = be, De = be);
        return ze && Hn(F, Me), Se;
      }
      for (be = o(be); Me < ee.length; Me++)
        Ne = le(
          be,
          F,
          Me,
          ee[Me],
          ce
        ), Ne !== null && (e && Ne.alternate !== null && be.delete(
          Ne.key === null ? Me : Ne.key
        ), I = p(
          Ne,
          I,
          Me
        ), De === null ? Se = Ne : De.sibling = Ne, De = Ne);
      return e && be.forEach(function(Rr) {
        return n(F, Rr);
      }), ze && Hn(F, Me), Se;
    }
    function Ee(F, I, ee, ce) {
      if (ee == null) throw Error(u(151));
      for (var Se = null, De = null, be = I, Me = I = 0, Ne = null, Le = ee.next(); be !== null && !Le.done; Me++, Le = ee.next()) {
        be.index > Me ? (Ne = be, be = null) : Ne = be.sibling;
        var Rr = ne(F, be, Le.value, ce);
        if (Rr === null) {
          be === null && (be = Ne);
          break;
        }
        e && be && Rr.alternate === null && n(F, be), I = p(Rr, I, Me), De === null ? Se = Rr : De.sibling = Rr, De = Rr, be = Ne;
      }
      if (Le.done)
        return a(F, be), ze && Hn(F, Me), Se;
      if (be === null) {
        for (; !Le.done; Me++, Le = ee.next())
          Le = fe(F, Le.value, ce), Le !== null && (I = p(Le, I, Me), De === null ? Se = Le : De.sibling = Le, De = Le);
        return ze && Hn(F, Me), Se;
      }
      for (be = o(be); !Le.done; Me++, Le = ee.next())
        Le = le(be, F, Me, Le.value, ce), Le !== null && (e && Le.alternate !== null && be.delete(Le.key === null ? Me : Le.key), I = p(Le, I, Me), De === null ? Se = Le : De.sibling = Le, De = Le);
      return e && be.forEach(function(BR) {
        return n(F, BR);
      }), ze && Hn(F, Me), Se;
    }
    function Ve(F, I, ee, ce) {
      if (typeof ee == "object" && ee !== null && ee.type === A && ee.key === null && (ee = ee.props.children), typeof ee == "object" && ee !== null) {
        switch (ee.$$typeof) {
          case b:
            e: {
              for (var Se = ee.key; I !== null; ) {
                if (I.key === Se) {
                  if (Se = ee.type, Se === A) {
                    if (I.tag === 7) {
                      a(
                        F,
                        I.sibling
                      ), ce = d(
                        I,
                        ee.props.children
                      ), ce.return = F, F = ce;
                      break e;
                    }
                  } else if (I.elementType === Se || typeof Se == "object" && Se !== null && Se.$$typeof === L && $r(Se) === I.type) {
                    a(
                      F,
                      I.sibling
                    ), ce = d(I, ee.props), Xi(ce, ee), ce.return = F, F = ce;
                    break e;
                  }
                  a(F, I);
                  break;
                } else n(F, I);
                I = I.sibling;
              }
              ee.type === A ? (ce = Xr(
                ee.props.children,
                F.mode,
                ce,
                ee.key
              ), ce.return = F, F = ce) : (ce = _l(
                ee.type,
                ee.key,
                ee.props,
                null,
                F.mode,
                ce
              ), Xi(ce, ee), ce.return = F, F = ce);
            }
            return x(F);
          case _:
            e: {
              for (Se = ee.key; I !== null; ) {
                if (I.key === Se)
                  if (I.tag === 4 && I.stateNode.containerInfo === ee.containerInfo && I.stateNode.implementation === ee.implementation) {
                    a(
                      F,
                      I.sibling
                    ), ce = d(I, ee.children || []), ce.return = F, F = ce;
                    break e;
                  } else {
                    a(F, I);
                    break;
                  }
                else n(F, I);
                I = I.sibling;
              }
              ce = Ws(ee, F.mode, ce), ce.return = F, F = ce;
            }
            return x(F);
          case L:
            return ee = $r(ee), Ve(
              F,
              I,
              ee,
              ce
            );
        }
        if (X(ee))
          return pe(
            F,
            I,
            ee,
            ce
          );
        if (J(ee)) {
          if (Se = J(ee), typeof Se != "function") throw Error(u(150));
          return ee = Se.call(ee), Ee(
            F,
            I,
            ee,
            ce
          );
        }
        if (typeof ee.then == "function")
          return Ve(
            F,
            I,
            Ml(ee),
            ce
          );
        if (ee.$$typeof === S)
          return Ve(
            F,
            I,
            El(F, ee),
            ce
          );
        Cl(F, ee);
      }
      return typeof ee == "string" && ee !== "" || typeof ee == "number" || typeof ee == "bigint" ? (ee = "" + ee, I !== null && I.tag === 6 ? (a(F, I.sibling), ce = d(I, ee), ce.return = F, F = ce) : (a(F, I), ce = Ps(ee, F.mode, ce), ce.return = F, F = ce), x(F)) : a(F, I);
    }
    return function(F, I, ee, ce) {
      try {
        ki = 0;
        var Se = Ve(
          F,
          I,
          ee,
          ce
        );
        return Ga = null, Se;
      } catch (be) {
        if (be === Ua || be === Al) throw be;
        var De = Vt(29, be, null, F.mode);
        return De.lanes = ce, De.return = F, De;
      } finally {
      }
    };
  }
  var Jr = h0(!0), g0 = h0(!1), hr = !1;
  function fc(e) {
    e.updateQueue = {
      baseState: e.memoizedState,
      firstBaseUpdate: null,
      lastBaseUpdate: null,
      shared: { pending: null, lanes: 0, hiddenCallbacks: null },
      callbacks: null
    };
  }
  function dc(e, n) {
    e = e.updateQueue, n.updateQueue === e && (n.updateQueue = {
      baseState: e.baseState,
      firstBaseUpdate: e.firstBaseUpdate,
      lastBaseUpdate: e.lastBaseUpdate,
      shared: e.shared,
      callbacks: null
    });
  }
  function gr(e) {
    return { lane: e, tag: 0, payload: null, callback: null, next: null };
  }
  function vr(e, n, a) {
    var o = e.updateQueue;
    if (o === null) return null;
    if (o = o.shared, (He & 2) !== 0) {
      var d = o.pending;
      return d === null ? n.next = n : (n.next = d.next, d.next = n), o.pending = n, n = bl(e), Jm(e, null, a), n;
    }
    return ml(e, o, n, a), bl(e);
  }
  function Ii(e, n, a) {
    if (n = n.updateQueue, n !== null && (n = n.shared, (a & 4194048) !== 0)) {
      var o = n.lanes;
      o &= e.pendingLanes, a |= o, n.lanes = a, Pu(e, a);
    }
  }
  function hc(e, n) {
    var a = e.updateQueue, o = e.alternate;
    if (o !== null && (o = o.updateQueue, a === o)) {
      var d = null, p = null;
      if (a = a.firstBaseUpdate, a !== null) {
        do {
          var x = {
            lane: a.lane,
            tag: a.tag,
            payload: a.payload,
            callback: null,
            next: null
          };
          p === null ? d = p = x : p = p.next = x, a = a.next;
        } while (a !== null);
        p === null ? d = p = n : p = p.next = n;
      } else d = p = n;
      a = {
        baseState: o.baseState,
        firstBaseUpdate: d,
        lastBaseUpdate: p,
        shared: o.shared,
        callbacks: o.callbacks
      }, e.updateQueue = a;
      return;
    }
    e = a.lastBaseUpdate, e === null ? a.firstBaseUpdate = n : e.next = n, a.lastBaseUpdate = n;
  }
  var gc = !1;
  function Zi() {
    if (gc) {
      var e = Ba;
      if (e !== null) throw e;
    }
  }
  function Ki(e, n, a, o) {
    gc = !1;
    var d = e.updateQueue;
    hr = !1;
    var p = d.firstBaseUpdate, x = d.lastBaseUpdate, C = d.shared.pending;
    if (C !== null) {
      d.shared.pending = null;
      var V = C, te = V.next;
      V.next = null, x === null ? p = te : x.next = te, x = V;
      var oe = e.alternate;
      oe !== null && (oe = oe.updateQueue, C = oe.lastBaseUpdate, C !== x && (C === null ? oe.firstBaseUpdate = te : C.next = te, oe.lastBaseUpdate = V));
    }
    if (p !== null) {
      var fe = d.baseState;
      x = 0, oe = te = V = null, C = p;
      do {
        var ne = C.lane & -536870913, le = ne !== C.lane;
        if (le ? (Re & ne) === ne : (o & ne) === ne) {
          ne !== 0 && ne === ja && (gc = !0), oe !== null && (oe = oe.next = {
            lane: 0,
            tag: C.tag,
            payload: C.payload,
            callback: null,
            next: null
          });
          e: {
            var pe = e, Ee = C;
            ne = n;
            var Ve = a;
            switch (Ee.tag) {
              case 1:
                if (pe = Ee.payload, typeof pe == "function") {
                  fe = pe.call(Ve, fe, ne);
                  break e;
                }
                fe = pe;
                break e;
              case 3:
                pe.flags = pe.flags & -65537 | 128;
              case 0:
                if (pe = Ee.payload, ne = typeof pe == "function" ? pe.call(Ve, fe, ne) : pe, ne == null) break e;
                fe = y({}, fe, ne);
                break e;
              case 2:
                hr = !0;
            }
          }
          ne = C.callback, ne !== null && (e.flags |= 64, le && (e.flags |= 8192), le = d.callbacks, le === null ? d.callbacks = [ne] : le.push(ne));
        } else
          le = {
            lane: ne,
            tag: C.tag,
            payload: C.payload,
            callback: C.callback,
            next: null
          }, oe === null ? (te = oe = le, V = fe) : oe = oe.next = le, x |= ne;
        if (C = C.next, C === null) {
          if (C = d.shared.pending, C === null)
            break;
          le = C, C = le.next, le.next = null, d.lastBaseUpdate = le, d.shared.pending = null;
        }
      } while (!0);
      oe === null && (V = fe), d.baseState = V, d.firstBaseUpdate = te, d.lastBaseUpdate = oe, p === null && (d.shared.lanes = 0), _r |= x, e.lanes = x, e.memoizedState = fe;
    }
  }
  function v0(e, n) {
    if (typeof e != "function")
      throw Error(u(191, e));
    e.call(n);
  }
  function p0(e, n) {
    var a = e.callbacks;
    if (a !== null)
      for (e.callbacks = null, e = 0; e < a.length; e++)
        v0(a[e], n);
  }
  var Va = O(null), ql = O(0);
  function y0(e, n) {
    e = Kn, ie(ql, e), ie(Va, n), Kn = e | n.baseLanes;
  }
  function vc() {
    ie(ql, Kn), ie(Va, Va.current);
  }
  function pc() {
    Kn = ql.current, G(Va), G(ql);
  }
  var Yt = O(null), en = null;
  function pr(e) {
    var n = e.alternate;
    ie(nt, nt.current & 1), ie(Yt, e), en === null && (n === null || Va.current !== null || n.memoizedState !== null) && (en = e);
  }
  function yc(e) {
    ie(nt, nt.current), ie(Yt, e), en === null && (en = e);
  }
  function m0(e) {
    e.tag === 22 ? (ie(nt, nt.current), ie(Yt, e), en === null && (en = e)) : yr();
  }
  function yr() {
    ie(nt, nt.current), ie(Yt, Yt.current);
  }
  function kt(e) {
    G(Yt), en === e && (en = null), G(nt);
  }
  var nt = O(0);
  function Rl(e) {
    for (var n = e; n !== null; ) {
      if (n.tag === 13) {
        var a = n.memoizedState;
        if (a !== null && (a = a.dehydrated, a === null || wf(a) || Af(a)))
          return n;
      } else if (n.tag === 19 && (n.memoizedProps.revealOrder === "forwards" || n.memoizedProps.revealOrder === "backwards" || n.memoizedProps.revealOrder === "unstable_legacy-backwards" || n.memoizedProps.revealOrder === "together")) {
        if ((n.flags & 128) !== 0) return n;
      } else if (n.child !== null) {
        n.child.return = n, n = n.child;
        continue;
      }
      if (n === e) break;
      for (; n.sibling === null; ) {
        if (n.return === null || n.return === e) return null;
        n = n.return;
      }
      n.sibling.return = n.return, n = n.sibling;
    }
    return null;
  }
  var Un = 0, Te = null, Ue = null, ut = null, Nl = !1, Ya = !1, Pr = !1, zl = 0, Qi = 0, ka = null, Cq = 0;
  function We() {
    throw Error(u(321));
  }
  function mc(e, n) {
    if (n === null) return !1;
    for (var a = 0; a < n.length && a < e.length; a++)
      if (!Gt(e[a], n[a])) return !1;
    return !0;
  }
  function bc(e, n, a, o, d, p) {
    return Un = p, Te = n, n.memoizedState = null, n.updateQueue = null, n.lanes = 0, z.H = e === null || e.memoizedState === null ? t1 : Dc, Pr = !1, p = a(o, d), Pr = !1, Ya && (p = _0(
      n,
      a,
      o,
      d
    )), b0(e), p;
  }
  function b0(e) {
    z.H = Ji;
    var n = Ue !== null && Ue.next !== null;
    if (Un = 0, ut = Ue = Te = null, Nl = !1, Qi = 0, ka = null, n) throw Error(u(300));
    e === null || lt || (e = e.dependencies, e !== null && Sl(e) && (lt = !0));
  }
  function _0(e, n, a, o) {
    Te = e;
    var d = 0;
    do {
      if (Ya && (ka = null), Qi = 0, Ya = !1, 25 <= d) throw Error(u(301));
      if (d += 1, ut = Ue = null, e.updateQueue != null) {
        var p = e.updateQueue;
        p.lastEffect = null, p.events = null, p.stores = null, p.memoCache != null && (p.memoCache.index = 0);
      }
      z.H = n1, p = n(a, o);
    } while (Ya);
    return p;
  }
  function qq() {
    var e = z.H, n = e.useState()[0];
    return n = typeof n.then == "function" ? $i(n) : n, e = e.useState()[0], (Ue !== null ? Ue.memoizedState : null) !== e && (Te.flags |= 1024), n;
  }
  function _c() {
    var e = zl !== 0;
    return zl = 0, e;
  }
  function xc(e, n, a) {
    n.updateQueue = e.updateQueue, n.flags &= -2053, e.lanes &= ~a;
  }
  function Sc(e) {
    if (Nl) {
      for (e = e.memoizedState; e !== null; ) {
        var n = e.queue;
        n !== null && (n.pending = null), e = e.next;
      }
      Nl = !1;
    }
    Un = 0, ut = Ue = Te = null, Ya = !1, Qi = zl = 0, ka = null;
  }
  function Ct() {
    var e = {
      memoizedState: null,
      baseState: null,
      baseQueue: null,
      queue: null,
      next: null
    };
    return ut === null ? Te.memoizedState = ut = e : ut = ut.next = e, ut;
  }
  function rt() {
    if (Ue === null) {
      var e = Te.alternate;
      e = e !== null ? e.memoizedState : null;
    } else e = Ue.next;
    var n = ut === null ? Te.memoizedState : ut.next;
    if (n !== null)
      ut = n, Ue = e;
    else {
      if (e === null)
        throw Te.alternate === null ? Error(u(467)) : Error(u(310));
      Ue = e, e = {
        memoizedState: Ue.memoizedState,
        baseState: Ue.baseState,
        baseQueue: Ue.baseQueue,
        queue: Ue.queue,
        next: null
      }, ut === null ? Te.memoizedState = ut = e : ut = ut.next = e;
    }
    return ut;
  }
  function Ol() {
    return { lastEffect: null, events: null, stores: null, memoCache: null };
  }
  function $i(e) {
    var n = Qi;
    return Qi += 1, ka === null && (ka = []), e = c0(ka, e, n), n = Te, (ut === null ? n.memoizedState : ut.next) === null && (n = n.alternate, z.H = n === null || n.memoizedState === null ? t1 : Dc), e;
  }
  function Dl(e) {
    if (e !== null && typeof e == "object") {
      if (typeof e.then == "function") return $i(e);
      if (e.$$typeof === S) return mt(e);
    }
    throw Error(u(438, String(e)));
  }
  function Ec(e) {
    var n = null, a = Te.updateQueue;
    if (a !== null && (n = a.memoCache), n == null) {
      var o = Te.alternate;
      o !== null && (o = o.updateQueue, o !== null && (o = o.memoCache, o != null && (n = {
        data: o.data.map(function(d) {
          return d.slice();
        }),
        index: 0
      })));
    }
    if (n == null && (n = { data: [], index: 0 }), a === null && (a = Ol(), Te.updateQueue = a), a.memoCache = n, a = n.data[n.index], a === void 0)
      for (a = n.data[n.index] = Array(e), o = 0; o < e; o++)
        a[o] = Z;
    return n.index++, a;
  }
  function Gn(e, n) {
    return typeof n == "function" ? n(e) : n;
  }
  function Ll(e) {
    var n = rt();
    return wc(n, Ue, e);
  }
  function wc(e, n, a) {
    var o = e.queue;
    if (o === null) throw Error(u(311));
    o.lastRenderedReducer = a;
    var d = e.baseQueue, p = o.pending;
    if (p !== null) {
      if (d !== null) {
        var x = d.next;
        d.next = p.next, p.next = x;
      }
      n.baseQueue = d = p, o.pending = null;
    }
    if (p = e.baseState, d === null) e.memoizedState = p;
    else {
      n = d.next;
      var C = x = null, V = null, te = n, oe = !1;
      do {
        var fe = te.lane & -536870913;
        if (fe !== te.lane ? (Re & fe) === fe : (Un & fe) === fe) {
          var ne = te.revertLane;
          if (ne === 0)
            V !== null && (V = V.next = {
              lane: 0,
              revertLane: 0,
              gesture: null,
              action: te.action,
              hasEagerState: te.hasEagerState,
              eagerState: te.eagerState,
              next: null
            }), fe === ja && (oe = !0);
          else if ((Un & ne) === ne) {
            te = te.next, ne === ja && (oe = !0);
            continue;
          } else
            fe = {
              lane: 0,
              revertLane: te.revertLane,
              gesture: null,
              action: te.action,
              hasEagerState: te.hasEagerState,
              eagerState: te.eagerState,
              next: null
            }, V === null ? (C = V = fe, x = p) : V = V.next = fe, Te.lanes |= ne, _r |= ne;
          fe = te.action, Pr && a(p, fe), p = te.hasEagerState ? te.eagerState : a(p, fe);
        } else
          ne = {
            lane: fe,
            revertLane: te.revertLane,
            gesture: te.gesture,
            action: te.action,
            hasEagerState: te.hasEagerState,
            eagerState: te.eagerState,
            next: null
          }, V === null ? (C = V = ne, x = p) : V = V.next = ne, Te.lanes |= fe, _r |= fe;
        te = te.next;
      } while (te !== null && te !== n);
      if (V === null ? x = p : V.next = C, !Gt(p, e.memoizedState) && (lt = !0, oe && (a = Ba, a !== null)))
        throw a;
      e.memoizedState = p, e.baseState = x, e.baseQueue = V, o.lastRenderedState = p;
    }
    return d === null && (o.lanes = 0), [e.memoizedState, o.dispatch];
  }
  function Ac(e) {
    var n = rt(), a = n.queue;
    if (a === null) throw Error(u(311));
    a.lastRenderedReducer = e;
    var o = a.dispatch, d = a.pending, p = n.memoizedState;
    if (d !== null) {
      a.pending = null;
      var x = d = d.next;
      do
        p = e(p, x.action), x = x.next;
      while (x !== d);
      Gt(p, n.memoizedState) || (lt = !0), n.memoizedState = p, n.baseQueue === null && (n.baseState = p), a.lastRenderedState = p;
    }
    return [p, o];
  }
  function x0(e, n, a) {
    var o = Te, d = rt(), p = ze;
    if (p) {
      if (a === void 0) throw Error(u(407));
      a = a();
    } else a = n();
    var x = !Gt(
      (Ue || d).memoizedState,
      a
    );
    if (x && (d.memoizedState = a, lt = !0), d = d.queue, Cc(w0.bind(null, o, d, e), [
      e
    ]), d.getSnapshot !== n || x || ut !== null && ut.memoizedState.tag & 1) {
      if (o.flags |= 2048, Xa(
        9,
        { destroy: void 0 },
        E0.bind(
          null,
          o,
          d,
          a,
          n
        ),
        null
      ), Ye === null) throw Error(u(349));
      p || (Un & 127) !== 0 || S0(o, n, a);
    }
    return a;
  }
  function S0(e, n, a) {
    e.flags |= 16384, e = { getSnapshot: n, value: a }, n = Te.updateQueue, n === null ? (n = Ol(), Te.updateQueue = n, n.stores = [e]) : (a = n.stores, a === null ? n.stores = [e] : a.push(e));
  }
  function E0(e, n, a, o) {
    n.value = a, n.getSnapshot = o, A0(n) && T0(e);
  }
  function w0(e, n, a) {
    return a(function() {
      A0(n) && T0(e);
    });
  }
  function A0(e) {
    var n = e.getSnapshot;
    e = e.value;
    try {
      var a = n();
      return !Gt(e, a);
    } catch {
      return !0;
    }
  }
  function T0(e) {
    var n = kr(e, 2);
    n !== null && jt(n, e, 2);
  }
  function Tc(e) {
    var n = Ct();
    if (typeof e == "function") {
      var a = e;
      if (e = a(), Pr) {
        mn(!0);
        try {
          a();
        } finally {
          mn(!1);
        }
      }
    }
    return n.memoizedState = n.baseState = e, n.queue = {
      pending: null,
      lanes: 0,
      dispatch: null,
      lastRenderedReducer: Gn,
      lastRenderedState: e
    }, n;
  }
  function M0(e, n, a, o) {
    return e.baseState = a, wc(
      e,
      Ue,
      typeof o == "function" ? o : Gn
    );
  }
  function Rq(e, n, a, o, d) {
    if (Bl(e)) throw Error(u(485));
    if (e = n.action, e !== null) {
      var p = {
        payload: d,
        action: e,
        next: null,
        isTransition: !0,
        status: "pending",
        value: null,
        reason: null,
        listeners: [],
        then: function(x) {
          p.listeners.push(x);
        }
      };
      z.T !== null ? a(!0) : p.isTransition = !1, o(p), a = n.pending, a === null ? (p.next = n.pending = p, C0(n, p)) : (p.next = a.next, n.pending = a.next = p);
    }
  }
  function C0(e, n) {
    var a = n.action, o = n.payload, d = e.state;
    if (n.isTransition) {
      var p = z.T, x = {};
      z.T = x;
      try {
        var C = a(d, o), V = z.S;
        V !== null && V(x, C), q0(e, n, C);
      } catch (te) {
        Mc(e, n, te);
      } finally {
        p !== null && x.types !== null && (p.types = x.types), z.T = p;
      }
    } else
      try {
        p = a(d, o), q0(e, n, p);
      } catch (te) {
        Mc(e, n, te);
      }
  }
  function q0(e, n, a) {
    a !== null && typeof a == "object" && typeof a.then == "function" ? a.then(
      function(o) {
        R0(e, n, o);
      },
      function(o) {
        return Mc(e, n, o);
      }
    ) : R0(e, n, a);
  }
  function R0(e, n, a) {
    n.status = "fulfilled", n.value = a, N0(n), e.state = a, n = e.pending, n !== null && (a = n.next, a === n ? e.pending = null : (a = a.next, n.next = a, C0(e, a)));
  }
  function Mc(e, n, a) {
    var o = e.pending;
    if (e.pending = null, o !== null) {
      o = o.next;
      do
        n.status = "rejected", n.reason = a, N0(n), n = n.next;
      while (n !== o);
    }
    e.action = null;
  }
  function N0(e) {
    e = e.listeners;
    for (var n = 0; n < e.length; n++) (0, e[n])();
  }
  function z0(e, n) {
    return n;
  }
  function O0(e, n) {
    if (ze) {
      var a = Ye.formState;
      if (a !== null) {
        e: {
          var o = Te;
          if (ze) {
            if (Xe) {
              t: {
                for (var d = Xe, p = Wt; d.nodeType !== 8; ) {
                  if (!p) {
                    d = null;
                    break t;
                  }
                  if (d = tn(
                    d.nextSibling
                  ), d === null) {
                    d = null;
                    break t;
                  }
                }
                p = d.data, d = p === "F!" || p === "F" ? d : null;
              }
              if (d) {
                Xe = tn(
                  d.nextSibling
                ), o = d.data === "F!";
                break e;
              }
            }
            fr(o);
          }
          o = !1;
        }
        o && (n = a[0]);
      }
    }
    return a = Ct(), a.memoizedState = a.baseState = n, o = {
      pending: null,
      lanes: 0,
      dispatch: null,
      lastRenderedReducer: z0,
      lastRenderedState: n
    }, a.queue = o, a = P0.bind(
      null,
      Te,
      o
    ), o.dispatch = a, o = Tc(!1), p = Oc.bind(
      null,
      Te,
      !1,
      o.queue
    ), o = Ct(), d = {
      state: n,
      dispatch: null,
      action: e,
      pending: null
    }, o.queue = d, a = Rq.bind(
      null,
      Te,
      d,
      p,
      a
    ), d.dispatch = a, o.memoizedState = e, [n, a, !1];
  }
  function D0(e) {
    var n = rt();
    return L0(n, Ue, e);
  }
  function L0(e, n, a) {
    if (n = wc(
      e,
      n,
      z0
    )[0], e = Ll(Gn)[0], typeof n == "object" && n !== null && typeof n.then == "function")
      try {
        var o = $i(n);
      } catch (x) {
        throw x === Ua ? Al : x;
      }
    else o = n;
    n = rt();
    var d = n.queue, p = d.dispatch;
    return a !== n.memoizedState && (Te.flags |= 2048, Xa(
      9,
      { destroy: void 0 },
      Nq.bind(null, d, a),
      null
    )), [o, p, e];
  }
  function Nq(e, n) {
    e.action = n;
  }
  function H0(e) {
    var n = rt(), a = Ue;
    if (a !== null)
      return L0(n, a, e);
    rt(), n = n.memoizedState, a = rt();
    var o = a.queue.dispatch;
    return a.memoizedState = e, [n, o, !1];
  }
  function Xa(e, n, a, o) {
    return e = { tag: e, create: a, deps: o, inst: n, next: null }, n = Te.updateQueue, n === null && (n = Ol(), Te.updateQueue = n), a = n.lastEffect, a === null ? n.lastEffect = e.next = e : (o = a.next, a.next = e, e.next = o, n.lastEffect = e), e;
  }
  function j0() {
    return rt().memoizedState;
  }
  function Hl(e, n, a, o) {
    var d = Ct();
    Te.flags |= e, d.memoizedState = Xa(
      1 | n,
      { destroy: void 0 },
      a,
      o === void 0 ? null : o
    );
  }
  function jl(e, n, a, o) {
    var d = rt();
    o = o === void 0 ? null : o;
    var p = d.memoizedState.inst;
    Ue !== null && o !== null && mc(o, Ue.memoizedState.deps) ? d.memoizedState = Xa(n, p, a, o) : (Te.flags |= e, d.memoizedState = Xa(
      1 | n,
      p,
      a,
      o
    ));
  }
  function B0(e, n) {
    Hl(8390656, 8, e, n);
  }
  function Cc(e, n) {
    jl(2048, 8, e, n);
  }
  function zq(e) {
    Te.flags |= 4;
    var n = Te.updateQueue;
    if (n === null)
      n = Ol(), Te.updateQueue = n, n.events = [e];
    else {
      var a = n.events;
      a === null ? n.events = [e] : a.push(e);
    }
  }
  function U0(e) {
    var n = rt().memoizedState;
    return zq({ ref: n, nextImpl: e }), function() {
      if ((He & 2) !== 0) throw Error(u(440));
      return n.impl.apply(void 0, arguments);
    };
  }
  function G0(e, n) {
    return jl(4, 2, e, n);
  }
  function V0(e, n) {
    return jl(4, 4, e, n);
  }
  function Y0(e, n) {
    if (typeof n == "function") {
      e = e();
      var a = n(e);
      return function() {
        typeof a == "function" ? a() : n(null);
      };
    }
    if (n != null)
      return e = e(), n.current = e, function() {
        n.current = null;
      };
  }
  function k0(e, n, a) {
    a = a != null ? a.concat([e]) : null, jl(4, 4, Y0.bind(null, n, e), a);
  }
  function qc() {
  }
  function X0(e, n) {
    var a = rt();
    n = n === void 0 ? null : n;
    var o = a.memoizedState;
    return n !== null && mc(n, o[1]) ? o[0] : (a.memoizedState = [e, n], e);
  }
  function I0(e, n) {
    var a = rt();
    n = n === void 0 ? null : n;
    var o = a.memoizedState;
    if (n !== null && mc(n, o[1]))
      return o[0];
    if (o = e(), Pr) {
      mn(!0);
      try {
        e();
      } finally {
        mn(!1);
      }
    }
    return a.memoizedState = [o, n], o;
  }
  function Rc(e, n, a) {
    return a === void 0 || (Un & 1073741824) !== 0 && (Re & 261930) === 0 ? e.memoizedState = n : (e.memoizedState = a, e = Z1(), Te.lanes |= e, _r |= e, a);
  }
  function Z0(e, n, a, o) {
    return Gt(a, n) ? a : Va.current !== null ? (e = Rc(e, a, o), Gt(e, n) || (lt = !0), e) : (Un & 42) === 0 || (Un & 1073741824) !== 0 && (Re & 261930) === 0 ? (lt = !0, e.memoizedState = a) : (e = Z1(), Te.lanes |= e, _r |= e, n);
  }
  function K0(e, n, a, o, d) {
    var p = B.p;
    B.p = p !== 0 && 8 > p ? p : 8;
    var x = z.T, C = {};
    z.T = C, Oc(e, !1, n, a);
    try {
      var V = d(), te = z.S;
      if (te !== null && te(C, V), V !== null && typeof V == "object" && typeof V.then == "function") {
        var oe = Mq(
          V,
          o
        );
        Fi(
          e,
          n,
          oe,
          Zt(e)
        );
      } else
        Fi(
          e,
          n,
          o,
          Zt(e)
        );
    } catch (fe) {
      Fi(
        e,
        n,
        { then: function() {
        }, status: "rejected", reason: fe },
        Zt()
      );
    } finally {
      B.p = p, x !== null && C.types !== null && (x.types = C.types), z.T = x;
    }
  }
  function Oq() {
  }
  function Nc(e, n, a, o) {
    if (e.tag !== 5) throw Error(u(476));
    var d = Q0(e).queue;
    K0(
      e,
      d,
      n,
      $,
      a === null ? Oq : function() {
        return $0(e), a(o);
      }
    );
  }
  function Q0(e) {
    var n = e.memoizedState;
    if (n !== null) return n;
    n = {
      memoizedState: $,
      baseState: $,
      baseQueue: null,
      queue: {
        pending: null,
        lanes: 0,
        dispatch: null,
        lastRenderedReducer: Gn,
        lastRenderedState: $
      },
      next: null
    };
    var a = {};
    return n.next = {
      memoizedState: a,
      baseState: a,
      baseQueue: null,
      queue: {
        pending: null,
        lanes: 0,
        dispatch: null,
        lastRenderedReducer: Gn,
        lastRenderedState: a
      },
      next: null
    }, e.memoizedState = n, e = e.alternate, e !== null && (e.memoizedState = n), n;
  }
  function $0(e) {
    var n = Q0(e);
    n.next === null && (n = e.alternate.memoizedState), Fi(
      e,
      n.next.queue,
      {},
      Zt()
    );
  }
  function zc() {
    return mt(hu);
  }
  function F0() {
    return rt().memoizedState;
  }
  function J0() {
    return rt().memoizedState;
  }
  function Dq(e) {
    for (var n = e.return; n !== null; ) {
      switch (n.tag) {
        case 24:
        case 3:
          var a = Zt();
          e = gr(a);
          var o = vr(n, e, a);
          o !== null && (jt(o, n, a), Ii(o, n, a)), n = { cache: lc() }, e.payload = n;
          return;
      }
      n = n.return;
    }
  }
  function Lq(e, n, a) {
    var o = Zt();
    a = {
      lane: o,
      revertLane: 0,
      gesture: null,
      action: a,
      hasEagerState: !1,
      eagerState: null,
      next: null
    }, Bl(e) ? W0(n, a) : (a = Fs(e, n, a, o), a !== null && (jt(a, e, o), e1(a, n, o)));
  }
  function P0(e, n, a) {
    var o = Zt();
    Fi(e, n, a, o);
  }
  function Fi(e, n, a, o) {
    var d = {
      lane: o,
      revertLane: 0,
      gesture: null,
      action: a,
      hasEagerState: !1,
      eagerState: null,
      next: null
    };
    if (Bl(e)) W0(n, d);
    else {
      var p = e.alternate;
      if (e.lanes === 0 && (p === null || p.lanes === 0) && (p = n.lastRenderedReducer, p !== null))
        try {
          var x = n.lastRenderedState, C = p(x, a);
          if (d.hasEagerState = !0, d.eagerState = C, Gt(C, x))
            return ml(e, n, d, 0), Ye === null && yl(), !1;
        } catch {
        } finally {
        }
      if (a = Fs(e, n, d, o), a !== null)
        return jt(a, e, o), e1(a, n, o), !0;
    }
    return !1;
  }
  function Oc(e, n, a, o) {
    if (o = {
      lane: 2,
      revertLane: df(),
      gesture: null,
      action: o,
      hasEagerState: !1,
      eagerState: null,
      next: null
    }, Bl(e)) {
      if (n) throw Error(u(479));
    } else
      n = Fs(
        e,
        a,
        o,
        2
      ), n !== null && jt(n, e, 2);
  }
  function Bl(e) {
    var n = e.alternate;
    return e === Te || n !== null && n === Te;
  }
  function W0(e, n) {
    Ya = Nl = !0;
    var a = e.pending;
    a === null ? n.next = n : (n.next = a.next, a.next = n), e.pending = n;
  }
  function e1(e, n, a) {
    if ((a & 4194048) !== 0) {
      var o = n.lanes;
      o &= e.pendingLanes, a |= o, n.lanes = a, Pu(e, a);
    }
  }
  var Ji = {
    readContext: mt,
    use: Dl,
    useCallback: We,
    useContext: We,
    useEffect: We,
    useImperativeHandle: We,
    useLayoutEffect: We,
    useInsertionEffect: We,
    useMemo: We,
    useReducer: We,
    useRef: We,
    useState: We,
    useDebugValue: We,
    useDeferredValue: We,
    useTransition: We,
    useSyncExternalStore: We,
    useId: We,
    useHostTransitionStatus: We,
    useFormState: We,
    useActionState: We,
    useOptimistic: We,
    useMemoCache: We,
    useCacheRefresh: We
  };
  Ji.useEffectEvent = We;
  var t1 = {
    readContext: mt,
    use: Dl,
    useCallback: function(e, n) {
      return Ct().memoizedState = [
        e,
        n === void 0 ? null : n
      ], e;
    },
    useContext: mt,
    useEffect: B0,
    useImperativeHandle: function(e, n, a) {
      a = a != null ? a.concat([e]) : null, Hl(
        4194308,
        4,
        Y0.bind(null, n, e),
        a
      );
    },
    useLayoutEffect: function(e, n) {
      return Hl(4194308, 4, e, n);
    },
    useInsertionEffect: function(e, n) {
      Hl(4, 2, e, n);
    },
    useMemo: function(e, n) {
      var a = Ct();
      n = n === void 0 ? null : n;
      var o = e();
      if (Pr) {
        mn(!0);
        try {
          e();
        } finally {
          mn(!1);
        }
      }
      return a.memoizedState = [o, n], o;
    },
    useReducer: function(e, n, a) {
      var o = Ct();
      if (a !== void 0) {
        var d = a(n);
        if (Pr) {
          mn(!0);
          try {
            a(n);
          } finally {
            mn(!1);
          }
        }
      } else d = n;
      return o.memoizedState = o.baseState = d, e = {
        pending: null,
        lanes: 0,
        dispatch: null,
        lastRenderedReducer: e,
        lastRenderedState: d
      }, o.queue = e, e = e.dispatch = Lq.bind(
        null,
        Te,
        e
      ), [o.memoizedState, e];
    },
    useRef: function(e) {
      var n = Ct();
      return e = { current: e }, n.memoizedState = e;
    },
    useState: function(e) {
      e = Tc(e);
      var n = e.queue, a = P0.bind(null, Te, n);
      return n.dispatch = a, [e.memoizedState, a];
    },
    useDebugValue: qc,
    useDeferredValue: function(e, n) {
      var a = Ct();
      return Rc(a, e, n);
    },
    useTransition: function() {
      var e = Tc(!1);
      return e = K0.bind(
        null,
        Te,
        e.queue,
        !0,
        !1
      ), Ct().memoizedState = e, [!1, e];
    },
    useSyncExternalStore: function(e, n, a) {
      var o = Te, d = Ct();
      if (ze) {
        if (a === void 0)
          throw Error(u(407));
        a = a();
      } else {
        if (a = n(), Ye === null)
          throw Error(u(349));
        (Re & 127) !== 0 || S0(o, n, a);
      }
      d.memoizedState = a;
      var p = { value: a, getSnapshot: n };
      return d.queue = p, B0(w0.bind(null, o, p, e), [
        e
      ]), o.flags |= 2048, Xa(
        9,
        { destroy: void 0 },
        E0.bind(
          null,
          o,
          p,
          a,
          n
        ),
        null
      ), a;
    },
    useId: function() {
      var e = Ct(), n = Ye.identifierPrefix;
      if (ze) {
        var a = xn, o = _n;
        a = (o & ~(1 << 32 - Mt(o) - 1)).toString(32) + a, n = "_" + n + "R_" + a, a = zl++, 0 < a && (n += "H" + a.toString(32)), n += "_";
      } else
        a = Cq++, n = "_" + n + "r_" + a.toString(32) + "_";
      return e.memoizedState = n;
    },
    useHostTransitionStatus: zc,
    useFormState: O0,
    useActionState: O0,
    useOptimistic: function(e) {
      var n = Ct();
      n.memoizedState = n.baseState = e;
      var a = {
        pending: null,
        lanes: 0,
        dispatch: null,
        lastRenderedReducer: null,
        lastRenderedState: null
      };
      return n.queue = a, n = Oc.bind(
        null,
        Te,
        !0,
        a
      ), a.dispatch = n, [e, n];
    },
    useMemoCache: Ec,
    useCacheRefresh: function() {
      return Ct().memoizedState = Dq.bind(
        null,
        Te
      );
    },
    useEffectEvent: function(e) {
      var n = Ct(), a = { impl: e };
      return n.memoizedState = a, function() {
        if ((He & 2) !== 0)
          throw Error(u(440));
        return a.impl.apply(void 0, arguments);
      };
    }
  }, Dc = {
    readContext: mt,
    use: Dl,
    useCallback: X0,
    useContext: mt,
    useEffect: Cc,
    useImperativeHandle: k0,
    useInsertionEffect: G0,
    useLayoutEffect: V0,
    useMemo: I0,
    useReducer: Ll,
    useRef: j0,
    useState: function() {
      return Ll(Gn);
    },
    useDebugValue: qc,
    useDeferredValue: function(e, n) {
      var a = rt();
      return Z0(
        a,
        Ue.memoizedState,
        e,
        n
      );
    },
    useTransition: function() {
      var e = Ll(Gn)[0], n = rt().memoizedState;
      return [
        typeof e == "boolean" ? e : $i(e),
        n
      ];
    },
    useSyncExternalStore: x0,
    useId: F0,
    useHostTransitionStatus: zc,
    useFormState: D0,
    useActionState: D0,
    useOptimistic: function(e, n) {
      var a = rt();
      return M0(a, Ue, e, n);
    },
    useMemoCache: Ec,
    useCacheRefresh: J0
  };
  Dc.useEffectEvent = U0;
  var n1 = {
    readContext: mt,
    use: Dl,
    useCallback: X0,
    useContext: mt,
    useEffect: Cc,
    useImperativeHandle: k0,
    useInsertionEffect: G0,
    useLayoutEffect: V0,
    useMemo: I0,
    useReducer: Ac,
    useRef: j0,
    useState: function() {
      return Ac(Gn);
    },
    useDebugValue: qc,
    useDeferredValue: function(e, n) {
      var a = rt();
      return Ue === null ? Rc(a, e, n) : Z0(
        a,
        Ue.memoizedState,
        e,
        n
      );
    },
    useTransition: function() {
      var e = Ac(Gn)[0], n = rt().memoizedState;
      return [
        typeof e == "boolean" ? e : $i(e),
        n
      ];
    },
    useSyncExternalStore: x0,
    useId: F0,
    useHostTransitionStatus: zc,
    useFormState: H0,
    useActionState: H0,
    useOptimistic: function(e, n) {
      var a = rt();
      return Ue !== null ? M0(a, Ue, e, n) : (a.baseState = e, [e, a.queue.dispatch]);
    },
    useMemoCache: Ec,
    useCacheRefresh: J0
  };
  n1.useEffectEvent = U0;
  function Lc(e, n, a, o) {
    n = e.memoizedState, a = a(o, n), a = a == null ? n : y({}, n, a), e.memoizedState = a, e.lanes === 0 && (e.updateQueue.baseState = a);
  }
  var Hc = {
    enqueueSetState: function(e, n, a) {
      e = e._reactInternals;
      var o = Zt(), d = gr(o);
      d.payload = n, a != null && (d.callback = a), n = vr(e, d, o), n !== null && (jt(n, e, o), Ii(n, e, o));
    },
    enqueueReplaceState: function(e, n, a) {
      e = e._reactInternals;
      var o = Zt(), d = gr(o);
      d.tag = 1, d.payload = n, a != null && (d.callback = a), n = vr(e, d, o), n !== null && (jt(n, e, o), Ii(n, e, o));
    },
    enqueueForceUpdate: function(e, n) {
      e = e._reactInternals;
      var a = Zt(), o = gr(a);
      o.tag = 2, n != null && (o.callback = n), n = vr(e, o, a), n !== null && (jt(n, e, a), Ii(n, e, a));
    }
  };
  function r1(e, n, a, o, d, p, x) {
    return e = e.stateNode, typeof e.shouldComponentUpdate == "function" ? e.shouldComponentUpdate(o, p, x) : n.prototype && n.prototype.isPureReactComponent ? !ji(a, o) || !ji(d, p) : !0;
  }
  function a1(e, n, a, o) {
    e = n.state, typeof n.componentWillReceiveProps == "function" && n.componentWillReceiveProps(a, o), typeof n.UNSAFE_componentWillReceiveProps == "function" && n.UNSAFE_componentWillReceiveProps(a, o), n.state !== e && Hc.enqueueReplaceState(n, n.state, null);
  }
  function Wr(e, n) {
    var a = n;
    if ("ref" in n) {
      a = {};
      for (var o in n)
        o !== "ref" && (a[o] = n[o]);
    }
    if (e = e.defaultProps) {
      a === n && (a = y({}, a));
      for (var d in e)
        a[d] === void 0 && (a[d] = e[d]);
    }
    return a;
  }
  function i1(e) {
    pl(e);
  }
  function u1(e) {
    console.error(e);
  }
  function l1(e) {
    pl(e);
  }
  function Ul(e, n) {
    try {
      var a = e.onUncaughtError;
      a(n.value, { componentStack: n.stack });
    } catch (o) {
      setTimeout(function() {
        throw o;
      });
    }
  }
  function o1(e, n, a) {
    try {
      var o = e.onCaughtError;
      o(a.value, {
        componentStack: a.stack,
        errorBoundary: n.tag === 1 ? n.stateNode : null
      });
    } catch (d) {
      setTimeout(function() {
        throw d;
      });
    }
  }
  function jc(e, n, a) {
    return a = gr(a), a.tag = 3, a.payload = { element: null }, a.callback = function() {
      Ul(e, n);
    }, a;
  }
  function s1(e) {
    return e = gr(e), e.tag = 3, e;
  }
  function c1(e, n, a, o) {
    var d = a.type.getDerivedStateFromError;
    if (typeof d == "function") {
      var p = o.value;
      e.payload = function() {
        return d(p);
      }, e.callback = function() {
        o1(n, a, o);
      };
    }
    var x = a.stateNode;
    x !== null && typeof x.componentDidCatch == "function" && (e.callback = function() {
      o1(n, a, o), typeof d != "function" && (xr === null ? xr = /* @__PURE__ */ new Set([this]) : xr.add(this));
      var C = o.stack;
      this.componentDidCatch(o.value, {
        componentStack: C !== null ? C : ""
      });
    });
  }
  function Hq(e, n, a, o, d) {
    if (a.flags |= 32768, o !== null && typeof o == "object" && typeof o.then == "function") {
      if (n = a.alternate, n !== null && Ha(
        n,
        a,
        d,
        !0
      ), a = Yt.current, a !== null) {
        switch (a.tag) {
          case 31:
          case 13:
            return en === null ? Jl() : a.alternate === null && et === 0 && (et = 3), a.flags &= -257, a.flags |= 65536, a.lanes = d, o === Tl ? a.flags |= 16384 : (n = a.updateQueue, n === null ? a.updateQueue = /* @__PURE__ */ new Set([o]) : n.add(o), sf(e, o, d)), !1;
          case 22:
            return a.flags |= 65536, o === Tl ? a.flags |= 16384 : (n = a.updateQueue, n === null ? (n = {
              transitions: null,
              markerInstances: null,
              retryQueue: /* @__PURE__ */ new Set([o])
            }, a.updateQueue = n) : (a = n.retryQueue, a === null ? n.retryQueue = /* @__PURE__ */ new Set([o]) : a.add(o)), sf(e, o, d)), !1;
        }
        throw Error(u(435, a.tag));
      }
      return sf(e, o, d), Jl(), !1;
    }
    if (ze)
      return n = Yt.current, n !== null ? ((n.flags & 65536) === 0 && (n.flags |= 256), n.flags |= 65536, n.lanes = d, o !== nc && (e = Error(u(422), { cause: o }), Gi(Ft(e, a)))) : (o !== nc && (n = Error(u(423), {
        cause: o
      }), Gi(
        Ft(n, a)
      )), e = e.current.alternate, e.flags |= 65536, d &= -d, e.lanes |= d, o = Ft(o, a), d = jc(
        e.stateNode,
        o,
        d
      ), hc(e, d), et !== 4 && (et = 2)), !1;
    var p = Error(u(520), { cause: o });
    if (p = Ft(p, a), iu === null ? iu = [p] : iu.push(p), et !== 4 && (et = 2), n === null) return !0;
    o = Ft(o, a), a = n;
    do {
      switch (a.tag) {
        case 3:
          return a.flags |= 65536, e = d & -d, a.lanes |= e, e = jc(a.stateNode, o, e), hc(a, e), !1;
        case 1:
          if (n = a.type, p = a.stateNode, (a.flags & 128) === 0 && (typeof n.getDerivedStateFromError == "function" || p !== null && typeof p.componentDidCatch == "function" && (xr === null || !xr.has(p))))
            return a.flags |= 65536, d &= -d, a.lanes |= d, d = s1(d), c1(
              d,
              e,
              a,
              o
            ), hc(a, d), !1;
      }
      a = a.return;
    } while (a !== null);
    return !1;
  }
  var Bc = Error(u(461)), lt = !1;
  function bt(e, n, a, o) {
    n.child = e === null ? g0(n, null, a, o) : Jr(
      n,
      e.child,
      a,
      o
    );
  }
  function f1(e, n, a, o, d) {
    a = a.render;
    var p = n.ref;
    if ("ref" in o) {
      var x = {};
      for (var C in o)
        C !== "ref" && (x[C] = o[C]);
    } else x = o;
    return Kr(n), o = bc(
      e,
      n,
      a,
      x,
      p,
      d
    ), C = _c(), e !== null && !lt ? (xc(e, n, d), Vn(e, n, d)) : (ze && C && ec(n), n.flags |= 1, bt(e, n, o, d), n.child);
  }
  function d1(e, n, a, o, d) {
    if (e === null) {
      var p = a.type;
      return typeof p == "function" && !Js(p) && p.defaultProps === void 0 && a.compare === null ? (n.tag = 15, n.type = p, h1(
        e,
        n,
        p,
        o,
        d
      )) : (e = _l(
        a.type,
        null,
        o,
        n,
        n.mode,
        d
      ), e.ref = n.ref, e.return = n, n.child = e);
    }
    if (p = e.child, !Zc(e, d)) {
      var x = p.memoizedProps;
      if (a = a.compare, a = a !== null ? a : ji, a(x, o) && e.ref === n.ref)
        return Vn(e, n, d);
    }
    return n.flags |= 1, e = Ln(p, o), e.ref = n.ref, e.return = n, n.child = e;
  }
  function h1(e, n, a, o, d) {
    if (e !== null) {
      var p = e.memoizedProps;
      if (ji(p, o) && e.ref === n.ref)
        if (lt = !1, n.pendingProps = o = p, Zc(e, d))
          (e.flags & 131072) !== 0 && (lt = !0);
        else
          return n.lanes = e.lanes, Vn(e, n, d);
    }
    return Uc(
      e,
      n,
      a,
      o,
      d
    );
  }
  function g1(e, n, a, o) {
    var d = o.children, p = e !== null ? e.memoizedState : null;
    if (e === null && n.stateNode === null && (n.stateNode = {
      _visibility: 1,
      _pendingMarkers: null,
      _retryCache: null,
      _transitions: null
    }), o.mode === "hidden") {
      if ((n.flags & 128) !== 0) {
        if (p = p !== null ? p.baseLanes | a : a, e !== null) {
          for (o = n.child = e.child, d = 0; o !== null; )
            d = d | o.lanes | o.childLanes, o = o.sibling;
          o = d & ~p;
        } else o = 0, n.child = null;
        return v1(
          e,
          n,
          p,
          a,
          o
        );
      }
      if ((a & 536870912) !== 0)
        n.memoizedState = { baseLanes: 0, cachePool: null }, e !== null && wl(
          n,
          p !== null ? p.cachePool : null
        ), p !== null ? y0(n, p) : vc(), m0(n);
      else
        return o = n.lanes = 536870912, v1(
          e,
          n,
          p !== null ? p.baseLanes | a : a,
          a,
          o
        );
    } else
      p !== null ? (wl(n, p.cachePool), y0(n, p), yr(), n.memoizedState = null) : (e !== null && wl(n, null), vc(), yr());
    return bt(e, n, d, a), n.child;
  }
  function Pi(e, n) {
    return e !== null && e.tag === 22 || n.stateNode !== null || (n.stateNode = {
      _visibility: 1,
      _pendingMarkers: null,
      _retryCache: null,
      _transitions: null
    }), n.sibling;
  }
  function v1(e, n, a, o, d) {
    var p = sc();
    return p = p === null ? null : { parent: it._currentValue, pool: p }, n.memoizedState = {
      baseLanes: a,
      cachePool: p
    }, e !== null && wl(n, null), vc(), m0(n), e !== null && Ha(e, n, o, !0), n.childLanes = d, null;
  }
  function Gl(e, n) {
    return n = Yl(
      { mode: n.mode, children: n.children },
      e.mode
    ), n.ref = e.ref, e.child = n, n.return = e, n;
  }
  function p1(e, n, a) {
    return Jr(n, e.child, null, a), e = Gl(n, n.pendingProps), e.flags |= 2, kt(n), n.memoizedState = null, e;
  }
  function jq(e, n, a) {
    var o = n.pendingProps, d = (n.flags & 128) !== 0;
    if (n.flags &= -129, e === null) {
      if (ze) {
        if (o.mode === "hidden")
          return e = Gl(n, o), n.lanes = 536870912, Pi(null, e);
        if (yc(n), (e = Xe) ? (e = Cb(
          e,
          Wt
        ), e = e !== null && e.data === "&" ? e : null, e !== null && (n.memoizedState = {
          dehydrated: e,
          treeContext: sr !== null ? { id: _n, overflow: xn } : null,
          retryLane: 536870912,
          hydrationErrors: null
        }, a = Wm(e), a.return = n, n.child = a, yt = n, Xe = null)) : e = null, e === null) throw fr(n);
        return n.lanes = 536870912, null;
      }
      return Gl(n, o);
    }
    var p = e.memoizedState;
    if (p !== null) {
      var x = p.dehydrated;
      if (yc(n), d)
        if (n.flags & 256)
          n.flags &= -257, n = p1(
            e,
            n,
            a
          );
        else if (n.memoizedState !== null)
          n.child = e.child, n.flags |= 128, n = null;
        else throw Error(u(558));
      else if (lt || Ha(e, n, a, !1), d = (a & e.childLanes) !== 0, lt || d) {
        if (o = Ye, o !== null && (x = Wu(o, a), x !== 0 && x !== p.retryLane))
          throw p.retryLane = x, kr(e, x), jt(o, e, x), Bc;
        Jl(), n = p1(
          e,
          n,
          a
        );
      } else
        e = p.treeContext, Xe = tn(x.nextSibling), yt = n, ze = !0, cr = null, Wt = !1, e !== null && n0(n, e), n = Gl(n, o), n.flags |= 4096;
      return n;
    }
    return e = Ln(e.child, {
      mode: o.mode,
      children: o.children
    }), e.ref = n.ref, n.child = e, e.return = n, e;
  }
  function Vl(e, n) {
    var a = n.ref;
    if (a === null)
      e !== null && e.ref !== null && (n.flags |= 4194816);
    else {
      if (typeof a != "function" && typeof a != "object")
        throw Error(u(284));
      (e === null || e.ref !== a) && (n.flags |= 4194816);
    }
  }
  function Uc(e, n, a, o, d) {
    return Kr(n), a = bc(
      e,
      n,
      a,
      o,
      void 0,
      d
    ), o = _c(), e !== null && !lt ? (xc(e, n, d), Vn(e, n, d)) : (ze && o && ec(n), n.flags |= 1, bt(e, n, a, d), n.child);
  }
  function y1(e, n, a, o, d, p) {
    return Kr(n), n.updateQueue = null, a = _0(
      n,
      o,
      a,
      d
    ), b0(e), o = _c(), e !== null && !lt ? (xc(e, n, p), Vn(e, n, p)) : (ze && o && ec(n), n.flags |= 1, bt(e, n, a, p), n.child);
  }
  function m1(e, n, a, o, d) {
    if (Kr(n), n.stateNode === null) {
      var p = za, x = a.contextType;
      typeof x == "object" && x !== null && (p = mt(x)), p = new a(o, p), n.memoizedState = p.state !== null && p.state !== void 0 ? p.state : null, p.updater = Hc, n.stateNode = p, p._reactInternals = n, p = n.stateNode, p.props = o, p.state = n.memoizedState, p.refs = {}, fc(n), x = a.contextType, p.context = typeof x == "object" && x !== null ? mt(x) : za, p.state = n.memoizedState, x = a.getDerivedStateFromProps, typeof x == "function" && (Lc(
        n,
        a,
        x,
        o
      ), p.state = n.memoizedState), typeof a.getDerivedStateFromProps == "function" || typeof p.getSnapshotBeforeUpdate == "function" || typeof p.UNSAFE_componentWillMount != "function" && typeof p.componentWillMount != "function" || (x = p.state, typeof p.componentWillMount == "function" && p.componentWillMount(), typeof p.UNSAFE_componentWillMount == "function" && p.UNSAFE_componentWillMount(), x !== p.state && Hc.enqueueReplaceState(p, p.state, null), Ki(n, o, p, d), Zi(), p.state = n.memoizedState), typeof p.componentDidMount == "function" && (n.flags |= 4194308), o = !0;
    } else if (e === null) {
      p = n.stateNode;
      var C = n.memoizedProps, V = Wr(a, C);
      p.props = V;
      var te = p.context, oe = a.contextType;
      x = za, typeof oe == "object" && oe !== null && (x = mt(oe));
      var fe = a.getDerivedStateFromProps;
      oe = typeof fe == "function" || typeof p.getSnapshotBeforeUpdate == "function", C = n.pendingProps !== C, oe || typeof p.UNSAFE_componentWillReceiveProps != "function" && typeof p.componentWillReceiveProps != "function" || (C || te !== x) && a1(
        n,
        p,
        o,
        x
      ), hr = !1;
      var ne = n.memoizedState;
      p.state = ne, Ki(n, o, p, d), Zi(), te = n.memoizedState, C || ne !== te || hr ? (typeof fe == "function" && (Lc(
        n,
        a,
        fe,
        o
      ), te = n.memoizedState), (V = hr || r1(
        n,
        a,
        V,
        o,
        ne,
        te,
        x
      )) ? (oe || typeof p.UNSAFE_componentWillMount != "function" && typeof p.componentWillMount != "function" || (typeof p.componentWillMount == "function" && p.componentWillMount(), typeof p.UNSAFE_componentWillMount == "function" && p.UNSAFE_componentWillMount()), typeof p.componentDidMount == "function" && (n.flags |= 4194308)) : (typeof p.componentDidMount == "function" && (n.flags |= 4194308), n.memoizedProps = o, n.memoizedState = te), p.props = o, p.state = te, p.context = x, o = V) : (typeof p.componentDidMount == "function" && (n.flags |= 4194308), o = !1);
    } else {
      p = n.stateNode, dc(e, n), x = n.memoizedProps, oe = Wr(a, x), p.props = oe, fe = n.pendingProps, ne = p.context, te = a.contextType, V = za, typeof te == "object" && te !== null && (V = mt(te)), C = a.getDerivedStateFromProps, (te = typeof C == "function" || typeof p.getSnapshotBeforeUpdate == "function") || typeof p.UNSAFE_componentWillReceiveProps != "function" && typeof p.componentWillReceiveProps != "function" || (x !== fe || ne !== V) && a1(
        n,
        p,
        o,
        V
      ), hr = !1, ne = n.memoizedState, p.state = ne, Ki(n, o, p, d), Zi();
      var le = n.memoizedState;
      x !== fe || ne !== le || hr || e !== null && e.dependencies !== null && Sl(e.dependencies) ? (typeof C == "function" && (Lc(
        n,
        a,
        C,
        o
      ), le = n.memoizedState), (oe = hr || r1(
        n,
        a,
        oe,
        o,
        ne,
        le,
        V
      ) || e !== null && e.dependencies !== null && Sl(e.dependencies)) ? (te || typeof p.UNSAFE_componentWillUpdate != "function" && typeof p.componentWillUpdate != "function" || (typeof p.componentWillUpdate == "function" && p.componentWillUpdate(o, le, V), typeof p.UNSAFE_componentWillUpdate == "function" && p.UNSAFE_componentWillUpdate(
        o,
        le,
        V
      )), typeof p.componentDidUpdate == "function" && (n.flags |= 4), typeof p.getSnapshotBeforeUpdate == "function" && (n.flags |= 1024)) : (typeof p.componentDidUpdate != "function" || x === e.memoizedProps && ne === e.memoizedState || (n.flags |= 4), typeof p.getSnapshotBeforeUpdate != "function" || x === e.memoizedProps && ne === e.memoizedState || (n.flags |= 1024), n.memoizedProps = o, n.memoizedState = le), p.props = o, p.state = le, p.context = V, o = oe) : (typeof p.componentDidUpdate != "function" || x === e.memoizedProps && ne === e.memoizedState || (n.flags |= 4), typeof p.getSnapshotBeforeUpdate != "function" || x === e.memoizedProps && ne === e.memoizedState || (n.flags |= 1024), o = !1);
    }
    return p = o, Vl(e, n), o = (n.flags & 128) !== 0, p || o ? (p = n.stateNode, a = o && typeof a.getDerivedStateFromError != "function" ? null : p.render(), n.flags |= 1, e !== null && o ? (n.child = Jr(
      n,
      e.child,
      null,
      d
    ), n.child = Jr(
      n,
      null,
      a,
      d
    )) : bt(e, n, a, d), n.memoizedState = p.state, e = n.child) : e = Vn(
      e,
      n,
      d
    ), e;
  }
  function b1(e, n, a, o) {
    return Ir(), n.flags |= 256, bt(e, n, a, o), n.child;
  }
  var Gc = {
    dehydrated: null,
    treeContext: null,
    retryLane: 0,
    hydrationErrors: null
  };
  function Vc(e) {
    return { baseLanes: e, cachePool: o0() };
  }
  function Yc(e, n, a) {
    return e = e !== null ? e.childLanes & ~a : 0, n && (e |= It), e;
  }
  function _1(e, n, a) {
    var o = n.pendingProps, d = !1, p = (n.flags & 128) !== 0, x;
    if ((x = p) || (x = e !== null && e.memoizedState === null ? !1 : (nt.current & 2) !== 0), x && (d = !0, n.flags &= -129), x = (n.flags & 32) !== 0, n.flags &= -33, e === null) {
      if (ze) {
        if (d ? pr(n) : yr(), (e = Xe) ? (e = Cb(
          e,
          Wt
        ), e = e !== null && e.data !== "&" ? e : null, e !== null && (n.memoizedState = {
          dehydrated: e,
          treeContext: sr !== null ? { id: _n, overflow: xn } : null,
          retryLane: 536870912,
          hydrationErrors: null
        }, a = Wm(e), a.return = n, n.child = a, yt = n, Xe = null)) : e = null, e === null) throw fr(n);
        return Af(e) ? n.lanes = 32 : n.lanes = 536870912, null;
      }
      var C = o.children;
      return o = o.fallback, d ? (yr(), d = n.mode, C = Yl(
        { mode: "hidden", children: C },
        d
      ), o = Xr(
        o,
        d,
        a,
        null
      ), C.return = n, o.return = n, C.sibling = o, n.child = C, o = n.child, o.memoizedState = Vc(a), o.childLanes = Yc(
        e,
        x,
        a
      ), n.memoizedState = Gc, Pi(null, o)) : (pr(n), kc(n, C));
    }
    var V = e.memoizedState;
    if (V !== null && (C = V.dehydrated, C !== null)) {
      if (p)
        n.flags & 256 ? (pr(n), n.flags &= -257, n = Xc(
          e,
          n,
          a
        )) : n.memoizedState !== null ? (yr(), n.child = e.child, n.flags |= 128, n = null) : (yr(), C = o.fallback, d = n.mode, o = Yl(
          { mode: "visible", children: o.children },
          d
        ), C = Xr(
          C,
          d,
          a,
          null
        ), C.flags |= 2, o.return = n, C.return = n, o.sibling = C, n.child = o, Jr(
          n,
          e.child,
          null,
          a
        ), o = n.child, o.memoizedState = Vc(a), o.childLanes = Yc(
          e,
          x,
          a
        ), n.memoizedState = Gc, n = Pi(null, o));
      else if (pr(n), Af(C)) {
        if (x = C.nextSibling && C.nextSibling.dataset, x) var te = x.dgst;
        x = te, o = Error(u(419)), o.stack = "", o.digest = x, Gi({ value: o, source: null, stack: null }), n = Xc(
          e,
          n,
          a
        );
      } else if (lt || Ha(e, n, a, !1), x = (a & e.childLanes) !== 0, lt || x) {
        if (x = Ye, x !== null && (o = Wu(x, a), o !== 0 && o !== V.retryLane))
          throw V.retryLane = o, kr(e, o), jt(x, e, o), Bc;
        wf(C) || Jl(), n = Xc(
          e,
          n,
          a
        );
      } else
        wf(C) ? (n.flags |= 192, n.child = e.child, n = null) : (e = V.treeContext, Xe = tn(
          C.nextSibling
        ), yt = n, ze = !0, cr = null, Wt = !1, e !== null && n0(n, e), n = kc(
          n,
          o.children
        ), n.flags |= 4096);
      return n;
    }
    return d ? (yr(), C = o.fallback, d = n.mode, V = e.child, te = V.sibling, o = Ln(V, {
      mode: "hidden",
      children: o.children
    }), o.subtreeFlags = V.subtreeFlags & 65011712, te !== null ? C = Ln(
      te,
      C
    ) : (C = Xr(
      C,
      d,
      a,
      null
    ), C.flags |= 2), C.return = n, o.return = n, o.sibling = C, n.child = o, Pi(null, o), o = n.child, C = e.child.memoizedState, C === null ? C = Vc(a) : (d = C.cachePool, d !== null ? (V = it._currentValue, d = d.parent !== V ? { parent: V, pool: V } : d) : d = o0(), C = {
      baseLanes: C.baseLanes | a,
      cachePool: d
    }), o.memoizedState = C, o.childLanes = Yc(
      e,
      x,
      a
    ), n.memoizedState = Gc, Pi(e.child, o)) : (pr(n), a = e.child, e = a.sibling, a = Ln(a, {
      mode: "visible",
      children: o.children
    }), a.return = n, a.sibling = null, e !== null && (x = n.deletions, x === null ? (n.deletions = [e], n.flags |= 16) : x.push(e)), n.child = a, n.memoizedState = null, a);
  }
  function kc(e, n) {
    return n = Yl(
      { mode: "visible", children: n },
      e.mode
    ), n.return = e, e.child = n;
  }
  function Yl(e, n) {
    return e = Vt(22, e, null, n), e.lanes = 0, e;
  }
  function Xc(e, n, a) {
    return Jr(n, e.child, null, a), e = kc(
      n,
      n.pendingProps.children
    ), e.flags |= 2, n.memoizedState = null, e;
  }
  function x1(e, n, a) {
    e.lanes |= n;
    var o = e.alternate;
    o !== null && (o.lanes |= n), ic(e.return, n, a);
  }
  function Ic(e, n, a, o, d, p) {
    var x = e.memoizedState;
    x === null ? e.memoizedState = {
      isBackwards: n,
      rendering: null,
      renderingStartTime: 0,
      last: o,
      tail: a,
      tailMode: d,
      treeForkCount: p
    } : (x.isBackwards = n, x.rendering = null, x.renderingStartTime = 0, x.last = o, x.tail = a, x.tailMode = d, x.treeForkCount = p);
  }
  function S1(e, n, a) {
    var o = n.pendingProps, d = o.revealOrder, p = o.tail;
    o = o.children;
    var x = nt.current, C = (x & 2) !== 0;
    if (C ? (x = x & 1 | 2, n.flags |= 128) : x &= 1, ie(nt, x), bt(e, n, o, a), o = ze ? Ui : 0, !C && e !== null && (e.flags & 128) !== 0)
      e: for (e = n.child; e !== null; ) {
        if (e.tag === 13)
          e.memoizedState !== null && x1(e, a, n);
        else if (e.tag === 19)
          x1(e, a, n);
        else if (e.child !== null) {
          e.child.return = e, e = e.child;
          continue;
        }
        if (e === n) break e;
        for (; e.sibling === null; ) {
          if (e.return === null || e.return === n)
            break e;
          e = e.return;
        }
        e.sibling.return = e.return, e = e.sibling;
      }
    switch (d) {
      case "forwards":
        for (a = n.child, d = null; a !== null; )
          e = a.alternate, e !== null && Rl(e) === null && (d = a), a = a.sibling;
        a = d, a === null ? (d = n.child, n.child = null) : (d = a.sibling, a.sibling = null), Ic(
          n,
          !1,
          d,
          a,
          p,
          o
        );
        break;
      case "backwards":
      case "unstable_legacy-backwards":
        for (a = null, d = n.child, n.child = null; d !== null; ) {
          if (e = d.alternate, e !== null && Rl(e) === null) {
            n.child = d;
            break;
          }
          e = d.sibling, d.sibling = a, a = d, d = e;
        }
        Ic(
          n,
          !0,
          a,
          null,
          p,
          o
        );
        break;
      case "together":
        Ic(
          n,
          !1,
          null,
          null,
          void 0,
          o
        );
        break;
      default:
        n.memoizedState = null;
    }
    return n.child;
  }
  function Vn(e, n, a) {
    if (e !== null && (n.dependencies = e.dependencies), _r |= n.lanes, (a & n.childLanes) === 0)
      if (e !== null) {
        if (Ha(
          e,
          n,
          a,
          !1
        ), (a & n.childLanes) === 0)
          return null;
      } else return null;
    if (e !== null && n.child !== e.child)
      throw Error(u(153));
    if (n.child !== null) {
      for (e = n.child, a = Ln(e, e.pendingProps), n.child = a, a.return = n; e.sibling !== null; )
        e = e.sibling, a = a.sibling = Ln(e, e.pendingProps), a.return = n;
      a.sibling = null;
    }
    return n.child;
  }
  function Zc(e, n) {
    return (e.lanes & n) !== 0 ? !0 : (e = e.dependencies, !!(e !== null && Sl(e)));
  }
  function Bq(e, n, a) {
    switch (n.tag) {
      case 3:
        W(n, n.stateNode.containerInfo), dr(n, it, e.memoizedState.cache), Ir();
        break;
      case 27:
      case 5:
        de(n);
        break;
      case 4:
        W(n, n.stateNode.containerInfo);
        break;
      case 10:
        dr(
          n,
          n.type,
          n.memoizedProps.value
        );
        break;
      case 31:
        if (n.memoizedState !== null)
          return n.flags |= 128, yc(n), null;
        break;
      case 13:
        var o = n.memoizedState;
        if (o !== null)
          return o.dehydrated !== null ? (pr(n), n.flags |= 128, null) : (a & n.child.childLanes) !== 0 ? _1(e, n, a) : (pr(n), e = Vn(
            e,
            n,
            a
          ), e !== null ? e.sibling : null);
        pr(n);
        break;
      case 19:
        var d = (e.flags & 128) !== 0;
        if (o = (a & n.childLanes) !== 0, o || (Ha(
          e,
          n,
          a,
          !1
        ), o = (a & n.childLanes) !== 0), d) {
          if (o)
            return S1(
              e,
              n,
              a
            );
          n.flags |= 128;
        }
        if (d = n.memoizedState, d !== null && (d.rendering = null, d.tail = null, d.lastEffect = null), ie(nt, nt.current), o) break;
        return null;
      case 22:
        return n.lanes = 0, g1(
          e,
          n,
          a,
          n.pendingProps
        );
      case 24:
        dr(n, it, e.memoizedState.cache);
    }
    return Vn(e, n, a);
  }
  function E1(e, n, a) {
    if (e !== null)
      if (e.memoizedProps !== n.pendingProps)
        lt = !0;
      else {
        if (!Zc(e, a) && (n.flags & 128) === 0)
          return lt = !1, Bq(
            e,
            n,
            a
          );
        lt = (e.flags & 131072) !== 0;
      }
    else
      lt = !1, ze && (n.flags & 1048576) !== 0 && t0(n, Ui, n.index);
    switch (n.lanes = 0, n.tag) {
      case 16:
        e: {
          var o = n.pendingProps;
          if (e = $r(n.elementType), n.type = e, typeof e == "function")
            Js(e) ? (o = Wr(e, o), n.tag = 1, n = m1(
              null,
              n,
              e,
              o,
              a
            )) : (n.tag = 0, n = Uc(
              null,
              n,
              e,
              o,
              a
            ));
          else {
            if (e != null) {
              var d = e.$$typeof;
              if (d === T) {
                n.tag = 11, n = f1(
                  null,
                  n,
                  e,
                  o,
                  a
                );
                break e;
              } else if (d === q) {
                n.tag = 14, n = d1(
                  null,
                  n,
                  e,
                  o,
                  a
                );
                break e;
              }
            }
            throw n = D(e) || e, Error(u(306, n, ""));
          }
        }
        return n;
      case 0:
        return Uc(
          e,
          n,
          n.type,
          n.pendingProps,
          a
        );
      case 1:
        return o = n.type, d = Wr(
          o,
          n.pendingProps
        ), m1(
          e,
          n,
          o,
          d,
          a
        );
      case 3:
        e: {
          if (W(
            n,
            n.stateNode.containerInfo
          ), e === null) throw Error(u(387));
          o = n.pendingProps;
          var p = n.memoizedState;
          d = p.element, dc(e, n), Ki(n, o, null, a);
          var x = n.memoizedState;
          if (o = x.cache, dr(n, it, o), o !== p.cache && uc(
            n,
            [it],
            a,
            !0
          ), Zi(), o = x.element, p.isDehydrated)
            if (p = {
              element: o,
              isDehydrated: !1,
              cache: x.cache
            }, n.updateQueue.baseState = p, n.memoizedState = p, n.flags & 256) {
              n = b1(
                e,
                n,
                o,
                a
              );
              break e;
            } else if (o !== d) {
              d = Ft(
                Error(u(424)),
                n
              ), Gi(d), n = b1(
                e,
                n,
                o,
                a
              );
              break e;
            } else {
              switch (e = n.stateNode.containerInfo, e.nodeType) {
                case 9:
                  e = e.body;
                  break;
                default:
                  e = e.nodeName === "HTML" ? e.ownerDocument.body : e;
              }
              for (Xe = tn(e.firstChild), yt = n, ze = !0, cr = null, Wt = !0, a = g0(
                n,
                null,
                o,
                a
              ), n.child = a; a; )
                a.flags = a.flags & -3 | 4096, a = a.sibling;
            }
          else {
            if (Ir(), o === d) {
              n = Vn(
                e,
                n,
                a
              );
              break e;
            }
            bt(e, n, o, a);
          }
          n = n.child;
        }
        return n;
      case 26:
        return Vl(e, n), e === null ? (a = Db(
          n.type,
          null,
          n.pendingProps,
          null
        )) ? n.memoizedState = a : ze || (a = n.type, e = n.pendingProps, o = ao(
          P.current
        ).createElement(a), o[dt] = n, o[wt] = e, _t(o, a, e), at(o), n.stateNode = o) : n.memoizedState = Db(
          n.type,
          e.memoizedProps,
          n.pendingProps,
          e.memoizedState
        ), null;
      case 27:
        return de(n), e === null && ze && (o = n.stateNode = Nb(
          n.type,
          n.pendingProps,
          P.current
        ), yt = n, Wt = !0, d = Xe, Ar(n.type) ? (Tf = d, Xe = tn(o.firstChild)) : Xe = d), bt(
          e,
          n,
          n.pendingProps.children,
          a
        ), Vl(e, n), e === null && (n.flags |= 4194304), n.child;
      case 5:
        return e === null && ze && ((d = o = Xe) && (o = gR(
          o,
          n.type,
          n.pendingProps,
          Wt
        ), o !== null ? (n.stateNode = o, yt = n, Xe = tn(o.firstChild), Wt = !1, d = !0) : d = !1), d || fr(n)), de(n), d = n.type, p = n.pendingProps, x = e !== null ? e.memoizedProps : null, o = p.children, xf(d, p) ? o = null : x !== null && xf(d, x) && (n.flags |= 32), n.memoizedState !== null && (d = bc(
          e,
          n,
          qq,
          null,
          null,
          a
        ), hu._currentValue = d), Vl(e, n), bt(e, n, o, a), n.child;
      case 6:
        return e === null && ze && ((e = a = Xe) && (a = vR(
          a,
          n.pendingProps,
          Wt
        ), a !== null ? (n.stateNode = a, yt = n, Xe = null, e = !0) : e = !1), e || fr(n)), null;
      case 13:
        return _1(e, n, a);
      case 4:
        return W(
          n,
          n.stateNode.containerInfo
        ), o = n.pendingProps, e === null ? n.child = Jr(
          n,
          null,
          o,
          a
        ) : bt(e, n, o, a), n.child;
      case 11:
        return f1(
          e,
          n,
          n.type,
          n.pendingProps,
          a
        );
      case 7:
        return bt(
          e,
          n,
          n.pendingProps,
          a
        ), n.child;
      case 8:
        return bt(
          e,
          n,
          n.pendingProps.children,
          a
        ), n.child;
      case 12:
        return bt(
          e,
          n,
          n.pendingProps.children,
          a
        ), n.child;
      case 10:
        return o = n.pendingProps, dr(n, n.type, o.value), bt(e, n, o.children, a), n.child;
      case 9:
        return d = n.type._context, o = n.pendingProps.children, Kr(n), d = mt(d), o = o(d), n.flags |= 1, bt(e, n, o, a), n.child;
      case 14:
        return d1(
          e,
          n,
          n.type,
          n.pendingProps,
          a
        );
      case 15:
        return h1(
          e,
          n,
          n.type,
          n.pendingProps,
          a
        );
      case 19:
        return S1(e, n, a);
      case 31:
        return jq(e, n, a);
      case 22:
        return g1(
          e,
          n,
          a,
          n.pendingProps
        );
      case 24:
        return Kr(n), o = mt(it), e === null ? (d = sc(), d === null && (d = Ye, p = lc(), d.pooledCache = p, p.refCount++, p !== null && (d.pooledCacheLanes |= a), d = p), n.memoizedState = { parent: o, cache: d }, fc(n), dr(n, it, d)) : ((e.lanes & a) !== 0 && (dc(e, n), Ki(n, null, null, a), Zi()), d = e.memoizedState, p = n.memoizedState, d.parent !== o ? (d = { parent: o, cache: o }, n.memoizedState = d, n.lanes === 0 && (n.memoizedState = n.updateQueue.baseState = d), dr(n, it, o)) : (o = p.cache, dr(n, it, o), o !== d.cache && uc(
          n,
          [it],
          a,
          !0
        ))), bt(
          e,
          n,
          n.pendingProps.children,
          a
        ), n.child;
      case 29:
        throw n.pendingProps;
    }
    throw Error(u(156, n.tag));
  }
  function Yn(e) {
    e.flags |= 4;
  }
  function Kc(e, n, a, o, d) {
    if ((n = (e.mode & 32) !== 0) && (n = !1), n) {
      if (e.flags |= 16777216, (d & 335544128) === d)
        if (e.stateNode.complete) e.flags |= 8192;
        else if (F1()) e.flags |= 8192;
        else
          throw Fr = Tl, cc;
    } else e.flags &= -16777217;
  }
  function w1(e, n) {
    if (n.type !== "stylesheet" || (n.state.loading & 4) !== 0)
      e.flags &= -16777217;
    else if (e.flags |= 16777216, !Ub(n))
      if (F1()) e.flags |= 8192;
      else
        throw Fr = Tl, cc;
  }
  function kl(e, n) {
    n !== null && (e.flags |= 4), e.flags & 16384 && (n = e.tag !== 22 ? Fu() : 536870912, e.lanes |= n, Qa |= n);
  }
  function Wi(e, n) {
    if (!ze)
      switch (e.tailMode) {
        case "hidden":
          n = e.tail;
          for (var a = null; n !== null; )
            n.alternate !== null && (a = n), n = n.sibling;
          a === null ? e.tail = null : a.sibling = null;
          break;
        case "collapsed":
          a = e.tail;
          for (var o = null; a !== null; )
            a.alternate !== null && (o = a), a = a.sibling;
          o === null ? n || e.tail === null ? e.tail = null : e.tail.sibling = null : o.sibling = null;
      }
  }
  function Ie(e) {
    var n = e.alternate !== null && e.alternate.child === e.child, a = 0, o = 0;
    if (n)
      for (var d = e.child; d !== null; )
        a |= d.lanes | d.childLanes, o |= d.subtreeFlags & 65011712, o |= d.flags & 65011712, d.return = e, d = d.sibling;
    else
      for (d = e.child; d !== null; )
        a |= d.lanes | d.childLanes, o |= d.subtreeFlags, o |= d.flags, d.return = e, d = d.sibling;
    return e.subtreeFlags |= o, e.childLanes = a, n;
  }
  function Uq(e, n, a) {
    var o = n.pendingProps;
    switch (tc(n), n.tag) {
      case 16:
      case 15:
      case 0:
      case 11:
      case 7:
      case 8:
      case 12:
      case 9:
      case 14:
        return Ie(n), null;
      case 1:
        return Ie(n), null;
      case 3:
        return a = n.stateNode, o = null, e !== null && (o = e.memoizedState.cache), n.memoizedState.cache !== o && (n.flags |= 2048), Bn(it), se(), a.pendingContext && (a.context = a.pendingContext, a.pendingContext = null), (e === null || e.child === null) && (La(n) ? Yn(n) : e === null || e.memoizedState.isDehydrated && (n.flags & 256) === 0 || (n.flags |= 1024, rc())), Ie(n), null;
      case 26:
        var d = n.type, p = n.memoizedState;
        return e === null ? (Yn(n), p !== null ? (Ie(n), w1(n, p)) : (Ie(n), Kc(
          n,
          d,
          null,
          o,
          a
        ))) : p ? p !== e.memoizedState ? (Yn(n), Ie(n), w1(n, p)) : (Ie(n), n.flags &= -16777217) : (e = e.memoizedProps, e !== o && Yn(n), Ie(n), Kc(
          n,
          d,
          e,
          o,
          a
        )), null;
      case 27:
        if (ve(n), a = P.current, d = n.type, e !== null && n.stateNode != null)
          e.memoizedProps !== o && Yn(n);
        else {
          if (!o) {
            if (n.stateNode === null)
              throw Error(u(166));
            return Ie(n), null;
          }
          e = H.current, La(n) ? r0(n) : (e = Nb(d, o, a), n.stateNode = e, Yn(n));
        }
        return Ie(n), null;
      case 5:
        if (ve(n), d = n.type, e !== null && n.stateNode != null)
          e.memoizedProps !== o && Yn(n);
        else {
          if (!o) {
            if (n.stateNode === null)
              throw Error(u(166));
            return Ie(n), null;
          }
          if (p = H.current, La(n))
            r0(n);
          else {
            var x = ao(
              P.current
            );
            switch (p) {
              case 1:
                p = x.createElementNS(
                  "http://www.w3.org/2000/svg",
                  d
                );
                break;
              case 2:
                p = x.createElementNS(
                  "http://www.w3.org/1998/Math/MathML",
                  d
                );
                break;
              default:
                switch (d) {
                  case "svg":
                    p = x.createElementNS(
                      "http://www.w3.org/2000/svg",
                      d
                    );
                    break;
                  case "math":
                    p = x.createElementNS(
                      "http://www.w3.org/1998/Math/MathML",
                      d
                    );
                    break;
                  case "script":
                    p = x.createElement("div"), p.innerHTML = "<script><\/script>", p = p.removeChild(
                      p.firstChild
                    );
                    break;
                  case "select":
                    p = typeof o.is == "string" ? x.createElement("select", {
                      is: o.is
                    }) : x.createElement("select"), o.multiple ? p.multiple = !0 : o.size && (p.size = o.size);
                    break;
                  default:
                    p = typeof o.is == "string" ? x.createElement(d, { is: o.is }) : x.createElement(d);
                }
            }
            p[dt] = n, p[wt] = o;
            e: for (x = n.child; x !== null; ) {
              if (x.tag === 5 || x.tag === 6)
                p.appendChild(x.stateNode);
              else if (x.tag !== 4 && x.tag !== 27 && x.child !== null) {
                x.child.return = x, x = x.child;
                continue;
              }
              if (x === n) break e;
              for (; x.sibling === null; ) {
                if (x.return === null || x.return === n)
                  break e;
                x = x.return;
              }
              x.sibling.return = x.return, x = x.sibling;
            }
            n.stateNode = p;
            e: switch (_t(p, d, o), d) {
              case "button":
              case "input":
              case "select":
              case "textarea":
                o = !!o.autoFocus;
                break e;
              case "img":
                o = !0;
                break e;
              default:
                o = !1;
            }
            o && Yn(n);
          }
        }
        return Ie(n), Kc(
          n,
          n.type,
          e === null ? null : e.memoizedProps,
          n.pendingProps,
          a
        ), null;
      case 6:
        if (e && n.stateNode != null)
          e.memoizedProps !== o && Yn(n);
        else {
          if (typeof o != "string" && n.stateNode === null)
            throw Error(u(166));
          if (e = P.current, La(n)) {
            if (e = n.stateNode, a = n.memoizedProps, o = null, d = yt, d !== null)
              switch (d.tag) {
                case 27:
                case 5:
                  o = d.memoizedProps;
              }
            e[dt] = n, e = !!(e.nodeValue === a || o !== null && o.suppressHydrationWarning === !0 || _b(e.nodeValue, a)), e || fr(n, !0);
          } else
            e = ao(e).createTextNode(
              o
            ), e[dt] = n, n.stateNode = e;
        }
        return Ie(n), null;
      case 31:
        if (a = n.memoizedState, e === null || e.memoizedState !== null) {
          if (o = La(n), a !== null) {
            if (e === null) {
              if (!o) throw Error(u(318));
              if (e = n.memoizedState, e = e !== null ? e.dehydrated : null, !e) throw Error(u(557));
              e[dt] = n;
            } else
              Ir(), (n.flags & 128) === 0 && (n.memoizedState = null), n.flags |= 4;
            Ie(n), e = !1;
          } else
            a = rc(), e !== null && e.memoizedState !== null && (e.memoizedState.hydrationErrors = a), e = !0;
          if (!e)
            return n.flags & 256 ? (kt(n), n) : (kt(n), null);
          if ((n.flags & 128) !== 0)
            throw Error(u(558));
        }
        return Ie(n), null;
      case 13:
        if (o = n.memoizedState, e === null || e.memoizedState !== null && e.memoizedState.dehydrated !== null) {
          if (d = La(n), o !== null && o.dehydrated !== null) {
            if (e === null) {
              if (!d) throw Error(u(318));
              if (d = n.memoizedState, d = d !== null ? d.dehydrated : null, !d) throw Error(u(317));
              d[dt] = n;
            } else
              Ir(), (n.flags & 128) === 0 && (n.memoizedState = null), n.flags |= 4;
            Ie(n), d = !1;
          } else
            d = rc(), e !== null && e.memoizedState !== null && (e.memoizedState.hydrationErrors = d), d = !0;
          if (!d)
            return n.flags & 256 ? (kt(n), n) : (kt(n), null);
        }
        return kt(n), (n.flags & 128) !== 0 ? (n.lanes = a, n) : (a = o !== null, e = e !== null && e.memoizedState !== null, a && (o = n.child, d = null, o.alternate !== null && o.alternate.memoizedState !== null && o.alternate.memoizedState.cachePool !== null && (d = o.alternate.memoizedState.cachePool.pool), p = null, o.memoizedState !== null && o.memoizedState.cachePool !== null && (p = o.memoizedState.cachePool.pool), p !== d && (o.flags |= 2048)), a !== e && a && (n.child.flags |= 8192), kl(n, n.updateQueue), Ie(n), null);
      case 4:
        return se(), e === null && pf(n.stateNode.containerInfo), Ie(n), null;
      case 10:
        return Bn(n.type), Ie(n), null;
      case 19:
        if (G(nt), o = n.memoizedState, o === null) return Ie(n), null;
        if (d = (n.flags & 128) !== 0, p = o.rendering, p === null)
          if (d) Wi(o, !1);
          else {
            if (et !== 0 || e !== null && (e.flags & 128) !== 0)
              for (e = n.child; e !== null; ) {
                if (p = Rl(e), p !== null) {
                  for (n.flags |= 128, Wi(o, !1), e = p.updateQueue, n.updateQueue = e, kl(n, e), n.subtreeFlags = 0, e = a, a = n.child; a !== null; )
                    Pm(a, e), a = a.sibling;
                  return ie(
                    nt,
                    nt.current & 1 | 2
                  ), ze && Hn(n, o.treeForkCount), n.child;
                }
                e = e.sibling;
              }
            o.tail !== null && ke() > Ql && (n.flags |= 128, d = !0, Wi(o, !1), n.lanes = 4194304);
          }
        else {
          if (!d)
            if (e = Rl(p), e !== null) {
              if (n.flags |= 128, d = !0, e = e.updateQueue, n.updateQueue = e, kl(n, e), Wi(o, !0), o.tail === null && o.tailMode === "hidden" && !p.alternate && !ze)
                return Ie(n), null;
            } else
              2 * ke() - o.renderingStartTime > Ql && a !== 536870912 && (n.flags |= 128, d = !0, Wi(o, !1), n.lanes = 4194304);
          o.isBackwards ? (p.sibling = n.child, n.child = p) : (e = o.last, e !== null ? e.sibling = p : n.child = p, o.last = p);
        }
        return o.tail !== null ? (e = o.tail, o.rendering = e, o.tail = e.sibling, o.renderingStartTime = ke(), e.sibling = null, a = nt.current, ie(
          nt,
          d ? a & 1 | 2 : a & 1
        ), ze && Hn(n, o.treeForkCount), e) : (Ie(n), null);
      case 22:
      case 23:
        return kt(n), pc(), o = n.memoizedState !== null, e !== null ? e.memoizedState !== null !== o && (n.flags |= 8192) : o && (n.flags |= 8192), o ? (a & 536870912) !== 0 && (n.flags & 128) === 0 && (Ie(n), n.subtreeFlags & 6 && (n.flags |= 8192)) : Ie(n), a = n.updateQueue, a !== null && kl(n, a.retryQueue), a = null, e !== null && e.memoizedState !== null && e.memoizedState.cachePool !== null && (a = e.memoizedState.cachePool.pool), o = null, n.memoizedState !== null && n.memoizedState.cachePool !== null && (o = n.memoizedState.cachePool.pool), o !== a && (n.flags |= 2048), e !== null && G(Qr), null;
      case 24:
        return a = null, e !== null && (a = e.memoizedState.cache), n.memoizedState.cache !== a && (n.flags |= 2048), Bn(it), Ie(n), null;
      case 25:
        return null;
      case 30:
        return null;
    }
    throw Error(u(156, n.tag));
  }
  function Gq(e, n) {
    switch (tc(n), n.tag) {
      case 1:
        return e = n.flags, e & 65536 ? (n.flags = e & -65537 | 128, n) : null;
      case 3:
        return Bn(it), se(), e = n.flags, (e & 65536) !== 0 && (e & 128) === 0 ? (n.flags = e & -65537 | 128, n) : null;
      case 26:
      case 27:
      case 5:
        return ve(n), null;
      case 31:
        if (n.memoizedState !== null) {
          if (kt(n), n.alternate === null)
            throw Error(u(340));
          Ir();
        }
        return e = n.flags, e & 65536 ? (n.flags = e & -65537 | 128, n) : null;
      case 13:
        if (kt(n), e = n.memoizedState, e !== null && e.dehydrated !== null) {
          if (n.alternate === null)
            throw Error(u(340));
          Ir();
        }
        return e = n.flags, e & 65536 ? (n.flags = e & -65537 | 128, n) : null;
      case 19:
        return G(nt), null;
      case 4:
        return se(), null;
      case 10:
        return Bn(n.type), null;
      case 22:
      case 23:
        return kt(n), pc(), e !== null && G(Qr), e = n.flags, e & 65536 ? (n.flags = e & -65537 | 128, n) : null;
      case 24:
        return Bn(it), null;
      case 25:
        return null;
      default:
        return null;
    }
  }
  function A1(e, n) {
    switch (tc(n), n.tag) {
      case 3:
        Bn(it), se();
        break;
      case 26:
      case 27:
      case 5:
        ve(n);
        break;
      case 4:
        se();
        break;
      case 31:
        n.memoizedState !== null && kt(n);
        break;
      case 13:
        kt(n);
        break;
      case 19:
        G(nt);
        break;
      case 10:
        Bn(n.type);
        break;
      case 22:
      case 23:
        kt(n), pc(), e !== null && G(Qr);
        break;
      case 24:
        Bn(it);
    }
  }
  function eu(e, n) {
    try {
      var a = n.updateQueue, o = a !== null ? a.lastEffect : null;
      if (o !== null) {
        var d = o.next;
        a = d;
        do {
          if ((a.tag & e) === e) {
            o = void 0;
            var p = a.create, x = a.inst;
            o = p(), x.destroy = o;
          }
          a = a.next;
        } while (a !== d);
      }
    } catch (C) {
      Be(n, n.return, C);
    }
  }
  function mr(e, n, a) {
    try {
      var o = n.updateQueue, d = o !== null ? o.lastEffect : null;
      if (d !== null) {
        var p = d.next;
        o = p;
        do {
          if ((o.tag & e) === e) {
            var x = o.inst, C = x.destroy;
            if (C !== void 0) {
              x.destroy = void 0, d = n;
              var V = a, te = C;
              try {
                te();
              } catch (oe) {
                Be(
                  d,
                  V,
                  oe
                );
              }
            }
          }
          o = o.next;
        } while (o !== p);
      }
    } catch (oe) {
      Be(n, n.return, oe);
    }
  }
  function T1(e) {
    var n = e.updateQueue;
    if (n !== null) {
      var a = e.stateNode;
      try {
        p0(n, a);
      } catch (o) {
        Be(e, e.return, o);
      }
    }
  }
  function M1(e, n, a) {
    a.props = Wr(
      e.type,
      e.memoizedProps
    ), a.state = e.memoizedState;
    try {
      a.componentWillUnmount();
    } catch (o) {
      Be(e, n, o);
    }
  }
  function tu(e, n) {
    try {
      var a = e.ref;
      if (a !== null) {
        switch (e.tag) {
          case 26:
          case 27:
          case 5:
            var o = e.stateNode;
            break;
          case 30:
            o = e.stateNode;
            break;
          default:
            o = e.stateNode;
        }
        typeof a == "function" ? e.refCleanup = a(o) : a.current = o;
      }
    } catch (d) {
      Be(e, n, d);
    }
  }
  function Sn(e, n) {
    var a = e.ref, o = e.refCleanup;
    if (a !== null)
      if (typeof o == "function")
        try {
          o();
        } catch (d) {
          Be(e, n, d);
        } finally {
          e.refCleanup = null, e = e.alternate, e != null && (e.refCleanup = null);
        }
      else if (typeof a == "function")
        try {
          a(null);
        } catch (d) {
          Be(e, n, d);
        }
      else a.current = null;
  }
  function C1(e) {
    var n = e.type, a = e.memoizedProps, o = e.stateNode;
    try {
      e: switch (n) {
        case "button":
        case "input":
        case "select":
        case "textarea":
          a.autoFocus && o.focus();
          break e;
        case "img":
          a.src ? o.src = a.src : a.srcSet && (o.srcset = a.srcSet);
      }
    } catch (d) {
      Be(e, e.return, d);
    }
  }
  function Qc(e, n, a) {
    try {
      var o = e.stateNode;
      oR(o, e.type, a, n), o[wt] = n;
    } catch (d) {
      Be(e, e.return, d);
    }
  }
  function q1(e) {
    return e.tag === 5 || e.tag === 3 || e.tag === 26 || e.tag === 27 && Ar(e.type) || e.tag === 4;
  }
  function $c(e) {
    e: for (; ; ) {
      for (; e.sibling === null; ) {
        if (e.return === null || q1(e.return)) return null;
        e = e.return;
      }
      for (e.sibling.return = e.return, e = e.sibling; e.tag !== 5 && e.tag !== 6 && e.tag !== 18; ) {
        if (e.tag === 27 && Ar(e.type) || e.flags & 2 || e.child === null || e.tag === 4) continue e;
        e.child.return = e, e = e.child;
      }
      if (!(e.flags & 2)) return e.stateNode;
    }
  }
  function Fc(e, n, a) {
    var o = e.tag;
    if (o === 5 || o === 6)
      e = e.stateNode, n ? (a.nodeType === 9 ? a.body : a.nodeName === "HTML" ? a.ownerDocument.body : a).insertBefore(e, n) : (n = a.nodeType === 9 ? a.body : a.nodeName === "HTML" ? a.ownerDocument.body : a, n.appendChild(e), a = a._reactRootContainer, a != null || n.onclick !== null || (n.onclick = On));
    else if (o !== 4 && (o === 27 && Ar(e.type) && (a = e.stateNode, n = null), e = e.child, e !== null))
      for (Fc(e, n, a), e = e.sibling; e !== null; )
        Fc(e, n, a), e = e.sibling;
  }
  function Xl(e, n, a) {
    var o = e.tag;
    if (o === 5 || o === 6)
      e = e.stateNode, n ? a.insertBefore(e, n) : a.appendChild(e);
    else if (o !== 4 && (o === 27 && Ar(e.type) && (a = e.stateNode), e = e.child, e !== null))
      for (Xl(e, n, a), e = e.sibling; e !== null; )
        Xl(e, n, a), e = e.sibling;
  }
  function R1(e) {
    var n = e.stateNode, a = e.memoizedProps;
    try {
      for (var o = e.type, d = n.attributes; d.length; )
        n.removeAttributeNode(d[0]);
      _t(n, o, a), n[dt] = e, n[wt] = a;
    } catch (p) {
      Be(e, e.return, p);
    }
  }
  var kn = !1, ot = !1, Jc = !1, N1 = typeof WeakSet == "function" ? WeakSet : Set, pt = null;
  function Vq(e, n) {
    if (e = e.containerInfo, bf = fo, e = km(e), Xs(e)) {
      if ("selectionStart" in e)
        var a = {
          start: e.selectionStart,
          end: e.selectionEnd
        };
      else
        e: {
          a = (a = e.ownerDocument) && a.defaultView || window;
          var o = a.getSelection && a.getSelection();
          if (o && o.rangeCount !== 0) {
            a = o.anchorNode;
            var d = o.anchorOffset, p = o.focusNode;
            o = o.focusOffset;
            try {
              a.nodeType, p.nodeType;
            } catch {
              a = null;
              break e;
            }
            var x = 0, C = -1, V = -1, te = 0, oe = 0, fe = e, ne = null;
            t: for (; ; ) {
              for (var le; fe !== a || d !== 0 && fe.nodeType !== 3 || (C = x + d), fe !== p || o !== 0 && fe.nodeType !== 3 || (V = x + o), fe.nodeType === 3 && (x += fe.nodeValue.length), (le = fe.firstChild) !== null; )
                ne = fe, fe = le;
              for (; ; ) {
                if (fe === e) break t;
                if (ne === a && ++te === d && (C = x), ne === p && ++oe === o && (V = x), (le = fe.nextSibling) !== null) break;
                fe = ne, ne = fe.parentNode;
              }
              fe = le;
            }
            a = C === -1 || V === -1 ? null : { start: C, end: V };
          } else a = null;
        }
      a = a || { start: 0, end: 0 };
    } else a = null;
    for (_f = { focusedElem: e, selectionRange: a }, fo = !1, pt = n; pt !== null; )
      if (n = pt, e = n.child, (n.subtreeFlags & 1028) !== 0 && e !== null)
        e.return = n, pt = e;
      else
        for (; pt !== null; ) {
          switch (n = pt, p = n.alternate, e = n.flags, n.tag) {
            case 0:
              if ((e & 4) !== 0 && (e = n.updateQueue, e = e !== null ? e.events : null, e !== null))
                for (a = 0; a < e.length; a++)
                  d = e[a], d.ref.impl = d.nextImpl;
              break;
            case 11:
            case 15:
              break;
            case 1:
              if ((e & 1024) !== 0 && p !== null) {
                e = void 0, a = n, d = p.memoizedProps, p = p.memoizedState, o = a.stateNode;
                try {
                  var pe = Wr(
                    a.type,
                    d
                  );
                  e = o.getSnapshotBeforeUpdate(
                    pe,
                    p
                  ), o.__reactInternalSnapshotBeforeUpdate = e;
                } catch (Ee) {
                  Be(
                    a,
                    a.return,
                    Ee
                  );
                }
              }
              break;
            case 3:
              if ((e & 1024) !== 0) {
                if (e = n.stateNode.containerInfo, a = e.nodeType, a === 9)
                  Ef(e);
                else if (a === 1)
                  switch (e.nodeName) {
                    case "HEAD":
                    case "HTML":
                    case "BODY":
                      Ef(e);
                      break;
                    default:
                      e.textContent = "";
                  }
              }
              break;
            case 5:
            case 26:
            case 27:
            case 6:
            case 4:
            case 17:
              break;
            default:
              if ((e & 1024) !== 0) throw Error(u(163));
          }
          if (e = n.sibling, e !== null) {
            e.return = n.return, pt = e;
            break;
          }
          pt = n.return;
        }
  }
  function z1(e, n, a) {
    var o = a.flags;
    switch (a.tag) {
      case 0:
      case 11:
      case 15:
        In(e, a), o & 4 && eu(5, a);
        break;
      case 1:
        if (In(e, a), o & 4)
          if (e = a.stateNode, n === null)
            try {
              e.componentDidMount();
            } catch (x) {
              Be(a, a.return, x);
            }
          else {
            var d = Wr(
              a.type,
              n.memoizedProps
            );
            n = n.memoizedState;
            try {
              e.componentDidUpdate(
                d,
                n,
                e.__reactInternalSnapshotBeforeUpdate
              );
            } catch (x) {
              Be(
                a,
                a.return,
                x
              );
            }
          }
        o & 64 && T1(a), o & 512 && tu(a, a.return);
        break;
      case 3:
        if (In(e, a), o & 64 && (e = a.updateQueue, e !== null)) {
          if (n = null, a.child !== null)
            switch (a.child.tag) {
              case 27:
              case 5:
                n = a.child.stateNode;
                break;
              case 1:
                n = a.child.stateNode;
            }
          try {
            p0(e, n);
          } catch (x) {
            Be(a, a.return, x);
          }
        }
        break;
      case 27:
        n === null && o & 4 && R1(a);
      case 26:
      case 5:
        In(e, a), n === null && o & 4 && C1(a), o & 512 && tu(a, a.return);
        break;
      case 12:
        In(e, a);
        break;
      case 31:
        In(e, a), o & 4 && L1(e, a);
        break;
      case 13:
        In(e, a), o & 4 && H1(e, a), o & 64 && (e = a.memoizedState, e !== null && (e = e.dehydrated, e !== null && (a = Fq.bind(
          null,
          a
        ), pR(e, a))));
        break;
      case 22:
        if (o = a.memoizedState !== null || kn, !o) {
          n = n !== null && n.memoizedState !== null || ot, d = kn;
          var p = ot;
          kn = o, (ot = n) && !p ? Zn(
            e,
            a,
            (a.subtreeFlags & 8772) !== 0
          ) : In(e, a), kn = d, ot = p;
        }
        break;
      case 30:
        break;
      default:
        In(e, a);
    }
  }
  function O1(e) {
    var n = e.alternate;
    n !== null && (e.alternate = null, O1(n)), e.child = null, e.deletions = null, e.sibling = null, e.tag === 5 && (n = e.stateNode, n !== null && wi(n)), e.stateNode = null, e.return = null, e.dependencies = null, e.memoizedProps = null, e.memoizedState = null, e.pendingProps = null, e.stateNode = null, e.updateQueue = null;
  }
  var Qe = null, Ot = !1;
  function Xn(e, n, a) {
    for (a = a.child; a !== null; )
      D1(e, n, a), a = a.sibling;
  }
  function D1(e, n, a) {
    if (Je && typeof Je.onCommitFiberUnmount == "function")
      try {
        Je.onCommitFiberUnmount(Et, a);
      } catch {
      }
    switch (a.tag) {
      case 26:
        ot || Sn(a, n), Xn(
          e,
          n,
          a
        ), a.memoizedState ? a.memoizedState.count-- : a.stateNode && (a = a.stateNode, a.parentNode.removeChild(a));
        break;
      case 27:
        ot || Sn(a, n);
        var o = Qe, d = Ot;
        Ar(a.type) && (Qe = a.stateNode, Ot = !1), Xn(
          e,
          n,
          a
        ), cu(a.stateNode), Qe = o, Ot = d;
        break;
      case 5:
        ot || Sn(a, n);
      case 6:
        if (o = Qe, d = Ot, Qe = null, Xn(
          e,
          n,
          a
        ), Qe = o, Ot = d, Qe !== null)
          if (Ot)
            try {
              (Qe.nodeType === 9 ? Qe.body : Qe.nodeName === "HTML" ? Qe.ownerDocument.body : Qe).removeChild(a.stateNode);
            } catch (p) {
              Be(
                a,
                n,
                p
              );
            }
          else
            try {
              Qe.removeChild(a.stateNode);
            } catch (p) {
              Be(
                a,
                n,
                p
              );
            }
        break;
      case 18:
        Qe !== null && (Ot ? (e = Qe, Tb(
          e.nodeType === 9 ? e.body : e.nodeName === "HTML" ? e.ownerDocument.body : e,
          a.stateNode
        ), ni(e)) : Tb(Qe, a.stateNode));
        break;
      case 4:
        o = Qe, d = Ot, Qe = a.stateNode.containerInfo, Ot = !0, Xn(
          e,
          n,
          a
        ), Qe = o, Ot = d;
        break;
      case 0:
      case 11:
      case 14:
      case 15:
        mr(2, a, n), ot || mr(4, a, n), Xn(
          e,
          n,
          a
        );
        break;
      case 1:
        ot || (Sn(a, n), o = a.stateNode, typeof o.componentWillUnmount == "function" && M1(
          a,
          n,
          o
        )), Xn(
          e,
          n,
          a
        );
        break;
      case 21:
        Xn(
          e,
          n,
          a
        );
        break;
      case 22:
        ot = (o = ot) || a.memoizedState !== null, Xn(
          e,
          n,
          a
        ), ot = o;
        break;
      default:
        Xn(
          e,
          n,
          a
        );
    }
  }
  function L1(e, n) {
    if (n.memoizedState === null && (e = n.alternate, e !== null && (e = e.memoizedState, e !== null))) {
      e = e.dehydrated;
      try {
        ni(e);
      } catch (a) {
        Be(n, n.return, a);
      }
    }
  }
  function H1(e, n) {
    if (n.memoizedState === null && (e = n.alternate, e !== null && (e = e.memoizedState, e !== null && (e = e.dehydrated, e !== null))))
      try {
        ni(e);
      } catch (a) {
        Be(n, n.return, a);
      }
  }
  function Yq(e) {
    switch (e.tag) {
      case 31:
      case 13:
      case 19:
        var n = e.stateNode;
        return n === null && (n = e.stateNode = new N1()), n;
      case 22:
        return e = e.stateNode, n = e._retryCache, n === null && (n = e._retryCache = new N1()), n;
      default:
        throw Error(u(435, e.tag));
    }
  }
  function Il(e, n) {
    var a = Yq(e);
    n.forEach(function(o) {
      if (!a.has(o)) {
        a.add(o);
        var d = Jq.bind(null, e, o);
        o.then(d, d);
      }
    });
  }
  function Dt(e, n) {
    var a = n.deletions;
    if (a !== null)
      for (var o = 0; o < a.length; o++) {
        var d = a[o], p = e, x = n, C = x;
        e: for (; C !== null; ) {
          switch (C.tag) {
            case 27:
              if (Ar(C.type)) {
                Qe = C.stateNode, Ot = !1;
                break e;
              }
              break;
            case 5:
              Qe = C.stateNode, Ot = !1;
              break e;
            case 3:
            case 4:
              Qe = C.stateNode.containerInfo, Ot = !0;
              break e;
          }
          C = C.return;
        }
        if (Qe === null) throw Error(u(160));
        D1(p, x, d), Qe = null, Ot = !1, p = d.alternate, p !== null && (p.return = null), d.return = null;
      }
    if (n.subtreeFlags & 13886)
      for (n = n.child; n !== null; )
        j1(n, e), n = n.sibling;
  }
  var on = null;
  function j1(e, n) {
    var a = e.alternate, o = e.flags;
    switch (e.tag) {
      case 0:
      case 11:
      case 14:
      case 15:
        Dt(n, e), Lt(e), o & 4 && (mr(3, e, e.return), eu(3, e), mr(5, e, e.return));
        break;
      case 1:
        Dt(n, e), Lt(e), o & 512 && (ot || a === null || Sn(a, a.return)), o & 64 && kn && (e = e.updateQueue, e !== null && (o = e.callbacks, o !== null && (a = e.shared.hiddenCallbacks, e.shared.hiddenCallbacks = a === null ? o : a.concat(o))));
        break;
      case 26:
        var d = on;
        if (Dt(n, e), Lt(e), o & 512 && (ot || a === null || Sn(a, a.return)), o & 4) {
          var p = a !== null ? a.memoizedState : null;
          if (o = e.memoizedState, a === null)
            if (o === null)
              if (e.stateNode === null) {
                e: {
                  o = e.type, a = e.memoizedProps, d = d.ownerDocument || d;
                  t: switch (o) {
                    case "title":
                      p = d.getElementsByTagName("title")[0], (!p || p[Br] || p[dt] || p.namespaceURI === "http://www.w3.org/2000/svg" || p.hasAttribute("itemprop")) && (p = d.createElement(o), d.head.insertBefore(
                        p,
                        d.querySelector("head > title")
                      )), _t(p, o, a), p[dt] = e, at(p), o = p;
                      break e;
                    case "link":
                      var x = jb(
                        "link",
                        "href",
                        d
                      ).get(o + (a.href || ""));
                      if (x) {
                        for (var C = 0; C < x.length; C++)
                          if (p = x[C], p.getAttribute("href") === (a.href == null || a.href === "" ? null : a.href) && p.getAttribute("rel") === (a.rel == null ? null : a.rel) && p.getAttribute("title") === (a.title == null ? null : a.title) && p.getAttribute("crossorigin") === (a.crossOrigin == null ? null : a.crossOrigin)) {
                            x.splice(C, 1);
                            break t;
                          }
                      }
                      p = d.createElement(o), _t(p, o, a), d.head.appendChild(p);
                      break;
                    case "meta":
                      if (x = jb(
                        "meta",
                        "content",
                        d
                      ).get(o + (a.content || ""))) {
                        for (C = 0; C < x.length; C++)
                          if (p = x[C], p.getAttribute("content") === (a.content == null ? null : "" + a.content) && p.getAttribute("name") === (a.name == null ? null : a.name) && p.getAttribute("property") === (a.property == null ? null : a.property) && p.getAttribute("http-equiv") === (a.httpEquiv == null ? null : a.httpEquiv) && p.getAttribute("charset") === (a.charSet == null ? null : a.charSet)) {
                            x.splice(C, 1);
                            break t;
                          }
                      }
                      p = d.createElement(o), _t(p, o, a), d.head.appendChild(p);
                      break;
                    default:
                      throw Error(u(468, o));
                  }
                  p[dt] = e, at(p), o = p;
                }
                e.stateNode = o;
              } else
                Bb(
                  d,
                  e.type,
                  e.stateNode
                );
            else
              e.stateNode = Hb(
                d,
                o,
                e.memoizedProps
              );
          else
            p !== o ? (p === null ? a.stateNode !== null && (a = a.stateNode, a.parentNode.removeChild(a)) : p.count--, o === null ? Bb(
              d,
              e.type,
              e.stateNode
            ) : Hb(
              d,
              o,
              e.memoizedProps
            )) : o === null && e.stateNode !== null && Qc(
              e,
              e.memoizedProps,
              a.memoizedProps
            );
        }
        break;
      case 27:
        Dt(n, e), Lt(e), o & 512 && (ot || a === null || Sn(a, a.return)), a !== null && o & 4 && Qc(
          e,
          e.memoizedProps,
          a.memoizedProps
        );
        break;
      case 5:
        if (Dt(n, e), Lt(e), o & 512 && (ot || a === null || Sn(a, a.return)), e.flags & 32) {
          d = e.stateNode;
          try {
            Aa(d, "");
          } catch (pe) {
            Be(e, e.return, pe);
          }
        }
        o & 4 && e.stateNode != null && (d = e.memoizedProps, Qc(
          e,
          d,
          a !== null ? a.memoizedProps : d
        )), o & 1024 && (Jc = !0);
        break;
      case 6:
        if (Dt(n, e), Lt(e), o & 4) {
          if (e.stateNode === null)
            throw Error(u(162));
          o = e.memoizedProps, a = e.stateNode;
          try {
            a.nodeValue = o;
          } catch (pe) {
            Be(e, e.return, pe);
          }
        }
        break;
      case 3:
        if (lo = null, d = on, on = io(n.containerInfo), Dt(n, e), on = d, Lt(e), o & 4 && a !== null && a.memoizedState.isDehydrated)
          try {
            ni(n.containerInfo);
          } catch (pe) {
            Be(e, e.return, pe);
          }
        Jc && (Jc = !1, B1(e));
        break;
      case 4:
        o = on, on = io(
          e.stateNode.containerInfo
        ), Dt(n, e), Lt(e), on = o;
        break;
      case 12:
        Dt(n, e), Lt(e);
        break;
      case 31:
        Dt(n, e), Lt(e), o & 4 && (o = e.updateQueue, o !== null && (e.updateQueue = null, Il(e, o)));
        break;
      case 13:
        Dt(n, e), Lt(e), e.child.flags & 8192 && e.memoizedState !== null != (a !== null && a.memoizedState !== null) && (Kl = ke()), o & 4 && (o = e.updateQueue, o !== null && (e.updateQueue = null, Il(e, o)));
        break;
      case 22:
        d = e.memoizedState !== null;
        var V = a !== null && a.memoizedState !== null, te = kn, oe = ot;
        if (kn = te || d, ot = oe || V, Dt(n, e), ot = oe, kn = te, Lt(e), o & 8192)
          e: for (n = e.stateNode, n._visibility = d ? n._visibility & -2 : n._visibility | 1, d && (a === null || V || kn || ot || ea(e)), a = null, n = e; ; ) {
            if (n.tag === 5 || n.tag === 26) {
              if (a === null) {
                V = a = n;
                try {
                  if (p = V.stateNode, d)
                    x = p.style, typeof x.setProperty == "function" ? x.setProperty("display", "none", "important") : x.display = "none";
                  else {
                    C = V.stateNode;
                    var fe = V.memoizedProps.style, ne = fe != null && fe.hasOwnProperty("display") ? fe.display : null;
                    C.style.display = ne == null || typeof ne == "boolean" ? "" : ("" + ne).trim();
                  }
                } catch (pe) {
                  Be(V, V.return, pe);
                }
              }
            } else if (n.tag === 6) {
              if (a === null) {
                V = n;
                try {
                  V.stateNode.nodeValue = d ? "" : V.memoizedProps;
                } catch (pe) {
                  Be(V, V.return, pe);
                }
              }
            } else if (n.tag === 18) {
              if (a === null) {
                V = n;
                try {
                  var le = V.stateNode;
                  d ? Mb(le, !0) : Mb(V.stateNode, !1);
                } catch (pe) {
                  Be(V, V.return, pe);
                }
              }
            } else if ((n.tag !== 22 && n.tag !== 23 || n.memoizedState === null || n === e) && n.child !== null) {
              n.child.return = n, n = n.child;
              continue;
            }
            if (n === e) break e;
            for (; n.sibling === null; ) {
              if (n.return === null || n.return === e) break e;
              a === n && (a = null), n = n.return;
            }
            a === n && (a = null), n.sibling.return = n.return, n = n.sibling;
          }
        o & 4 && (o = e.updateQueue, o !== null && (a = o.retryQueue, a !== null && (o.retryQueue = null, Il(e, a))));
        break;
      case 19:
        Dt(n, e), Lt(e), o & 4 && (o = e.updateQueue, o !== null && (e.updateQueue = null, Il(e, o)));
        break;
      case 30:
        break;
      case 21:
        break;
      default:
        Dt(n, e), Lt(e);
    }
  }
  function Lt(e) {
    var n = e.flags;
    if (n & 2) {
      try {
        for (var a, o = e.return; o !== null; ) {
          if (q1(o)) {
            a = o;
            break;
          }
          o = o.return;
        }
        if (a == null) throw Error(u(160));
        switch (a.tag) {
          case 27:
            var d = a.stateNode, p = $c(e);
            Xl(e, p, d);
            break;
          case 5:
            var x = a.stateNode;
            a.flags & 32 && (Aa(x, ""), a.flags &= -33);
            var C = $c(e);
            Xl(e, C, x);
            break;
          case 3:
          case 4:
            var V = a.stateNode.containerInfo, te = $c(e);
            Fc(
              e,
              te,
              V
            );
            break;
          default:
            throw Error(u(161));
        }
      } catch (oe) {
        Be(e, e.return, oe);
      }
      e.flags &= -3;
    }
    n & 4096 && (e.flags &= -4097);
  }
  function B1(e) {
    if (e.subtreeFlags & 1024)
      for (e = e.child; e !== null; ) {
        var n = e;
        B1(n), n.tag === 5 && n.flags & 1024 && n.stateNode.reset(), e = e.sibling;
      }
  }
  function In(e, n) {
    if (n.subtreeFlags & 8772)
      for (n = n.child; n !== null; )
        z1(e, n.alternate, n), n = n.sibling;
  }
  function ea(e) {
    for (e = e.child; e !== null; ) {
      var n = e;
      switch (n.tag) {
        case 0:
        case 11:
        case 14:
        case 15:
          mr(4, n, n.return), ea(n);
          break;
        case 1:
          Sn(n, n.return);
          var a = n.stateNode;
          typeof a.componentWillUnmount == "function" && M1(
            n,
            n.return,
            a
          ), ea(n);
          break;
        case 27:
          cu(n.stateNode);
        case 26:
        case 5:
          Sn(n, n.return), ea(n);
          break;
        case 22:
          n.memoizedState === null && ea(n);
          break;
        case 30:
          ea(n);
          break;
        default:
          ea(n);
      }
      e = e.sibling;
    }
  }
  function Zn(e, n, a) {
    for (a = a && (n.subtreeFlags & 8772) !== 0, n = n.child; n !== null; ) {
      var o = n.alternate, d = e, p = n, x = p.flags;
      switch (p.tag) {
        case 0:
        case 11:
        case 15:
          Zn(
            d,
            p,
            a
          ), eu(4, p);
          break;
        case 1:
          if (Zn(
            d,
            p,
            a
          ), o = p, d = o.stateNode, typeof d.componentDidMount == "function")
            try {
              d.componentDidMount();
            } catch (te) {
              Be(o, o.return, te);
            }
          if (o = p, d = o.updateQueue, d !== null) {
            var C = o.stateNode;
            try {
              var V = d.shared.hiddenCallbacks;
              if (V !== null)
                for (d.shared.hiddenCallbacks = null, d = 0; d < V.length; d++)
                  v0(V[d], C);
            } catch (te) {
              Be(o, o.return, te);
            }
          }
          a && x & 64 && T1(p), tu(p, p.return);
          break;
        case 27:
          R1(p);
        case 26:
        case 5:
          Zn(
            d,
            p,
            a
          ), a && o === null && x & 4 && C1(p), tu(p, p.return);
          break;
        case 12:
          Zn(
            d,
            p,
            a
          );
          break;
        case 31:
          Zn(
            d,
            p,
            a
          ), a && x & 4 && L1(d, p);
          break;
        case 13:
          Zn(
            d,
            p,
            a
          ), a && x & 4 && H1(d, p);
          break;
        case 22:
          p.memoizedState === null && Zn(
            d,
            p,
            a
          ), tu(p, p.return);
          break;
        case 30:
          break;
        default:
          Zn(
            d,
            p,
            a
          );
      }
      n = n.sibling;
    }
  }
  function Pc(e, n) {
    var a = null;
    e !== null && e.memoizedState !== null && e.memoizedState.cachePool !== null && (a = e.memoizedState.cachePool.pool), e = null, n.memoizedState !== null && n.memoizedState.cachePool !== null && (e = n.memoizedState.cachePool.pool), e !== a && (e != null && e.refCount++, a != null && Vi(a));
  }
  function Wc(e, n) {
    e = null, n.alternate !== null && (e = n.alternate.memoizedState.cache), n = n.memoizedState.cache, n !== e && (n.refCount++, e != null && Vi(e));
  }
  function sn(e, n, a, o) {
    if (n.subtreeFlags & 10256)
      for (n = n.child; n !== null; )
        U1(
          e,
          n,
          a,
          o
        ), n = n.sibling;
  }
  function U1(e, n, a, o) {
    var d = n.flags;
    switch (n.tag) {
      case 0:
      case 11:
      case 15:
        sn(
          e,
          n,
          a,
          o
        ), d & 2048 && eu(9, n);
        break;
      case 1:
        sn(
          e,
          n,
          a,
          o
        );
        break;
      case 3:
        sn(
          e,
          n,
          a,
          o
        ), d & 2048 && (e = null, n.alternate !== null && (e = n.alternate.memoizedState.cache), n = n.memoizedState.cache, n !== e && (n.refCount++, e != null && Vi(e)));
        break;
      case 12:
        if (d & 2048) {
          sn(
            e,
            n,
            a,
            o
          ), e = n.stateNode;
          try {
            var p = n.memoizedProps, x = p.id, C = p.onPostCommit;
            typeof C == "function" && C(
              x,
              n.alternate === null ? "mount" : "update",
              e.passiveEffectDuration,
              -0
            );
          } catch (V) {
            Be(n, n.return, V);
          }
        } else
          sn(
            e,
            n,
            a,
            o
          );
        break;
      case 31:
        sn(
          e,
          n,
          a,
          o
        );
        break;
      case 13:
        sn(
          e,
          n,
          a,
          o
        );
        break;
      case 23:
        break;
      case 22:
        p = n.stateNode, x = n.alternate, n.memoizedState !== null ? p._visibility & 2 ? sn(
          e,
          n,
          a,
          o
        ) : nu(e, n) : p._visibility & 2 ? sn(
          e,
          n,
          a,
          o
        ) : (p._visibility |= 2, Ia(
          e,
          n,
          a,
          o,
          (n.subtreeFlags & 10256) !== 0 || !1
        )), d & 2048 && Pc(x, n);
        break;
      case 24:
        sn(
          e,
          n,
          a,
          o
        ), d & 2048 && Wc(n.alternate, n);
        break;
      default:
        sn(
          e,
          n,
          a,
          o
        );
    }
  }
  function Ia(e, n, a, o, d) {
    for (d = d && ((n.subtreeFlags & 10256) !== 0 || !1), n = n.child; n !== null; ) {
      var p = e, x = n, C = a, V = o, te = x.flags;
      switch (x.tag) {
        case 0:
        case 11:
        case 15:
          Ia(
            p,
            x,
            C,
            V,
            d
          ), eu(8, x);
          break;
        case 23:
          break;
        case 22:
          var oe = x.stateNode;
          x.memoizedState !== null ? oe._visibility & 2 ? Ia(
            p,
            x,
            C,
            V,
            d
          ) : nu(
            p,
            x
          ) : (oe._visibility |= 2, Ia(
            p,
            x,
            C,
            V,
            d
          )), d && te & 2048 && Pc(
            x.alternate,
            x
          );
          break;
        case 24:
          Ia(
            p,
            x,
            C,
            V,
            d
          ), d && te & 2048 && Wc(x.alternate, x);
          break;
        default:
          Ia(
            p,
            x,
            C,
            V,
            d
          );
      }
      n = n.sibling;
    }
  }
  function nu(e, n) {
    if (n.subtreeFlags & 10256)
      for (n = n.child; n !== null; ) {
        var a = e, o = n, d = o.flags;
        switch (o.tag) {
          case 22:
            nu(a, o), d & 2048 && Pc(
              o.alternate,
              o
            );
            break;
          case 24:
            nu(a, o), d & 2048 && Wc(o.alternate, o);
            break;
          default:
            nu(a, o);
        }
        n = n.sibling;
      }
  }
  var ru = 8192;
  function Za(e, n, a) {
    if (e.subtreeFlags & ru)
      for (e = e.child; e !== null; )
        G1(
          e,
          n,
          a
        ), e = e.sibling;
  }
  function G1(e, n, a) {
    switch (e.tag) {
      case 26:
        Za(
          e,
          n,
          a
        ), e.flags & ru && e.memoizedState !== null && CR(
          a,
          on,
          e.memoizedState,
          e.memoizedProps
        );
        break;
      case 5:
        Za(
          e,
          n,
          a
        );
        break;
      case 3:
      case 4:
        var o = on;
        on = io(e.stateNode.containerInfo), Za(
          e,
          n,
          a
        ), on = o;
        break;
      case 22:
        e.memoizedState === null && (o = e.alternate, o !== null && o.memoizedState !== null ? (o = ru, ru = 16777216, Za(
          e,
          n,
          a
        ), ru = o) : Za(
          e,
          n,
          a
        ));
        break;
      default:
        Za(
          e,
          n,
          a
        );
    }
  }
  function V1(e) {
    var n = e.alternate;
    if (n !== null && (e = n.child, e !== null)) {
      n.child = null;
      do
        n = e.sibling, e.sibling = null, e = n;
      while (e !== null);
    }
  }
  function au(e) {
    var n = e.deletions;
    if ((e.flags & 16) !== 0) {
      if (n !== null)
        for (var a = 0; a < n.length; a++) {
          var o = n[a];
          pt = o, k1(
            o,
            e
          );
        }
      V1(e);
    }
    if (e.subtreeFlags & 10256)
      for (e = e.child; e !== null; )
        Y1(e), e = e.sibling;
  }
  function Y1(e) {
    switch (e.tag) {
      case 0:
      case 11:
      case 15:
        au(e), e.flags & 2048 && mr(9, e, e.return);
        break;
      case 3:
        au(e);
        break;
      case 12:
        au(e);
        break;
      case 22:
        var n = e.stateNode;
        e.memoizedState !== null && n._visibility & 2 && (e.return === null || e.return.tag !== 13) ? (n._visibility &= -3, Zl(e)) : au(e);
        break;
      default:
        au(e);
    }
  }
  function Zl(e) {
    var n = e.deletions;
    if ((e.flags & 16) !== 0) {
      if (n !== null)
        for (var a = 0; a < n.length; a++) {
          var o = n[a];
          pt = o, k1(
            o,
            e
          );
        }
      V1(e);
    }
    for (e = e.child; e !== null; ) {
      switch (n = e, n.tag) {
        case 0:
        case 11:
        case 15:
          mr(8, n, n.return), Zl(n);
          break;
        case 22:
          a = n.stateNode, a._visibility & 2 && (a._visibility &= -3, Zl(n));
          break;
        default:
          Zl(n);
      }
      e = e.sibling;
    }
  }
  function k1(e, n) {
    for (; pt !== null; ) {
      var a = pt;
      switch (a.tag) {
        case 0:
        case 11:
        case 15:
          mr(8, a, n);
          break;
        case 23:
        case 22:
          if (a.memoizedState !== null && a.memoizedState.cachePool !== null) {
            var o = a.memoizedState.cachePool.pool;
            o != null && o.refCount++;
          }
          break;
        case 24:
          Vi(a.memoizedState.cache);
      }
      if (o = a.child, o !== null) o.return = a, pt = o;
      else
        e: for (a = e; pt !== null; ) {
          o = pt;
          var d = o.sibling, p = o.return;
          if (O1(o), o === a) {
            pt = null;
            break e;
          }
          if (d !== null) {
            d.return = p, pt = d;
            break e;
          }
          pt = p;
        }
    }
  }
  var kq = {
    getCacheForType: function(e) {
      var n = mt(it), a = n.data.get(e);
      return a === void 0 && (a = e(), n.data.set(e, a)), a;
    },
    cacheSignal: function() {
      return mt(it).controller.signal;
    }
  }, Xq = typeof WeakMap == "function" ? WeakMap : Map, He = 0, Ye = null, Ce = null, Re = 0, je = 0, Xt = null, br = !1, Ka = !1, ef = !1, Kn = 0, et = 0, _r = 0, ta = 0, tf = 0, It = 0, Qa = 0, iu = null, Ht = null, nf = !1, Kl = 0, X1 = 0, Ql = 1 / 0, $l = null, xr = null, ht = 0, Sr = null, $a = null, Qn = 0, rf = 0, af = null, I1 = null, uu = 0, uf = null;
  function Zt() {
    return (He & 2) !== 0 && Re !== 0 ? Re & -Re : z.T !== null ? df() : el();
  }
  function Z1() {
    if (It === 0)
      if ((Re & 536870912) === 0 || ze) {
        var e = ya;
        ya <<= 1, (ya & 3932160) === 0 && (ya = 262144), It = e;
      } else It = 536870912;
    return e = Yt.current, e !== null && (e.flags |= 32), It;
  }
  function jt(e, n, a) {
    (e === Ye && (je === 2 || je === 9) || e.cancelPendingCommit !== null) && (Fa(e, 0), Er(
      e,
      Re,
      It,
      !1
    )), jr(e, a), ((He & 2) === 0 || e !== Ye) && (e === Ye && ((He & 2) === 0 && (ta |= a), et === 4 && Er(
      e,
      Re,
      It,
      !1
    )), En(e));
  }
  function K1(e, n, a) {
    if ((He & 6) !== 0) throw Error(u(327));
    var o = !a && (n & 127) === 0 && (n & e.expiredLanes) === 0 || Hr(e, n), d = o ? Kq(e, n) : of(e, n, !0), p = o;
    do {
      if (d === 0) {
        Ka && !o && Er(e, n, 0, !1);
        break;
      } else {
        if (a = e.current.alternate, p && !Iq(a)) {
          d = of(e, n, !1), p = !1;
          continue;
        }
        if (d === 2) {
          if (p = n, e.errorRecoveryDisabledLanes & p)
            var x = 0;
          else
            x = e.pendingLanes & -536870913, x = x !== 0 ? x : x & 536870912 ? 536870912 : 0;
          if (x !== 0) {
            n = x;
            e: {
              var C = e;
              d = iu;
              var V = C.current.memoizedState.isDehydrated;
              if (V && (Fa(C, x).flags |= 256), x = of(
                C,
                x,
                !1
              ), x !== 2) {
                if (ef && !V) {
                  C.errorRecoveryDisabledLanes |= p, ta |= p, d = 4;
                  break e;
                }
                p = Ht, Ht = d, p !== null && (Ht === null ? Ht = p : Ht.push.apply(
                  Ht,
                  p
                ));
              }
              d = x;
            }
            if (p = !1, d !== 2) continue;
          }
        }
        if (d === 1) {
          Fa(e, 0), Er(e, n, 0, !0);
          break;
        }
        e: {
          switch (o = e, p = d, p) {
            case 0:
            case 1:
              throw Error(u(345));
            case 4:
              if ((n & 4194048) !== n) break;
            case 6:
              Er(
                o,
                n,
                It,
                !br
              );
              break e;
            case 2:
              Ht = null;
              break;
            case 3:
            case 5:
              break;
            default:
              throw Error(u(329));
          }
          if ((n & 62914560) === n && (d = Kl + 300 - ke(), 10 < d)) {
            if (Er(
              o,
              n,
              It,
              !br
            ), ba(o, 0, !0) !== 0) break e;
            Qn = n, o.timeoutHandle = wb(
              Q1.bind(
                null,
                o,
                a,
                Ht,
                $l,
                nf,
                n,
                It,
                ta,
                Qa,
                br,
                p,
                "Throttled",
                -0,
                0
              ),
              d
            );
            break e;
          }
          Q1(
            o,
            a,
            Ht,
            $l,
            nf,
            n,
            It,
            ta,
            Qa,
            br,
            p,
            null,
            -0,
            0
          );
        }
      }
      break;
    } while (!0);
    En(e);
  }
  function Q1(e, n, a, o, d, p, x, C, V, te, oe, fe, ne, le) {
    if (e.timeoutHandle = -1, fe = n.subtreeFlags, fe & 8192 || (fe & 16785408) === 16785408) {
      fe = {
        stylesheets: null,
        count: 0,
        imgCount: 0,
        imgBytes: 0,
        suspenseyImages: [],
        waitingForImages: !0,
        waitingForViewTransition: !1,
        unsuspend: On
      }, G1(
        n,
        p,
        fe
      );
      var pe = (p & 62914560) === p ? Kl - ke() : (p & 4194048) === p ? X1 - ke() : 0;
      if (pe = qR(
        fe,
        pe
      ), pe !== null) {
        Qn = p, e.cancelPendingCommit = pe(
          nb.bind(
            null,
            e,
            n,
            p,
            a,
            o,
            d,
            x,
            C,
            V,
            oe,
            fe,
            null,
            ne,
            le
          )
        ), Er(e, p, x, !te);
        return;
      }
    }
    nb(
      e,
      n,
      p,
      a,
      o,
      d,
      x,
      C,
      V
    );
  }
  function Iq(e) {
    for (var n = e; ; ) {
      var a = n.tag;
      if ((a === 0 || a === 11 || a === 15) && n.flags & 16384 && (a = n.updateQueue, a !== null && (a = a.stores, a !== null)))
        for (var o = 0; o < a.length; o++) {
          var d = a[o], p = d.getSnapshot;
          d = d.value;
          try {
            if (!Gt(p(), d)) return !1;
          } catch {
            return !1;
          }
        }
      if (a = n.child, n.subtreeFlags & 16384 && a !== null)
        a.return = n, n = a;
      else {
        if (n === e) break;
        for (; n.sibling === null; ) {
          if (n.return === null || n.return === e) return !0;
          n = n.return;
        }
        n.sibling.return = n.return, n = n.sibling;
      }
    }
    return !0;
  }
  function Er(e, n, a, o) {
    n &= ~tf, n &= ~ta, e.suspendedLanes |= n, e.pingedLanes &= ~n, o && (e.warmLanes |= n), o = e.expirationTimes;
    for (var d = n; 0 < d; ) {
      var p = 31 - Mt(d), x = 1 << p;
      o[p] = -1, d &= ~x;
    }
    a !== 0 && Ju(e, a, n);
  }
  function Fl() {
    return (He & 6) === 0 ? (lu(0), !1) : !0;
  }
  function lf() {
    if (Ce !== null) {
      if (je === 0)
        var e = Ce.return;
      else
        e = Ce, jn = Zr = null, Sc(e), Ga = null, ki = 0, e = Ce;
      for (; e !== null; )
        A1(e.alternate, e), e = e.return;
      Ce = null;
    }
  }
  function Fa(e, n) {
    var a = e.timeoutHandle;
    a !== -1 && (e.timeoutHandle = -1, fR(a)), a = e.cancelPendingCommit, a !== null && (e.cancelPendingCommit = null, a()), Qn = 0, lf(), Ye = e, Ce = a = Ln(e.current, null), Re = n, je = 0, Xt = null, br = !1, Ka = Hr(e, n), ef = !1, Qa = It = tf = ta = _r = et = 0, Ht = iu = null, nf = !1, (n & 8) !== 0 && (n |= n & 32);
    var o = e.entangledLanes;
    if (o !== 0)
      for (e = e.entanglements, o &= n; 0 < o; ) {
        var d = 31 - Mt(o), p = 1 << d;
        n |= e[d], o &= ~p;
      }
    return Kn = n, yl(), a;
  }
  function $1(e, n) {
    Te = null, z.H = Ji, n === Ua || n === Al ? (n = f0(), je = 3) : n === cc ? (n = f0(), je = 4) : je = n === Bc ? 8 : n !== null && typeof n == "object" && typeof n.then == "function" ? 6 : 1, Xt = n, Ce === null && (et = 1, Ul(
      e,
      Ft(n, e.current)
    ));
  }
  function F1() {
    var e = Yt.current;
    return e === null ? !0 : (Re & 4194048) === Re ? en === null : (Re & 62914560) === Re || (Re & 536870912) !== 0 ? e === en : !1;
  }
  function J1() {
    var e = z.H;
    return z.H = Ji, e === null ? Ji : e;
  }
  function P1() {
    var e = z.A;
    return z.A = kq, e;
  }
  function Jl() {
    et = 4, br || (Re & 4194048) !== Re && Yt.current !== null || (Ka = !0), (_r & 134217727) === 0 && (ta & 134217727) === 0 || Ye === null || Er(
      Ye,
      Re,
      It,
      !1
    );
  }
  function of(e, n, a) {
    var o = He;
    He |= 2;
    var d = J1(), p = P1();
    (Ye !== e || Re !== n) && ($l = null, Fa(e, n)), n = !1;
    var x = et;
    e: do
      try {
        if (je !== 0 && Ce !== null) {
          var C = Ce, V = Xt;
          switch (je) {
            case 8:
              lf(), x = 6;
              break e;
            case 3:
            case 2:
            case 9:
            case 6:
              Yt.current === null && (n = !0);
              var te = je;
              if (je = 0, Xt = null, Ja(e, C, V, te), a && Ka) {
                x = 0;
                break e;
              }
              break;
            default:
              te = je, je = 0, Xt = null, Ja(e, C, V, te);
          }
        }
        Zq(), x = et;
        break;
      } catch (oe) {
        $1(e, oe);
      }
    while (!0);
    return n && e.shellSuspendCounter++, jn = Zr = null, He = o, z.H = d, z.A = p, Ce === null && (Ye = null, Re = 0, yl()), x;
  }
  function Zq() {
    for (; Ce !== null; ) W1(Ce);
  }
  function Kq(e, n) {
    var a = He;
    He |= 2;
    var o = J1(), d = P1();
    Ye !== e || Re !== n ? ($l = null, Ql = ke() + 500, Fa(e, n)) : Ka = Hr(
      e,
      n
    );
    e: do
      try {
        if (je !== 0 && Ce !== null) {
          n = Ce;
          var p = Xt;
          t: switch (je) {
            case 1:
              je = 0, Xt = null, Ja(e, n, p, 1);
              break;
            case 2:
            case 9:
              if (s0(p)) {
                je = 0, Xt = null, eb(n);
                break;
              }
              n = function() {
                je !== 2 && je !== 9 || Ye !== e || (je = 7), En(e);
              }, p.then(n, n);
              break e;
            case 3:
              je = 7;
              break e;
            case 4:
              je = 5;
              break e;
            case 7:
              s0(p) ? (je = 0, Xt = null, eb(n)) : (je = 0, Xt = null, Ja(e, n, p, 7));
              break;
            case 5:
              var x = null;
              switch (Ce.tag) {
                case 26:
                  x = Ce.memoizedState;
                case 5:
                case 27:
                  var C = Ce;
                  if (x ? Ub(x) : C.stateNode.complete) {
                    je = 0, Xt = null;
                    var V = C.sibling;
                    if (V !== null) Ce = V;
                    else {
                      var te = C.return;
                      te !== null ? (Ce = te, Pl(te)) : Ce = null;
                    }
                    break t;
                  }
              }
              je = 0, Xt = null, Ja(e, n, p, 5);
              break;
            case 6:
              je = 0, Xt = null, Ja(e, n, p, 6);
              break;
            case 8:
              lf(), et = 6;
              break e;
            default:
              throw Error(u(462));
          }
        }
        Qq();
        break;
      } catch (oe) {
        $1(e, oe);
      }
    while (!0);
    return jn = Zr = null, z.H = o, z.A = d, He = a, Ce !== null ? 0 : (Ye = null, Re = 0, yl(), et);
  }
  function Qq() {
    for (; Ce !== null && !St(); )
      W1(Ce);
  }
  function W1(e) {
    var n = E1(e.alternate, e, Kn);
    e.memoizedProps = e.pendingProps, n === null ? Pl(e) : Ce = n;
  }
  function eb(e) {
    var n = e, a = n.alternate;
    switch (n.tag) {
      case 15:
      case 0:
        n = y1(
          a,
          n,
          n.pendingProps,
          n.type,
          void 0,
          Re
        );
        break;
      case 11:
        n = y1(
          a,
          n,
          n.pendingProps,
          n.type.render,
          n.ref,
          Re
        );
        break;
      case 5:
        Sc(n);
      default:
        A1(a, n), n = Ce = Pm(n, Kn), n = E1(a, n, Kn);
    }
    e.memoizedProps = e.pendingProps, n === null ? Pl(e) : Ce = n;
  }
  function Ja(e, n, a, o) {
    jn = Zr = null, Sc(n), Ga = null, ki = 0;
    var d = n.return;
    try {
      if (Hq(
        e,
        d,
        n,
        a,
        Re
      )) {
        et = 1, Ul(
          e,
          Ft(a, e.current)
        ), Ce = null;
        return;
      }
    } catch (p) {
      if (d !== null) throw Ce = d, p;
      et = 1, Ul(
        e,
        Ft(a, e.current)
      ), Ce = null;
      return;
    }
    n.flags & 32768 ? (ze || o === 1 ? e = !0 : Ka || (Re & 536870912) !== 0 ? e = !1 : (br = e = !0, (o === 2 || o === 9 || o === 3 || o === 6) && (o = Yt.current, o !== null && o.tag === 13 && (o.flags |= 16384))), tb(n, e)) : Pl(n);
  }
  function Pl(e) {
    var n = e;
    do {
      if ((n.flags & 32768) !== 0) {
        tb(
          n,
          br
        );
        return;
      }
      e = n.return;
      var a = Uq(
        n.alternate,
        n,
        Kn
      );
      if (a !== null) {
        Ce = a;
        return;
      }
      if (n = n.sibling, n !== null) {
        Ce = n;
        return;
      }
      Ce = n = e;
    } while (n !== null);
    et === 0 && (et = 5);
  }
  function tb(e, n) {
    do {
      var a = Gq(e.alternate, e);
      if (a !== null) {
        a.flags &= 32767, Ce = a;
        return;
      }
      if (a = e.return, a !== null && (a.flags |= 32768, a.subtreeFlags = 0, a.deletions = null), !n && (e = e.sibling, e !== null)) {
        Ce = e;
        return;
      }
      Ce = e = a;
    } while (e !== null);
    et = 6, Ce = null;
  }
  function nb(e, n, a, o, d, p, x, C, V) {
    e.cancelPendingCommit = null;
    do
      Wl();
    while (ht !== 0);
    if ((He & 6) !== 0) throw Error(u(327));
    if (n !== null) {
      if (n === e.current) throw Error(u(177));
      if (p = n.lanes | n.childLanes, p |= $s, Ts(
        e,
        a,
        p,
        x,
        C,
        V
      ), e === Ye && (Ce = Ye = null, Re = 0), $a = n, Sr = e, Qn = a, rf = p, af = d, I1 = o, (n.subtreeFlags & 10256) !== 0 || (n.flags & 10256) !== 0 ? (e.callbackNode = null, e.callbackPriority = 0, Pq(ft, function() {
        return lb(), null;
      })) : (e.callbackNode = null, e.callbackPriority = 0), o = (n.flags & 13878) !== 0, (n.subtreeFlags & 13878) !== 0 || o) {
        o = z.T, z.T = null, d = B.p, B.p = 2, x = He, He |= 4;
        try {
          Vq(e, n, a);
        } finally {
          He = x, B.p = d, z.T = o;
        }
      }
      ht = 1, rb(), ab(), ib();
    }
  }
  function rb() {
    if (ht === 1) {
      ht = 0;
      var e = Sr, n = $a, a = (n.flags & 13878) !== 0;
      if ((n.subtreeFlags & 13878) !== 0 || a) {
        a = z.T, z.T = null;
        var o = B.p;
        B.p = 2;
        var d = He;
        He |= 4;
        try {
          j1(n, e);
          var p = _f, x = km(e.containerInfo), C = p.focusedElem, V = p.selectionRange;
          if (x !== C && C && C.ownerDocument && Ym(
            C.ownerDocument.documentElement,
            C
          )) {
            if (V !== null && Xs(C)) {
              var te = V.start, oe = V.end;
              if (oe === void 0 && (oe = te), "selectionStart" in C)
                C.selectionStart = te, C.selectionEnd = Math.min(
                  oe,
                  C.value.length
                );
              else {
                var fe = C.ownerDocument || document, ne = fe && fe.defaultView || window;
                if (ne.getSelection) {
                  var le = ne.getSelection(), pe = C.textContent.length, Ee = Math.min(V.start, pe), Ve = V.end === void 0 ? Ee : Math.min(V.end, pe);
                  !le.extend && Ee > Ve && (x = Ve, Ve = Ee, Ee = x);
                  var F = Vm(
                    C,
                    Ee
                  ), I = Vm(
                    C,
                    Ve
                  );
                  if (F && I && (le.rangeCount !== 1 || le.anchorNode !== F.node || le.anchorOffset !== F.offset || le.focusNode !== I.node || le.focusOffset !== I.offset)) {
                    var ee = fe.createRange();
                    ee.setStart(F.node, F.offset), le.removeAllRanges(), Ee > Ve ? (le.addRange(ee), le.extend(I.node, I.offset)) : (ee.setEnd(I.node, I.offset), le.addRange(ee));
                  }
                }
              }
            }
            for (fe = [], le = C; le = le.parentNode; )
              le.nodeType === 1 && fe.push({
                element: le,
                left: le.scrollLeft,
                top: le.scrollTop
              });
            for (typeof C.focus == "function" && C.focus(), C = 0; C < fe.length; C++) {
              var ce = fe[C];
              ce.element.scrollLeft = ce.left, ce.element.scrollTop = ce.top;
            }
          }
          fo = !!bf, _f = bf = null;
        } finally {
          He = d, B.p = o, z.T = a;
        }
      }
      e.current = n, ht = 2;
    }
  }
  function ab() {
    if (ht === 2) {
      ht = 0;
      var e = Sr, n = $a, a = (n.flags & 8772) !== 0;
      if ((n.subtreeFlags & 8772) !== 0 || a) {
        a = z.T, z.T = null;
        var o = B.p;
        B.p = 2;
        var d = He;
        He |= 4;
        try {
          z1(e, n.alternate, n);
        } finally {
          He = d, B.p = o, z.T = a;
        }
      }
      ht = 3;
    }
  }
  function ib() {
    if (ht === 4 || ht === 3) {
      ht = 0, Ke();
      var e = Sr, n = $a, a = Qn, o = I1;
      (n.subtreeFlags & 10256) !== 0 || (n.flags & 10256) !== 0 ? ht = 5 : (ht = 0, $a = Sr = null, ub(e, e.pendingLanes));
      var d = e.pendingLanes;
      if (d === 0 && (xr = null), Ei(a), n = n.stateNode, Je && typeof Je.onCommitFiberRoot == "function")
        try {
          Je.onCommitFiberRoot(
            Et,
            n,
            void 0,
            (n.current.flags & 128) === 128
          );
        } catch {
        }
      if (o !== null) {
        n = z.T, d = B.p, B.p = 2, z.T = null;
        try {
          for (var p = e.onRecoverableError, x = 0; x < o.length; x++) {
            var C = o[x];
            p(C.value, {
              componentStack: C.stack
            });
          }
        } finally {
          z.T = n, B.p = d;
        }
      }
      (Qn & 3) !== 0 && Wl(), En(e), d = e.pendingLanes, (a & 261930) !== 0 && (d & 42) !== 0 ? e === uf ? uu++ : (uu = 0, uf = e) : uu = 0, lu(0);
    }
  }
  function ub(e, n) {
    (e.pooledCacheLanes &= n) === 0 && (n = e.pooledCache, n != null && (e.pooledCache = null, Vi(n)));
  }
  function Wl() {
    return rb(), ab(), ib(), lb();
  }
  function lb() {
    if (ht !== 5) return !1;
    var e = Sr, n = rf;
    rf = 0;
    var a = Ei(Qn), o = z.T, d = B.p;
    try {
      B.p = 32 > a ? 32 : a, z.T = null, a = af, af = null;
      var p = Sr, x = Qn;
      if (ht = 0, $a = Sr = null, Qn = 0, (He & 6) !== 0) throw Error(u(331));
      var C = He;
      if (He |= 4, Y1(p.current), U1(
        p,
        p.current,
        x,
        a
      ), He = C, lu(0, !1), Je && typeof Je.onPostCommitFiberRoot == "function")
        try {
          Je.onPostCommitFiberRoot(Et, p);
        } catch {
        }
      return !0;
    } finally {
      B.p = d, z.T = o, ub(e, n);
    }
  }
  function ob(e, n, a) {
    n = Ft(a, n), n = jc(e.stateNode, n, 2), e = vr(e, n, 2), e !== null && (jr(e, 2), En(e));
  }
  function Be(e, n, a) {
    if (e.tag === 3)
      ob(e, e, a);
    else
      for (; n !== null; ) {
        if (n.tag === 3) {
          ob(
            n,
            e,
            a
          );
          break;
        } else if (n.tag === 1) {
          var o = n.stateNode;
          if (typeof n.type.getDerivedStateFromError == "function" || typeof o.componentDidCatch == "function" && (xr === null || !xr.has(o))) {
            e = Ft(a, e), a = s1(2), o = vr(n, a, 2), o !== null && (c1(
              a,
              o,
              n,
              e
            ), jr(o, 2), En(o));
            break;
          }
        }
        n = n.return;
      }
  }
  function sf(e, n, a) {
    var o = e.pingCache;
    if (o === null) {
      o = e.pingCache = new Xq();
      var d = /* @__PURE__ */ new Set();
      o.set(n, d);
    } else
      d = o.get(n), d === void 0 && (d = /* @__PURE__ */ new Set(), o.set(n, d));
    d.has(a) || (ef = !0, d.add(a), e = $q.bind(null, e, n, a), n.then(e, e));
  }
  function $q(e, n, a) {
    var o = e.pingCache;
    o !== null && o.delete(n), e.pingedLanes |= e.suspendedLanes & a, e.warmLanes &= ~a, Ye === e && (Re & a) === a && (et === 4 || et === 3 && (Re & 62914560) === Re && 300 > ke() - Kl ? (He & 2) === 0 && Fa(e, 0) : tf |= a, Qa === Re && (Qa = 0)), En(e);
  }
  function sb(e, n) {
    n === 0 && (n = Fu()), e = kr(e, n), e !== null && (jr(e, n), En(e));
  }
  function Fq(e) {
    var n = e.memoizedState, a = 0;
    n !== null && (a = n.retryLane), sb(e, a);
  }
  function Jq(e, n) {
    var a = 0;
    switch (e.tag) {
      case 31:
      case 13:
        var o = e.stateNode, d = e.memoizedState;
        d !== null && (a = d.retryLane);
        break;
      case 19:
        o = e.stateNode;
        break;
      case 22:
        o = e.stateNode._retryCache;
        break;
      default:
        throw Error(u(314));
    }
    o !== null && o.delete(n), sb(e, a);
  }
  function Pq(e, n) {
    return xt(e, n);
  }
  var eo = null, Pa = null, cf = !1, to = !1, ff = !1, wr = 0;
  function En(e) {
    e !== Pa && e.next === null && (Pa === null ? eo = Pa = e : Pa = Pa.next = e), to = !0, cf || (cf = !0, eR());
  }
  function lu(e, n) {
    if (!ff && to) {
      ff = !0;
      do
        for (var a = !1, o = eo; o !== null; ) {
          if (e !== 0) {
            var d = o.pendingLanes;
            if (d === 0) var p = 0;
            else {
              var x = o.suspendedLanes, C = o.pingedLanes;
              p = (1 << 31 - Mt(42 | e) + 1) - 1, p &= d & ~(x & ~C), p = p & 201326741 ? p & 201326741 | 1 : p ? p | 2 : 0;
            }
            p !== 0 && (a = !0, hb(o, p));
          } else
            p = Re, p = ba(
              o,
              o === Ye ? p : 0,
              o.cancelPendingCommit !== null || o.timeoutHandle !== -1
            ), (p & 3) === 0 || Hr(o, p) || (a = !0, hb(o, p));
          o = o.next;
        }
      while (a);
      ff = !1;
    }
  }
  function Wq() {
    cb();
  }
  function cb() {
    to = cf = !1;
    var e = 0;
    wr !== 0 && cR() && (e = wr);
    for (var n = ke(), a = null, o = eo; o !== null; ) {
      var d = o.next, p = fb(o, n);
      p === 0 ? (o.next = null, a === null ? eo = d : a.next = d, d === null && (Pa = a)) : (a = o, (e !== 0 || (p & 3) !== 0) && (to = !0)), o = d;
    }
    ht !== 0 && ht !== 5 || lu(e), wr !== 0 && (wr = 0);
  }
  function fb(e, n) {
    for (var a = e.suspendedLanes, o = e.pingedLanes, d = e.expirationTimes, p = e.pendingLanes & -62914561; 0 < p; ) {
      var x = 31 - Mt(p), C = 1 << x, V = d[x];
      V === -1 ? ((C & a) === 0 || (C & o) !== 0) && (d[x] = As(C, n)) : V <= n && (e.expiredLanes |= C), p &= ~C;
    }
    if (n = Ye, a = Re, a = ba(
      e,
      e === n ? a : 0,
      e.cancelPendingCommit !== null || e.timeoutHandle !== -1
    ), o = e.callbackNode, a === 0 || e === n && (je === 2 || je === 9) || e.cancelPendingCommit !== null)
      return o !== null && o !== null && gt(o), e.callbackNode = null, e.callbackPriority = 0;
    if ((a & 3) === 0 || Hr(e, a)) {
      if (n = a & -a, n === e.callbackPriority) return n;
      switch (o !== null && gt(o), Ei(a)) {
        case 2:
        case 8:
          a = Tt;
          break;
        case 32:
          a = ft;
          break;
        case 268435456:
          a = qn;
          break;
        default:
          a = ft;
      }
      return o = db.bind(null, e), a = xt(a, o), e.callbackPriority = n, e.callbackNode = a, n;
    }
    return o !== null && o !== null && gt(o), e.callbackPriority = 2, e.callbackNode = null, 2;
  }
  function db(e, n) {
    if (ht !== 0 && ht !== 5)
      return e.callbackNode = null, e.callbackPriority = 0, null;
    var a = e.callbackNode;
    if (Wl() && e.callbackNode !== a)
      return null;
    var o = Re;
    return o = ba(
      e,
      e === Ye ? o : 0,
      e.cancelPendingCommit !== null || e.timeoutHandle !== -1
    ), o === 0 ? null : (K1(e, o, n), fb(e, ke()), e.callbackNode != null && e.callbackNode === a ? db.bind(null, e) : null);
  }
  function hb(e, n) {
    if (Wl()) return null;
    K1(e, n, !0);
  }
  function eR() {
    dR(function() {
      (He & 6) !== 0 ? xt(
        qt,
        Wq
      ) : cb();
    });
  }
  function df() {
    if (wr === 0) {
      var e = ja;
      e === 0 && (e = pa, pa <<= 1, (pa & 261888) === 0 && (pa = 256)), wr = e;
    }
    return wr;
  }
  function gb(e) {
    return e == null || typeof e == "symbol" || typeof e == "boolean" ? null : typeof e == "function" ? e : sl("" + e);
  }
  function vb(e, n) {
    var a = n.ownerDocument.createElement("input");
    return a.name = n.name, a.value = n.value, e.id && a.setAttribute("form", e.id), n.parentNode.insertBefore(a, n), e = new FormData(e), a.parentNode.removeChild(a), e;
  }
  function tR(e, n, a, o, d) {
    if (n === "submit" && a && a.stateNode === d) {
      var p = gb(
        (d[wt] || null).action
      ), x = o.submitter;
      x && (n = (n = x[wt] || null) ? gb(n.formAction) : x.getAttribute("formAction"), n !== null && (p = n, x = null));
      var C = new hl(
        "action",
        "action",
        null,
        o,
        d
      );
      e.push({
        event: C,
        listeners: [
          {
            instance: null,
            listener: function() {
              if (o.defaultPrevented) {
                if (wr !== 0) {
                  var V = x ? vb(d, x) : new FormData(d);
                  Nc(
                    a,
                    {
                      pending: !0,
                      data: V,
                      method: d.method,
                      action: p
                    },
                    null,
                    V
                  );
                }
              } else
                typeof p == "function" && (C.preventDefault(), V = x ? vb(d, x) : new FormData(d), Nc(
                  a,
                  {
                    pending: !0,
                    data: V,
                    method: d.method,
                    action: p
                  },
                  p,
                  V
                ));
            },
            currentTarget: d
          }
        ]
      });
    }
  }
  for (var hf = 0; hf < Qs.length; hf++) {
    var gf = Qs[hf], nR = gf.toLowerCase(), rR = gf[0].toUpperCase() + gf.slice(1);
    ln(
      nR,
      "on" + rR
    );
  }
  ln(Zm, "onAnimationEnd"), ln(Km, "onAnimationIteration"), ln(Qm, "onAnimationStart"), ln("dblclick", "onDoubleClick"), ln("focusin", "onFocus"), ln("focusout", "onBlur"), ln(bq, "onTransitionRun"), ln(_q, "onTransitionStart"), ln(xq, "onTransitionCancel"), ln($m, "onTransitionEnd"), ur("onMouseEnter", ["mouseout", "mouseover"]), ur("onMouseLeave", ["mouseout", "mouseover"]), ur("onPointerEnter", ["pointerout", "pointerover"]), ur("onPointerLeave", ["pointerout", "pointerover"]), zn(
    "onChange",
    "change click focusin focusout input keydown keyup selectionchange".split(" ")
  ), zn(
    "onSelect",
    "focusout contextmenu dragend focusin keydown keyup mousedown mouseup selectionchange".split(
      " "
    )
  ), zn("onBeforeInput", [
    "compositionend",
    "keypress",
    "textInput",
    "paste"
  ]), zn(
    "onCompositionEnd",
    "compositionend focusout keydown keypress keyup mousedown".split(" ")
  ), zn(
    "onCompositionStart",
    "compositionstart focusout keydown keypress keyup mousedown".split(" ")
  ), zn(
    "onCompositionUpdate",
    "compositionupdate focusout keydown keypress keyup mousedown".split(" ")
  );
  var ou = "abort canplay canplaythrough durationchange emptied encrypted ended error loadeddata loadedmetadata loadstart pause play playing progress ratechange resize seeked seeking stalled suspend timeupdate volumechange waiting".split(
    " "
  ), aR = new Set(
    "beforetoggle cancel close invalid load scroll scrollend toggle".split(" ").concat(ou)
  );
  function pb(e, n) {
    n = (n & 4) !== 0;
    for (var a = 0; a < e.length; a++) {
      var o = e[a], d = o.event;
      o = o.listeners;
      e: {
        var p = void 0;
        if (n)
          for (var x = o.length - 1; 0 <= x; x--) {
            var C = o[x], V = C.instance, te = C.currentTarget;
            if (C = C.listener, V !== p && d.isPropagationStopped())
              break e;
            p = C, d.currentTarget = te;
            try {
              p(d);
            } catch (oe) {
              pl(oe);
            }
            d.currentTarget = null, p = V;
          }
        else
          for (x = 0; x < o.length; x++) {
            if (C = o[x], V = C.instance, te = C.currentTarget, C = C.listener, V !== p && d.isPropagationStopped())
              break e;
            p = C, d.currentTarget = te;
            try {
              p(d);
            } catch (oe) {
              pl(oe);
            }
            d.currentTarget = null, p = V;
          }
      }
    }
  }
  function qe(e, n) {
    var a = n[_a];
    a === void 0 && (a = n[_a] = /* @__PURE__ */ new Set());
    var o = e + "__bubble";
    a.has(o) || (yb(n, e, 2, !1), a.add(o));
  }
  function vf(e, n, a) {
    var o = 0;
    n && (o |= 4), yb(
      a,
      e,
      o,
      n
    );
  }
  var no = "_reactListening" + Math.random().toString(36).slice(2);
  function pf(e) {
    if (!e[no]) {
      e[no] = !0, al.forEach(function(a) {
        a !== "selectionchange" && (aR.has(a) || vf(a, !1, e), vf(a, !0, e));
      });
      var n = e.nodeType === 9 ? e : e.ownerDocument;
      n === null || n[no] || (n[no] = !0, vf("selectionchange", !1, n));
    }
  }
  function yb(e, n, a, o) {
    switch (Zb(n)) {
      case 2:
        var d = zR;
        break;
      case 8:
        d = OR;
        break;
      default:
        d = Nf;
    }
    a = d.bind(
      null,
      n,
      a,
      e
    ), d = void 0, !Ls || n !== "touchstart" && n !== "touchmove" && n !== "wheel" || (d = !0), o ? d !== void 0 ? e.addEventListener(n, a, {
      capture: !0,
      passive: d
    }) : e.addEventListener(n, a, !0) : d !== void 0 ? e.addEventListener(n, a, {
      passive: d
    }) : e.addEventListener(n, a, !1);
  }
  function yf(e, n, a, o, d) {
    var p = o;
    if ((n & 1) === 0 && (n & 2) === 0 && o !== null)
      e: for (; ; ) {
        if (o === null) return;
        var x = o.tag;
        if (x === 3 || x === 4) {
          var C = o.stateNode.containerInfo;
          if (C === d) break;
          if (x === 4)
            for (x = o.return; x !== null; ) {
              var V = x.tag;
              if ((V === 3 || V === 4) && x.stateNode.containerInfo === d)
                return;
              x = x.return;
            }
          for (; C !== null; ) {
            if (x = nr(C), x === null) return;
            if (V = x.tag, V === 5 || V === 6 || V === 26 || V === 27) {
              o = p = x;
              continue e;
            }
            C = C.parentNode;
          }
        }
        o = o.return;
      }
    Sm(function() {
      var te = p, oe = Os(a), fe = [];
      e: {
        var ne = Fm.get(e);
        if (ne !== void 0) {
          var le = hl, pe = e;
          switch (e) {
            case "keypress":
              if (fl(a) === 0) break e;
            case "keydown":
            case "keyup":
              le = JC;
              break;
            case "focusin":
              pe = "focus", le = Us;
              break;
            case "focusout":
              pe = "blur", le = Us;
              break;
            case "beforeblur":
            case "afterblur":
              le = Us;
              break;
            case "click":
              if (a.button === 2) break e;
            case "auxclick":
            case "dblclick":
            case "mousedown":
            case "mousemove":
            case "mouseup":
            case "mouseout":
            case "mouseover":
            case "contextmenu":
              le = Am;
              break;
            case "drag":
            case "dragend":
            case "dragenter":
            case "dragexit":
            case "dragleave":
            case "dragover":
            case "dragstart":
            case "drop":
              le = UC;
              break;
            case "touchcancel":
            case "touchend":
            case "touchmove":
            case "touchstart":
              le = eq;
              break;
            case Zm:
            case Km:
            case Qm:
              le = YC;
              break;
            case $m:
              le = nq;
              break;
            case "scroll":
            case "scrollend":
              le = jC;
              break;
            case "wheel":
              le = aq;
              break;
            case "copy":
            case "cut":
            case "paste":
              le = XC;
              break;
            case "gotpointercapture":
            case "lostpointercapture":
            case "pointercancel":
            case "pointerdown":
            case "pointermove":
            case "pointerout":
            case "pointerover":
            case "pointerup":
              le = Mm;
              break;
            case "toggle":
            case "beforetoggle":
              le = uq;
          }
          var Ee = (n & 4) !== 0, Ve = !Ee && (e === "scroll" || e === "scrollend"), F = Ee ? ne !== null ? ne + "Capture" : null : ne;
          Ee = [];
          for (var I = te, ee; I !== null; ) {
            var ce = I;
            if (ee = ce.stateNode, ce = ce.tag, ce !== 5 && ce !== 26 && ce !== 27 || ee === null || F === null || (ce = Ri(I, F), ce != null && Ee.push(
              su(I, ce, ee)
            )), Ve) break;
            I = I.return;
          }
          0 < Ee.length && (ne = new le(
            ne,
            pe,
            null,
            a,
            oe
          ), fe.push({ event: ne, listeners: Ee }));
        }
      }
      if ((n & 7) === 0) {
        e: {
          if (ne = e === "mouseover" || e === "pointerover", le = e === "mouseout" || e === "pointerout", ne && a !== zs && (pe = a.relatedTarget || a.fromElement) && (nr(pe) || pe[Nn]))
            break e;
          if ((le || ne) && (ne = oe.window === oe ? oe : (ne = oe.ownerDocument) ? ne.defaultView || ne.parentWindow : window, le ? (pe = a.relatedTarget || a.toElement, le = te, pe = pe ? nr(pe) : null, pe !== null && (Ve = s(pe), Ee = pe.tag, pe !== Ve || Ee !== 5 && Ee !== 27 && Ee !== 6) && (pe = null)) : (le = null, pe = te), le !== pe)) {
            if (Ee = Am, ce = "onMouseLeave", F = "onMouseEnter", I = "mouse", (e === "pointerout" || e === "pointerover") && (Ee = Mm, ce = "onPointerLeave", F = "onPointerEnter", I = "pointer"), Ve = le == null ? ne : ar(le), ee = pe == null ? ne : ar(pe), ne = new Ee(
              ce,
              I + "leave",
              le,
              a,
              oe
            ), ne.target = Ve, ne.relatedTarget = ee, ce = null, nr(oe) === te && (Ee = new Ee(
              F,
              I + "enter",
              pe,
              a,
              oe
            ), Ee.target = ee, Ee.relatedTarget = Ve, ce = Ee), Ve = ce, le && pe)
              t: {
                for (Ee = iR, F = le, I = pe, ee = 0, ce = F; ce; ce = Ee(ce))
                  ee++;
                ce = 0;
                for (var Se = I; Se; Se = Ee(Se))
                  ce++;
                for (; 0 < ee - ce; )
                  F = Ee(F), ee--;
                for (; 0 < ce - ee; )
                  I = Ee(I), ce--;
                for (; ee--; ) {
                  if (F === I || I !== null && F === I.alternate) {
                    Ee = F;
                    break t;
                  }
                  F = Ee(F), I = Ee(I);
                }
                Ee = null;
              }
            else Ee = null;
            le !== null && mb(
              fe,
              ne,
              le,
              Ee,
              !1
            ), pe !== null && Ve !== null && mb(
              fe,
              Ve,
              pe,
              Ee,
              !0
            );
          }
        }
        e: {
          if (ne = te ? ar(te) : window, le = ne.nodeName && ne.nodeName.toLowerCase(), le === "select" || le === "input" && ne.type === "file")
            var De = Lm;
          else if (Om(ne))
            if (Hm)
              De = pq;
            else {
              De = gq;
              var be = hq;
            }
          else
            le = ne.nodeName, !le || le.toLowerCase() !== "input" || ne.type !== "checkbox" && ne.type !== "radio" ? te && Ns(te.elementType) && (De = Lm) : De = vq;
          if (De && (De = De(e, te))) {
            Dm(
              fe,
              De,
              a,
              oe
            );
            break e;
          }
          be && be(e, ne, te), e === "focusout" && te && ne.type === "number" && te.memoizedProps.value != null && Ci(ne, "number", ne.value);
        }
        switch (be = te ? ar(te) : window, e) {
          case "focusin":
            (Om(be) || be.contentEditable === "true") && (qa = be, Is = te, Bi = null);
            break;
          case "focusout":
            Bi = Is = qa = null;
            break;
          case "mousedown":
            Zs = !0;
            break;
          case "contextmenu":
          case "mouseup":
          case "dragend":
            Zs = !1, Xm(fe, a, oe);
            break;
          case "selectionchange":
            if (mq) break;
          case "keydown":
          case "keyup":
            Xm(fe, a, oe);
        }
        var Me;
        if (Vs)
          e: {
            switch (e) {
              case "compositionstart":
                var Ne = "onCompositionStart";
                break e;
              case "compositionend":
                Ne = "onCompositionEnd";
                break e;
              case "compositionupdate":
                Ne = "onCompositionUpdate";
                break e;
            }
            Ne = void 0;
          }
        else
          Ca ? Nm(e, a) && (Ne = "onCompositionEnd") : e === "keydown" && a.keyCode === 229 && (Ne = "onCompositionStart");
        Ne && (Cm && a.locale !== "ko" && (Ca || Ne !== "onCompositionStart" ? Ne === "onCompositionEnd" && Ca && (Me = Em()) : (or = oe, Hs = "value" in or ? or.value : or.textContent, Ca = !0)), be = ro(te, Ne), 0 < be.length && (Ne = new Tm(
          Ne,
          e,
          null,
          a,
          oe
        ), fe.push({ event: Ne, listeners: be }), Me ? Ne.data = Me : (Me = zm(a), Me !== null && (Ne.data = Me)))), (Me = oq ? sq(e, a) : cq(e, a)) && (Ne = ro(te, "onBeforeInput"), 0 < Ne.length && (be = new Tm(
          "onBeforeInput",
          "beforeinput",
          null,
          a,
          oe
        ), fe.push({
          event: be,
          listeners: Ne
        }), be.data = Me)), tR(
          fe,
          e,
          te,
          a,
          oe
        );
      }
      pb(fe, n);
    });
  }
  function su(e, n, a) {
    return {
      instance: e,
      listener: n,
      currentTarget: a
    };
  }
  function ro(e, n) {
    for (var a = n + "Capture", o = []; e !== null; ) {
      var d = e, p = d.stateNode;
      if (d = d.tag, d !== 5 && d !== 26 && d !== 27 || p === null || (d = Ri(e, a), d != null && o.unshift(
        su(e, d, p)
      ), d = Ri(e, n), d != null && o.push(
        su(e, d, p)
      )), e.tag === 3) return o;
      e = e.return;
    }
    return [];
  }
  function iR(e) {
    if (e === null) return null;
    do
      e = e.return;
    while (e && e.tag !== 5 && e.tag !== 27);
    return e || null;
  }
  function mb(e, n, a, o, d) {
    for (var p = n._reactName, x = []; a !== null && a !== o; ) {
      var C = a, V = C.alternate, te = C.stateNode;
      if (C = C.tag, V !== null && V === o) break;
      C !== 5 && C !== 26 && C !== 27 || te === null || (V = te, d ? (te = Ri(a, p), te != null && x.unshift(
        su(a, te, V)
      )) : d || (te = Ri(a, p), te != null && x.push(
        su(a, te, V)
      ))), a = a.return;
    }
    x.length !== 0 && e.push({ event: n, listeners: x });
  }
  var uR = /\r\n?/g, lR = /\u0000|\uFFFD/g;
  function bb(e) {
    return (typeof e == "string" ? e : "" + e).replace(uR, `
`).replace(lR, "");
  }
  function _b(e, n) {
    return n = bb(n), bb(e) === n;
  }
  function Ge(e, n, a, o, d, p) {
    switch (a) {
      case "children":
        typeof o == "string" ? n === "body" || n === "textarea" && o === "" || Aa(e, o) : (typeof o == "number" || typeof o == "bigint") && n !== "body" && Aa(e, "" + o);
        break;
      case "className":
        Sa(e, "class", o);
        break;
      case "tabIndex":
        Sa(e, "tabindex", o);
        break;
      case "dir":
      case "role":
      case "viewBox":
      case "width":
      case "height":
        Sa(e, a, o);
        break;
      case "style":
        _m(e, o, p);
        break;
      case "data":
        if (n !== "object") {
          Sa(e, "data", o);
          break;
        }
      case "src":
      case "href":
        if (o === "" && (n !== "a" || a !== "href")) {
          e.removeAttribute(a);
          break;
        }
        if (o == null || typeof o == "function" || typeof o == "symbol" || typeof o == "boolean") {
          e.removeAttribute(a);
          break;
        }
        o = sl("" + o), e.setAttribute(a, o);
        break;
      case "action":
      case "formAction":
        if (typeof o == "function") {
          e.setAttribute(
            a,
            "javascript:throw new Error('A React form was unexpectedly submitted. If you called form.submit() manually, consider using form.requestSubmit() instead. If you\\'re trying to use event.stopPropagation() in a submit event handler, consider also calling event.preventDefault().')"
          );
          break;
        } else
          typeof p == "function" && (a === "formAction" ? (n !== "input" && Ge(e, n, "name", d.name, d, null), Ge(
            e,
            n,
            "formEncType",
            d.formEncType,
            d,
            null
          ), Ge(
            e,
            n,
            "formMethod",
            d.formMethod,
            d,
            null
          ), Ge(
            e,
            n,
            "formTarget",
            d.formTarget,
            d,
            null
          )) : (Ge(e, n, "encType", d.encType, d, null), Ge(e, n, "method", d.method, d, null), Ge(e, n, "target", d.target, d, null)));
        if (o == null || typeof o == "symbol" || typeof o == "boolean") {
          e.removeAttribute(a);
          break;
        }
        o = sl("" + o), e.setAttribute(a, o);
        break;
      case "onClick":
        o != null && (e.onclick = On);
        break;
      case "onScroll":
        o != null && qe("scroll", e);
        break;
      case "onScrollEnd":
        o != null && qe("scrollend", e);
        break;
      case "dangerouslySetInnerHTML":
        if (o != null) {
          if (typeof o != "object" || !("__html" in o))
            throw Error(u(61));
          if (a = o.__html, a != null) {
            if (d.children != null) throw Error(u(60));
            e.innerHTML = a;
          }
        }
        break;
      case "multiple":
        e.multiple = o && typeof o != "function" && typeof o != "symbol";
        break;
      case "muted":
        e.muted = o && typeof o != "function" && typeof o != "symbol";
        break;
      case "suppressContentEditableWarning":
      case "suppressHydrationWarning":
      case "defaultValue":
      case "defaultChecked":
      case "innerHTML":
      case "ref":
        break;
      case "autoFocus":
        break;
      case "xlinkHref":
        if (o == null || typeof o == "function" || typeof o == "boolean" || typeof o == "symbol") {
          e.removeAttribute("xlink:href");
          break;
        }
        a = sl("" + o), e.setAttributeNS(
          "http://www.w3.org/1999/xlink",
          "xlink:href",
          a
        );
        break;
      case "contentEditable":
      case "spellCheck":
      case "draggable":
      case "value":
      case "autoReverse":
      case "externalResourcesRequired":
      case "focusable":
      case "preserveAlpha":
        o != null && typeof o != "function" && typeof o != "symbol" ? e.setAttribute(a, "" + o) : e.removeAttribute(a);
        break;
      case "inert":
      case "allowFullScreen":
      case "async":
      case "autoPlay":
      case "controls":
      case "default":
      case "defer":
      case "disabled":
      case "disablePictureInPicture":
      case "disableRemotePlayback":
      case "formNoValidate":
      case "hidden":
      case "loop":
      case "noModule":
      case "noValidate":
      case "open":
      case "playsInline":
      case "readOnly":
      case "required":
      case "reversed":
      case "scoped":
      case "seamless":
      case "itemScope":
        o && typeof o != "function" && typeof o != "symbol" ? e.setAttribute(a, "") : e.removeAttribute(a);
        break;
      case "capture":
      case "download":
        o === !0 ? e.setAttribute(a, "") : o !== !1 && o != null && typeof o != "function" && typeof o != "symbol" ? e.setAttribute(a, o) : e.removeAttribute(a);
        break;
      case "cols":
      case "rows":
      case "size":
      case "span":
        o != null && typeof o != "function" && typeof o != "symbol" && !isNaN(o) && 1 <= o ? e.setAttribute(a, o) : e.removeAttribute(a);
        break;
      case "rowSpan":
      case "start":
        o == null || typeof o == "function" || typeof o == "symbol" || isNaN(o) ? e.removeAttribute(a) : e.setAttribute(a, o);
        break;
      case "popover":
        qe("beforetoggle", e), qe("toggle", e), xa(e, "popover", o);
        break;
      case "xlinkActuate":
        un(
          e,
          "http://www.w3.org/1999/xlink",
          "xlink:actuate",
          o
        );
        break;
      case "xlinkArcrole":
        un(
          e,
          "http://www.w3.org/1999/xlink",
          "xlink:arcrole",
          o
        );
        break;
      case "xlinkRole":
        un(
          e,
          "http://www.w3.org/1999/xlink",
          "xlink:role",
          o
        );
        break;
      case "xlinkShow":
        un(
          e,
          "http://www.w3.org/1999/xlink",
          "xlink:show",
          o
        );
        break;
      case "xlinkTitle":
        un(
          e,
          "http://www.w3.org/1999/xlink",
          "xlink:title",
          o
        );
        break;
      case "xlinkType":
        un(
          e,
          "http://www.w3.org/1999/xlink",
          "xlink:type",
          o
        );
        break;
      case "xmlBase":
        un(
          e,
          "http://www.w3.org/XML/1998/namespace",
          "xml:base",
          o
        );
        break;
      case "xmlLang":
        un(
          e,
          "http://www.w3.org/XML/1998/namespace",
          "xml:lang",
          o
        );
        break;
      case "xmlSpace":
        un(
          e,
          "http://www.w3.org/XML/1998/namespace",
          "xml:space",
          o
        );
        break;
      case "is":
        xa(e, "is", o);
        break;
      case "innerText":
      case "textContent":
        break;
      default:
        (!(2 < a.length) || a[0] !== "o" && a[0] !== "O" || a[1] !== "n" && a[1] !== "N") && (a = LC.get(a) || a, xa(e, a, o));
    }
  }
  function mf(e, n, a, o, d, p) {
    switch (a) {
      case "style":
        _m(e, o, p);
        break;
      case "dangerouslySetInnerHTML":
        if (o != null) {
          if (typeof o != "object" || !("__html" in o))
            throw Error(u(61));
          if (a = o.__html, a != null) {
            if (d.children != null) throw Error(u(60));
            e.innerHTML = a;
          }
        }
        break;
      case "children":
        typeof o == "string" ? Aa(e, o) : (typeof o == "number" || typeof o == "bigint") && Aa(e, "" + o);
        break;
      case "onScroll":
        o != null && qe("scroll", e);
        break;
      case "onScrollEnd":
        o != null && qe("scrollend", e);
        break;
      case "onClick":
        o != null && (e.onclick = On);
        break;
      case "suppressContentEditableWarning":
      case "suppressHydrationWarning":
      case "innerHTML":
      case "ref":
        break;
      case "innerText":
      case "textContent":
        break;
      default:
        if (!il.hasOwnProperty(a))
          e: {
            if (a[0] === "o" && a[1] === "n" && (d = a.endsWith("Capture"), n = a.slice(2, d ? a.length - 7 : void 0), p = e[wt] || null, p = p != null ? p[a] : null, typeof p == "function" && e.removeEventListener(n, p, d), typeof o == "function")) {
              typeof p != "function" && p !== null && (a in e ? e[a] = null : e.hasAttribute(a) && e.removeAttribute(a)), e.addEventListener(n, o, d);
              break e;
            }
            a in e ? e[a] = o : o === !0 ? e.setAttribute(a, "") : xa(e, a, o);
          }
    }
  }
  function _t(e, n, a) {
    switch (n) {
      case "div":
      case "span":
      case "svg":
      case "path":
      case "a":
      case "g":
      case "p":
      case "li":
        break;
      case "img":
        qe("error", e), qe("load", e);
        var o = !1, d = !1, p;
        for (p in a)
          if (a.hasOwnProperty(p)) {
            var x = a[p];
            if (x != null)
              switch (p) {
                case "src":
                  o = !0;
                  break;
                case "srcSet":
                  d = !0;
                  break;
                case "children":
                case "dangerouslySetInnerHTML":
                  throw Error(u(137, n));
                default:
                  Ge(e, n, p, x, a, null);
              }
          }
        d && Ge(e, n, "srcSet", a.srcSet, a, null), o && Ge(e, n, "src", a.src, a, null);
        return;
      case "input":
        qe("invalid", e);
        var C = p = x = d = null, V = null, te = null;
        for (o in a)
          if (a.hasOwnProperty(o)) {
            var oe = a[o];
            if (oe != null)
              switch (o) {
                case "name":
                  d = oe;
                  break;
                case "type":
                  x = oe;
                  break;
                case "checked":
                  V = oe;
                  break;
                case "defaultChecked":
                  te = oe;
                  break;
                case "value":
                  p = oe;
                  break;
                case "defaultValue":
                  C = oe;
                  break;
                case "children":
                case "dangerouslySetInnerHTML":
                  if (oe != null)
                    throw Error(u(137, n));
                  break;
                default:
                  Ge(e, n, o, oe, a, null);
              }
          }
        wa(
          e,
          p,
          C,
          V,
          te,
          x,
          d,
          !1
        );
        return;
      case "select":
        qe("invalid", e), o = x = p = null;
        for (d in a)
          if (a.hasOwnProperty(d) && (C = a[d], C != null))
            switch (d) {
              case "value":
                p = C;
                break;
              case "defaultValue":
                x = C;
                break;
              case "multiple":
                o = C;
              default:
                Ge(e, n, d, C, a, null);
            }
        n = p, a = x, e.multiple = !!o, n != null ? lr(e, !!o, n, !1) : a != null && lr(e, !!o, a, !0);
        return;
      case "textarea":
        qe("invalid", e), p = d = o = null;
        for (x in a)
          if (a.hasOwnProperty(x) && (C = a[x], C != null))
            switch (x) {
              case "value":
                o = C;
                break;
              case "defaultValue":
                d = C;
                break;
              case "children":
                p = C;
                break;
              case "dangerouslySetInnerHTML":
                if (C != null) throw Error(u(91));
                break;
              default:
                Ge(e, n, x, C, a, null);
            }
        mm(e, o, d, p);
        return;
      case "option":
        for (V in a)
          if (a.hasOwnProperty(V) && (o = a[V], o != null))
            switch (V) {
              case "selected":
                e.selected = o && typeof o != "function" && typeof o != "symbol";
                break;
              default:
                Ge(e, n, V, o, a, null);
            }
        return;
      case "dialog":
        qe("beforetoggle", e), qe("toggle", e), qe("cancel", e), qe("close", e);
        break;
      case "iframe":
      case "object":
        qe("load", e);
        break;
      case "video":
      case "audio":
        for (o = 0; o < ou.length; o++)
          qe(ou[o], e);
        break;
      case "image":
        qe("error", e), qe("load", e);
        break;
      case "details":
        qe("toggle", e);
        break;
      case "embed":
      case "source":
      case "link":
        qe("error", e), qe("load", e);
      case "area":
      case "base":
      case "br":
      case "col":
      case "hr":
      case "keygen":
      case "meta":
      case "param":
      case "track":
      case "wbr":
      case "menuitem":
        for (te in a)
          if (a.hasOwnProperty(te) && (o = a[te], o != null))
            switch (te) {
              case "children":
              case "dangerouslySetInnerHTML":
                throw Error(u(137, n));
              default:
                Ge(e, n, te, o, a, null);
            }
        return;
      default:
        if (Ns(n)) {
          for (oe in a)
            a.hasOwnProperty(oe) && (o = a[oe], o !== void 0 && mf(
              e,
              n,
              oe,
              o,
              a,
              void 0
            ));
          return;
        }
    }
    for (C in a)
      a.hasOwnProperty(C) && (o = a[C], o != null && Ge(e, n, C, o, a, null));
  }
  function oR(e, n, a, o) {
    switch (n) {
      case "div":
      case "span":
      case "svg":
      case "path":
      case "a":
      case "g":
      case "p":
      case "li":
        break;
      case "input":
        var d = null, p = null, x = null, C = null, V = null, te = null, oe = null;
        for (le in a) {
          var fe = a[le];
          if (a.hasOwnProperty(le) && fe != null)
            switch (le) {
              case "checked":
                break;
              case "value":
                break;
              case "defaultValue":
                V = fe;
              default:
                o.hasOwnProperty(le) || Ge(e, n, le, null, o, fe);
            }
        }
        for (var ne in o) {
          var le = o[ne];
          if (fe = a[ne], o.hasOwnProperty(ne) && (le != null || fe != null))
            switch (ne) {
              case "type":
                p = le;
                break;
              case "name":
                d = le;
                break;
              case "checked":
                te = le;
                break;
              case "defaultChecked":
                oe = le;
                break;
              case "value":
                x = le;
                break;
              case "defaultValue":
                C = le;
                break;
              case "children":
              case "dangerouslySetInnerHTML":
                if (le != null)
                  throw Error(u(137, n));
                break;
              default:
                le !== fe && Ge(
                  e,
                  n,
                  ne,
                  le,
                  o,
                  fe
                );
            }
        }
        Mi(
          e,
          x,
          C,
          V,
          te,
          oe,
          p,
          d
        );
        return;
      case "select":
        le = x = C = ne = null;
        for (p in a)
          if (V = a[p], a.hasOwnProperty(p) && V != null)
            switch (p) {
              case "value":
                break;
              case "multiple":
                le = V;
              default:
                o.hasOwnProperty(p) || Ge(
                  e,
                  n,
                  p,
                  null,
                  o,
                  V
                );
            }
        for (d in o)
          if (p = o[d], V = a[d], o.hasOwnProperty(d) && (p != null || V != null))
            switch (d) {
              case "value":
                ne = p;
                break;
              case "defaultValue":
                C = p;
                break;
              case "multiple":
                x = p;
              default:
                p !== V && Ge(
                  e,
                  n,
                  d,
                  p,
                  o,
                  V
                );
            }
        n = C, a = x, o = le, ne != null ? lr(e, !!a, ne, !1) : !!o != !!a && (n != null ? lr(e, !!a, n, !0) : lr(e, !!a, a ? [] : "", !1));
        return;
      case "textarea":
        le = ne = null;
        for (C in a)
          if (d = a[C], a.hasOwnProperty(C) && d != null && !o.hasOwnProperty(C))
            switch (C) {
              case "value":
                break;
              case "children":
                break;
              default:
                Ge(e, n, C, null, o, d);
            }
        for (x in o)
          if (d = o[x], p = a[x], o.hasOwnProperty(x) && (d != null || p != null))
            switch (x) {
              case "value":
                ne = d;
                break;
              case "defaultValue":
                le = d;
                break;
              case "children":
                break;
              case "dangerouslySetInnerHTML":
                if (d != null) throw Error(u(91));
                break;
              default:
                d !== p && Ge(e, n, x, d, o, p);
            }
        qi(e, ne, le);
        return;
      case "option":
        for (var pe in a)
          if (ne = a[pe], a.hasOwnProperty(pe) && ne != null && !o.hasOwnProperty(pe))
            switch (pe) {
              case "selected":
                e.selected = !1;
                break;
              default:
                Ge(
                  e,
                  n,
                  pe,
                  null,
                  o,
                  ne
                );
            }
        for (V in o)
          if (ne = o[V], le = a[V], o.hasOwnProperty(V) && ne !== le && (ne != null || le != null))
            switch (V) {
              case "selected":
                e.selected = ne && typeof ne != "function" && typeof ne != "symbol";
                break;
              default:
                Ge(
                  e,
                  n,
                  V,
                  ne,
                  o,
                  le
                );
            }
        return;
      case "img":
      case "link":
      case "area":
      case "base":
      case "br":
      case "col":
      case "embed":
      case "hr":
      case "keygen":
      case "meta":
      case "param":
      case "source":
      case "track":
      case "wbr":
      case "menuitem":
        for (var Ee in a)
          ne = a[Ee], a.hasOwnProperty(Ee) && ne != null && !o.hasOwnProperty(Ee) && Ge(e, n, Ee, null, o, ne);
        for (te in o)
          if (ne = o[te], le = a[te], o.hasOwnProperty(te) && ne !== le && (ne != null || le != null))
            switch (te) {
              case "children":
              case "dangerouslySetInnerHTML":
                if (ne != null)
                  throw Error(u(137, n));
                break;
              default:
                Ge(
                  e,
                  n,
                  te,
                  ne,
                  o,
                  le
                );
            }
        return;
      default:
        if (Ns(n)) {
          for (var Ve in a)
            ne = a[Ve], a.hasOwnProperty(Ve) && ne !== void 0 && !o.hasOwnProperty(Ve) && mf(
              e,
              n,
              Ve,
              void 0,
              o,
              ne
            );
          for (oe in o)
            ne = o[oe], le = a[oe], !o.hasOwnProperty(oe) || ne === le || ne === void 0 && le === void 0 || mf(
              e,
              n,
              oe,
              ne,
              o,
              le
            );
          return;
        }
    }
    for (var F in a)
      ne = a[F], a.hasOwnProperty(F) && ne != null && !o.hasOwnProperty(F) && Ge(e, n, F, null, o, ne);
    for (fe in o)
      ne = o[fe], le = a[fe], !o.hasOwnProperty(fe) || ne === le || ne == null && le == null || Ge(e, n, fe, ne, o, le);
  }
  function xb(e) {
    switch (e) {
      case "css":
      case "script":
      case "font":
      case "img":
      case "image":
      case "input":
      case "link":
        return !0;
      default:
        return !1;
    }
  }
  function sR() {
    if (typeof performance.getEntriesByType == "function") {
      for (var e = 0, n = 0, a = performance.getEntriesByType("resource"), o = 0; o < a.length; o++) {
        var d = a[o], p = d.transferSize, x = d.initiatorType, C = d.duration;
        if (p && C && xb(x)) {
          for (x = 0, C = d.responseEnd, o += 1; o < a.length; o++) {
            var V = a[o], te = V.startTime;
            if (te > C) break;
            var oe = V.transferSize, fe = V.initiatorType;
            oe && xb(fe) && (V = V.responseEnd, x += oe * (V < C ? 1 : (C - te) / (V - te)));
          }
          if (--o, n += 8 * (p + x) / (d.duration / 1e3), e++, 10 < e) break;
        }
      }
      if (0 < e) return n / e / 1e6;
    }
    return navigator.connection && (e = navigator.connection.downlink, typeof e == "number") ? e : 5;
  }
  var bf = null, _f = null;
  function ao(e) {
    return e.nodeType === 9 ? e : e.ownerDocument;
  }
  function Sb(e) {
    switch (e) {
      case "http://www.w3.org/2000/svg":
        return 1;
      case "http://www.w3.org/1998/Math/MathML":
        return 2;
      default:
        return 0;
    }
  }
  function Eb(e, n) {
    if (e === 0)
      switch (n) {
        case "svg":
          return 1;
        case "math":
          return 2;
        default:
          return 0;
      }
    return e === 1 && n === "foreignObject" ? 0 : e;
  }
  function xf(e, n) {
    return e === "textarea" || e === "noscript" || typeof n.children == "string" || typeof n.children == "number" || typeof n.children == "bigint" || typeof n.dangerouslySetInnerHTML == "object" && n.dangerouslySetInnerHTML !== null && n.dangerouslySetInnerHTML.__html != null;
  }
  var Sf = null;
  function cR() {
    var e = window.event;
    return e && e.type === "popstate" ? e === Sf ? !1 : (Sf = e, !0) : (Sf = null, !1);
  }
  var wb = typeof setTimeout == "function" ? setTimeout : void 0, fR = typeof clearTimeout == "function" ? clearTimeout : void 0, Ab = typeof Promise == "function" ? Promise : void 0, dR = typeof queueMicrotask == "function" ? queueMicrotask : typeof Ab < "u" ? function(e) {
    return Ab.resolve(null).then(e).catch(hR);
  } : wb;
  function hR(e) {
    setTimeout(function() {
      throw e;
    });
  }
  function Ar(e) {
    return e === "head";
  }
  function Tb(e, n) {
    var a = n, o = 0;
    do {
      var d = a.nextSibling;
      if (e.removeChild(a), d && d.nodeType === 8)
        if (a = d.data, a === "/$" || a === "/&") {
          if (o === 0) {
            e.removeChild(d), ni(n);
            return;
          }
          o--;
        } else if (a === "$" || a === "$?" || a === "$~" || a === "$!" || a === "&")
          o++;
        else if (a === "html")
          cu(e.ownerDocument.documentElement);
        else if (a === "head") {
          a = e.ownerDocument.head, cu(a);
          for (var p = a.firstChild; p; ) {
            var x = p.nextSibling, C = p.nodeName;
            p[Br] || C === "SCRIPT" || C === "STYLE" || C === "LINK" && p.rel.toLowerCase() === "stylesheet" || a.removeChild(p), p = x;
          }
        } else
          a === "body" && cu(e.ownerDocument.body);
      a = d;
    } while (a);
    ni(n);
  }
  function Mb(e, n) {
    var a = e;
    e = 0;
    do {
      var o = a.nextSibling;
      if (a.nodeType === 1 ? n ? (a._stashedDisplay = a.style.display, a.style.display = "none") : (a.style.display = a._stashedDisplay || "", a.getAttribute("style") === "" && a.removeAttribute("style")) : a.nodeType === 3 && (n ? (a._stashedText = a.nodeValue, a.nodeValue = "") : a.nodeValue = a._stashedText || ""), o && o.nodeType === 8)
        if (a = o.data, a === "/$") {
          if (e === 0) break;
          e--;
        } else
          a !== "$" && a !== "$?" && a !== "$~" && a !== "$!" || e++;
      a = o;
    } while (a);
  }
  function Ef(e) {
    var n = e.firstChild;
    for (n && n.nodeType === 10 && (n = n.nextSibling); n; ) {
      var a = n;
      switch (n = n.nextSibling, a.nodeName) {
        case "HTML":
        case "HEAD":
        case "BODY":
          Ef(a), wi(a);
          continue;
        case "SCRIPT":
        case "STYLE":
          continue;
        case "LINK":
          if (a.rel.toLowerCase() === "stylesheet") continue;
      }
      e.removeChild(a);
    }
  }
  function gR(e, n, a, o) {
    for (; e.nodeType === 1; ) {
      var d = a;
      if (e.nodeName.toLowerCase() !== n.toLowerCase()) {
        if (!o && (e.nodeName !== "INPUT" || e.type !== "hidden"))
          break;
      } else if (o) {
        if (!e[Br])
          switch (n) {
            case "meta":
              if (!e.hasAttribute("itemprop")) break;
              return e;
            case "link":
              if (p = e.getAttribute("rel"), p === "stylesheet" && e.hasAttribute("data-precedence"))
                break;
              if (p !== d.rel || e.getAttribute("href") !== (d.href == null || d.href === "" ? null : d.href) || e.getAttribute("crossorigin") !== (d.crossOrigin == null ? null : d.crossOrigin) || e.getAttribute("title") !== (d.title == null ? null : d.title))
                break;
              return e;
            case "style":
              if (e.hasAttribute("data-precedence")) break;
              return e;
            case "script":
              if (p = e.getAttribute("src"), (p !== (d.src == null ? null : d.src) || e.getAttribute("type") !== (d.type == null ? null : d.type) || e.getAttribute("crossorigin") !== (d.crossOrigin == null ? null : d.crossOrigin)) && p && e.hasAttribute("async") && !e.hasAttribute("itemprop"))
                break;
              return e;
            default:
              return e;
          }
      } else if (n === "input" && e.type === "hidden") {
        var p = d.name == null ? null : "" + d.name;
        if (d.type === "hidden" && e.getAttribute("name") === p)
          return e;
      } else return e;
      if (e = tn(e.nextSibling), e === null) break;
    }
    return null;
  }
  function vR(e, n, a) {
    if (n === "") return null;
    for (; e.nodeType !== 3; )
      if ((e.nodeType !== 1 || e.nodeName !== "INPUT" || e.type !== "hidden") && !a || (e = tn(e.nextSibling), e === null)) return null;
    return e;
  }
  function Cb(e, n) {
    for (; e.nodeType !== 8; )
      if ((e.nodeType !== 1 || e.nodeName !== "INPUT" || e.type !== "hidden") && !n || (e = tn(e.nextSibling), e === null)) return null;
    return e;
  }
  function wf(e) {
    return e.data === "$?" || e.data === "$~";
  }
  function Af(e) {
    return e.data === "$!" || e.data === "$?" && e.ownerDocument.readyState !== "loading";
  }
  function pR(e, n) {
    var a = e.ownerDocument;
    if (e.data === "$~") e._reactRetry = n;
    else if (e.data !== "$?" || a.readyState !== "loading")
      n();
    else {
      var o = function() {
        n(), a.removeEventListener("DOMContentLoaded", o);
      };
      a.addEventListener("DOMContentLoaded", o), e._reactRetry = o;
    }
  }
  function tn(e) {
    for (; e != null; e = e.nextSibling) {
      var n = e.nodeType;
      if (n === 1 || n === 3) break;
      if (n === 8) {
        if (n = e.data, n === "$" || n === "$!" || n === "$?" || n === "$~" || n === "&" || n === "F!" || n === "F")
          break;
        if (n === "/$" || n === "/&") return null;
      }
    }
    return e;
  }
  var Tf = null;
  function qb(e) {
    e = e.nextSibling;
    for (var n = 0; e; ) {
      if (e.nodeType === 8) {
        var a = e.data;
        if (a === "/$" || a === "/&") {
          if (n === 0)
            return tn(e.nextSibling);
          n--;
        } else
          a !== "$" && a !== "$!" && a !== "$?" && a !== "$~" && a !== "&" || n++;
      }
      e = e.nextSibling;
    }
    return null;
  }
  function Rb(e) {
    e = e.previousSibling;
    for (var n = 0; e; ) {
      if (e.nodeType === 8) {
        var a = e.data;
        if (a === "$" || a === "$!" || a === "$?" || a === "$~" || a === "&") {
          if (n === 0) return e;
          n--;
        } else a !== "/$" && a !== "/&" || n++;
      }
      e = e.previousSibling;
    }
    return null;
  }
  function Nb(e, n, a) {
    switch (n = ao(a), e) {
      case "html":
        if (e = n.documentElement, !e) throw Error(u(452));
        return e;
      case "head":
        if (e = n.head, !e) throw Error(u(453));
        return e;
      case "body":
        if (e = n.body, !e) throw Error(u(454));
        return e;
      default:
        throw Error(u(451));
    }
  }
  function cu(e) {
    for (var n = e.attributes; n.length; )
      e.removeAttributeNode(n[0]);
    wi(e);
  }
  var nn = /* @__PURE__ */ new Map(), zb = /* @__PURE__ */ new Set();
  function io(e) {
    return typeof e.getRootNode == "function" ? e.getRootNode() : e.nodeType === 9 ? e : e.ownerDocument;
  }
  var $n = B.d;
  B.d = {
    f: yR,
    r: mR,
    D: bR,
    C: _R,
    L: xR,
    m: SR,
    X: wR,
    S: ER,
    M: AR
  };
  function yR() {
    var e = $n.f(), n = Fl();
    return e || n;
  }
  function mR(e) {
    var n = rr(e);
    n !== null && n.tag === 5 && n.type === "form" ? $0(n) : $n.r(e);
  }
  var Wa = typeof document > "u" ? null : document;
  function Ob(e, n, a) {
    var o = Wa;
    if (o && typeof n == "string" && n) {
      var d = Nt(n);
      d = 'link[rel="' + e + '"][href="' + d + '"]', typeof a == "string" && (d += '[crossorigin="' + a + '"]'), zb.has(d) || (zb.add(d), e = { rel: e, crossOrigin: a, href: n }, o.querySelector(d) === null && (n = o.createElement("link"), _t(n, "link", e), at(n), o.head.appendChild(n)));
    }
  }
  function bR(e) {
    $n.D(e), Ob("dns-prefetch", e, null);
  }
  function _R(e, n) {
    $n.C(e, n), Ob("preconnect", e, n);
  }
  function xR(e, n, a) {
    $n.L(e, n, a);
    var o = Wa;
    if (o && e && n) {
      var d = 'link[rel="preload"][as="' + Nt(n) + '"]';
      n === "image" && a && a.imageSrcSet ? (d += '[imagesrcset="' + Nt(
        a.imageSrcSet
      ) + '"]', typeof a.imageSizes == "string" && (d += '[imagesizes="' + Nt(
        a.imageSizes
      ) + '"]')) : d += '[href="' + Nt(e) + '"]';
      var p = d;
      switch (n) {
        case "style":
          p = ei(e);
          break;
        case "script":
          p = ti(e);
      }
      nn.has(p) || (e = y(
        {
          rel: "preload",
          href: n === "image" && a && a.imageSrcSet ? void 0 : e,
          as: n
        },
        a
      ), nn.set(p, e), o.querySelector(d) !== null || n === "style" && o.querySelector(fu(p)) || n === "script" && o.querySelector(du(p)) || (n = o.createElement("link"), _t(n, "link", e), at(n), o.head.appendChild(n)));
    }
  }
  function SR(e, n) {
    $n.m(e, n);
    var a = Wa;
    if (a && e) {
      var o = n && typeof n.as == "string" ? n.as : "script", d = 'link[rel="modulepreload"][as="' + Nt(o) + '"][href="' + Nt(e) + '"]', p = d;
      switch (o) {
        case "audioworklet":
        case "paintworklet":
        case "serviceworker":
        case "sharedworker":
        case "worker":
        case "script":
          p = ti(e);
      }
      if (!nn.has(p) && (e = y({ rel: "modulepreload", href: e }, n), nn.set(p, e), a.querySelector(d) === null)) {
        switch (o) {
          case "audioworklet":
          case "paintworklet":
          case "serviceworker":
          case "sharedworker":
          case "worker":
          case "script":
            if (a.querySelector(du(p)))
              return;
        }
        o = a.createElement("link"), _t(o, "link", e), at(o), a.head.appendChild(o);
      }
    }
  }
  function ER(e, n, a) {
    $n.S(e, n, a);
    var o = Wa;
    if (o && e) {
      var d = ir(o).hoistableStyles, p = ei(e);
      n = n || "default";
      var x = d.get(p);
      if (!x) {
        var C = { loading: 0, preload: null };
        if (x = o.querySelector(
          fu(p)
        ))
          C.loading = 5;
        else {
          e = y(
            { rel: "stylesheet", href: e, "data-precedence": n },
            a
          ), (a = nn.get(p)) && Mf(e, a);
          var V = x = o.createElement("link");
          at(V), _t(V, "link", e), V._p = new Promise(function(te, oe) {
            V.onload = te, V.onerror = oe;
          }), V.addEventListener("load", function() {
            C.loading |= 1;
          }), V.addEventListener("error", function() {
            C.loading |= 2;
          }), C.loading |= 4, uo(x, n, o);
        }
        x = {
          type: "stylesheet",
          instance: x,
          count: 1,
          state: C
        }, d.set(p, x);
      }
    }
  }
  function wR(e, n) {
    $n.X(e, n);
    var a = Wa;
    if (a && e) {
      var o = ir(a).hoistableScripts, d = ti(e), p = o.get(d);
      p || (p = a.querySelector(du(d)), p || (e = y({ src: e, async: !0 }, n), (n = nn.get(d)) && Cf(e, n), p = a.createElement("script"), at(p), _t(p, "link", e), a.head.appendChild(p)), p = {
        type: "script",
        instance: p,
        count: 1,
        state: null
      }, o.set(d, p));
    }
  }
  function AR(e, n) {
    $n.M(e, n);
    var a = Wa;
    if (a && e) {
      var o = ir(a).hoistableScripts, d = ti(e), p = o.get(d);
      p || (p = a.querySelector(du(d)), p || (e = y({ src: e, async: !0, type: "module" }, n), (n = nn.get(d)) && Cf(e, n), p = a.createElement("script"), at(p), _t(p, "link", e), a.head.appendChild(p)), p = {
        type: "script",
        instance: p,
        count: 1,
        state: null
      }, o.set(d, p));
    }
  }
  function Db(e, n, a, o) {
    var d = (d = P.current) ? io(d) : null;
    if (!d) throw Error(u(446));
    switch (e) {
      case "meta":
      case "title":
        return null;
      case "style":
        return typeof a.precedence == "string" && typeof a.href == "string" ? (n = ei(a.href), a = ir(
          d
        ).hoistableStyles, o = a.get(n), o || (o = {
          type: "style",
          instance: null,
          count: 0,
          state: null
        }, a.set(n, o)), o) : { type: "void", instance: null, count: 0, state: null };
      case "link":
        if (a.rel === "stylesheet" && typeof a.href == "string" && typeof a.precedence == "string") {
          e = ei(a.href);
          var p = ir(
            d
          ).hoistableStyles, x = p.get(e);
          if (x || (d = d.ownerDocument || d, x = {
            type: "stylesheet",
            instance: null,
            count: 0,
            state: { loading: 0, preload: null }
          }, p.set(e, x), (p = d.querySelector(
            fu(e)
          )) && !p._p && (x.instance = p, x.state.loading = 5), nn.has(e) || (a = {
            rel: "preload",
            as: "style",
            href: a.href,
            crossOrigin: a.crossOrigin,
            integrity: a.integrity,
            media: a.media,
            hrefLang: a.hrefLang,
            referrerPolicy: a.referrerPolicy
          }, nn.set(e, a), p || TR(
            d,
            e,
            a,
            x.state
          ))), n && o === null)
            throw Error(u(528, ""));
          return x;
        }
        if (n && o !== null)
          throw Error(u(529, ""));
        return null;
      case "script":
        return n = a.async, a = a.src, typeof a == "string" && n && typeof n != "function" && typeof n != "symbol" ? (n = ti(a), a = ir(
          d
        ).hoistableScripts, o = a.get(n), o || (o = {
          type: "script",
          instance: null,
          count: 0,
          state: null
        }, a.set(n, o)), o) : { type: "void", instance: null, count: 0, state: null };
      default:
        throw Error(u(444, e));
    }
  }
  function ei(e) {
    return 'href="' + Nt(e) + '"';
  }
  function fu(e) {
    return 'link[rel="stylesheet"][' + e + "]";
  }
  function Lb(e) {
    return y({}, e, {
      "data-precedence": e.precedence,
      precedence: null
    });
  }
  function TR(e, n, a, o) {
    e.querySelector('link[rel="preload"][as="style"][' + n + "]") ? o.loading = 1 : (n = e.createElement("link"), o.preload = n, n.addEventListener("load", function() {
      return o.loading |= 1;
    }), n.addEventListener("error", function() {
      return o.loading |= 2;
    }), _t(n, "link", a), at(n), e.head.appendChild(n));
  }
  function ti(e) {
    return '[src="' + Nt(e) + '"]';
  }
  function du(e) {
    return "script[async]" + e;
  }
  function Hb(e, n, a) {
    if (n.count++, n.instance === null)
      switch (n.type) {
        case "style":
          var o = e.querySelector(
            'style[data-href~="' + Nt(a.href) + '"]'
          );
          if (o)
            return n.instance = o, at(o), o;
          var d = y({}, a, {
            "data-href": a.href,
            "data-precedence": a.precedence,
            href: null,
            precedence: null
          });
          return o = (e.ownerDocument || e).createElement(
            "style"
          ), at(o), _t(o, "style", d), uo(o, a.precedence, e), n.instance = o;
        case "stylesheet":
          d = ei(a.href);
          var p = e.querySelector(
            fu(d)
          );
          if (p)
            return n.state.loading |= 4, n.instance = p, at(p), p;
          o = Lb(a), (d = nn.get(d)) && Mf(o, d), p = (e.ownerDocument || e).createElement("link"), at(p);
          var x = p;
          return x._p = new Promise(function(C, V) {
            x.onload = C, x.onerror = V;
          }), _t(p, "link", o), n.state.loading |= 4, uo(p, a.precedence, e), n.instance = p;
        case "script":
          return p = ti(a.src), (d = e.querySelector(
            du(p)
          )) ? (n.instance = d, at(d), d) : (o = a, (d = nn.get(p)) && (o = y({}, a), Cf(o, d)), e = e.ownerDocument || e, d = e.createElement("script"), at(d), _t(d, "link", o), e.head.appendChild(d), n.instance = d);
        case "void":
          return null;
        default:
          throw Error(u(443, n.type));
      }
    else
      n.type === "stylesheet" && (n.state.loading & 4) === 0 && (o = n.instance, n.state.loading |= 4, uo(o, a.precedence, e));
    return n.instance;
  }
  function uo(e, n, a) {
    for (var o = a.querySelectorAll(
      'link[rel="stylesheet"][data-precedence],style[data-precedence]'
    ), d = o.length ? o[o.length - 1] : null, p = d, x = 0; x < o.length; x++) {
      var C = o[x];
      if (C.dataset.precedence === n) p = C;
      else if (p !== d) break;
    }
    p ? p.parentNode.insertBefore(e, p.nextSibling) : (n = a.nodeType === 9 ? a.head : a, n.insertBefore(e, n.firstChild));
  }
  function Mf(e, n) {
    e.crossOrigin == null && (e.crossOrigin = n.crossOrigin), e.referrerPolicy == null && (e.referrerPolicy = n.referrerPolicy), e.title == null && (e.title = n.title);
  }
  function Cf(e, n) {
    e.crossOrigin == null && (e.crossOrigin = n.crossOrigin), e.referrerPolicy == null && (e.referrerPolicy = n.referrerPolicy), e.integrity == null && (e.integrity = n.integrity);
  }
  var lo = null;
  function jb(e, n, a) {
    if (lo === null) {
      var o = /* @__PURE__ */ new Map(), d = lo = /* @__PURE__ */ new Map();
      d.set(a, o);
    } else
      d = lo, o = d.get(a), o || (o = /* @__PURE__ */ new Map(), d.set(a, o));
    if (o.has(e)) return o;
    for (o.set(e, null), a = a.getElementsByTagName(e), d = 0; d < a.length; d++) {
      var p = a[d];
      if (!(p[Br] || p[dt] || e === "link" && p.getAttribute("rel") === "stylesheet") && p.namespaceURI !== "http://www.w3.org/2000/svg") {
        var x = p.getAttribute(n) || "";
        x = e + x;
        var C = o.get(x);
        C ? C.push(p) : o.set(x, [p]);
      }
    }
    return o;
  }
  function Bb(e, n, a) {
    e = e.ownerDocument || e, e.head.insertBefore(
      a,
      n === "title" ? e.querySelector("head > title") : null
    );
  }
  function MR(e, n, a) {
    if (a === 1 || n.itemProp != null) return !1;
    switch (e) {
      case "meta":
      case "title":
        return !0;
      case "style":
        if (typeof n.precedence != "string" || typeof n.href != "string" || n.href === "")
          break;
        return !0;
      case "link":
        if (typeof n.rel != "string" || typeof n.href != "string" || n.href === "" || n.onLoad || n.onError)
          break;
        switch (n.rel) {
          case "stylesheet":
            return e = n.disabled, typeof n.precedence == "string" && e == null;
          default:
            return !0;
        }
      case "script":
        if (n.async && typeof n.async != "function" && typeof n.async != "symbol" && !n.onLoad && !n.onError && n.src && typeof n.src == "string")
          return !0;
    }
    return !1;
  }
  function Ub(e) {
    return !(e.type === "stylesheet" && (e.state.loading & 3) === 0);
  }
  function CR(e, n, a, o) {
    if (a.type === "stylesheet" && (typeof o.media != "string" || matchMedia(o.media).matches !== !1) && (a.state.loading & 4) === 0) {
      if (a.instance === null) {
        var d = ei(o.href), p = n.querySelector(
          fu(d)
        );
        if (p) {
          n = p._p, n !== null && typeof n == "object" && typeof n.then == "function" && (e.count++, e = oo.bind(e), n.then(e, e)), a.state.loading |= 4, a.instance = p, at(p);
          return;
        }
        p = n.ownerDocument || n, o = Lb(o), (d = nn.get(d)) && Mf(o, d), p = p.createElement("link"), at(p);
        var x = p;
        x._p = new Promise(function(C, V) {
          x.onload = C, x.onerror = V;
        }), _t(p, "link", o), a.instance = p;
      }
      e.stylesheets === null && (e.stylesheets = /* @__PURE__ */ new Map()), e.stylesheets.set(a, n), (n = a.state.preload) && (a.state.loading & 3) === 0 && (e.count++, a = oo.bind(e), n.addEventListener("load", a), n.addEventListener("error", a));
    }
  }
  var qf = 0;
  function qR(e, n) {
    return e.stylesheets && e.count === 0 && co(e, e.stylesheets), 0 < e.count || 0 < e.imgCount ? function(a) {
      var o = setTimeout(function() {
        if (e.stylesheets && co(e, e.stylesheets), e.unsuspend) {
          var p = e.unsuspend;
          e.unsuspend = null, p();
        }
      }, 6e4 + n);
      0 < e.imgBytes && qf === 0 && (qf = 62500 * sR());
      var d = setTimeout(
        function() {
          if (e.waitingForImages = !1, e.count === 0 && (e.stylesheets && co(e, e.stylesheets), e.unsuspend)) {
            var p = e.unsuspend;
            e.unsuspend = null, p();
          }
        },
        (e.imgBytes > qf ? 50 : 800) + n
      );
      return e.unsuspend = a, function() {
        e.unsuspend = null, clearTimeout(o), clearTimeout(d);
      };
    } : null;
  }
  function oo() {
    if (this.count--, this.count === 0 && (this.imgCount === 0 || !this.waitingForImages)) {
      if (this.stylesheets) co(this, this.stylesheets);
      else if (this.unsuspend) {
        var e = this.unsuspend;
        this.unsuspend = null, e();
      }
    }
  }
  var so = null;
  function co(e, n) {
    e.stylesheets = null, e.unsuspend !== null && (e.count++, so = /* @__PURE__ */ new Map(), n.forEach(RR, e), so = null, oo.call(e));
  }
  function RR(e, n) {
    if (!(n.state.loading & 4)) {
      var a = so.get(e);
      if (a) var o = a.get(null);
      else {
        a = /* @__PURE__ */ new Map(), so.set(e, a);
        for (var d = e.querySelectorAll(
          "link[data-precedence],style[data-precedence]"
        ), p = 0; p < d.length; p++) {
          var x = d[p];
          (x.nodeName === "LINK" || x.getAttribute("media") !== "not all") && (a.set(x.dataset.precedence, x), o = x);
        }
        o && a.set(null, o);
      }
      d = n.instance, x = d.getAttribute("data-precedence"), p = a.get(x) || o, p === o && a.set(null, d), a.set(x, d), this.count++, o = oo.bind(this), d.addEventListener("load", o), d.addEventListener("error", o), p ? p.parentNode.insertBefore(d, p.nextSibling) : (e = e.nodeType === 9 ? e.head : e, e.insertBefore(d, e.firstChild)), n.state.loading |= 4;
    }
  }
  var hu = {
    $$typeof: S,
    Provider: null,
    Consumer: null,
    _currentValue: $,
    _currentValue2: $,
    _threadCount: 0
  };
  function NR(e, n, a, o, d, p, x, C, V) {
    this.tag = 1, this.containerInfo = e, this.pingCache = this.current = this.pendingChildren = null, this.timeoutHandle = -1, this.callbackNode = this.next = this.pendingContext = this.context = this.cancelPendingCommit = null, this.callbackPriority = 0, this.expirationTimes = xi(-1), this.entangledLanes = this.shellSuspendCounter = this.errorRecoveryDisabledLanes = this.expiredLanes = this.warmLanes = this.pingedLanes = this.suspendedLanes = this.pendingLanes = 0, this.entanglements = xi(0), this.hiddenUpdates = xi(null), this.identifierPrefix = o, this.onUncaughtError = d, this.onCaughtError = p, this.onRecoverableError = x, this.pooledCache = null, this.pooledCacheLanes = 0, this.formState = V, this.incompleteTransitions = /* @__PURE__ */ new Map();
  }
  function Gb(e, n, a, o, d, p, x, C, V, te, oe, fe) {
    return e = new NR(
      e,
      n,
      a,
      x,
      V,
      te,
      oe,
      fe,
      C
    ), n = 1, p === !0 && (n |= 24), p = Vt(3, null, null, n), e.current = p, p.stateNode = e, n = lc(), n.refCount++, e.pooledCache = n, n.refCount++, p.memoizedState = {
      element: o,
      isDehydrated: a,
      cache: n
    }, fc(p), e;
  }
  function Vb(e) {
    return e ? (e = za, e) : za;
  }
  function Yb(e, n, a, o, d, p) {
    d = Vb(d), o.context === null ? o.context = d : o.pendingContext = d, o = gr(n), o.payload = { element: a }, p = p === void 0 ? null : p, p !== null && (o.callback = p), a = vr(e, o, n), a !== null && (jt(a, e, n), Ii(a, e, n));
  }
  function kb(e, n) {
    if (e = e.memoizedState, e !== null && e.dehydrated !== null) {
      var a = e.retryLane;
      e.retryLane = a !== 0 && a < n ? a : n;
    }
  }
  function Rf(e, n) {
    kb(e, n), (e = e.alternate) && kb(e, n);
  }
  function Xb(e) {
    if (e.tag === 13 || e.tag === 31) {
      var n = kr(e, 67108864);
      n !== null && jt(n, e, 67108864), Rf(e, 67108864);
    }
  }
  function Ib(e) {
    if (e.tag === 13 || e.tag === 31) {
      var n = Zt();
      n = Si(n);
      var a = kr(e, n);
      a !== null && jt(a, e, n), Rf(e, n);
    }
  }
  var fo = !0;
  function zR(e, n, a, o) {
    var d = z.T;
    z.T = null;
    var p = B.p;
    try {
      B.p = 2, Nf(e, n, a, o);
    } finally {
      B.p = p, z.T = d;
    }
  }
  function OR(e, n, a, o) {
    var d = z.T;
    z.T = null;
    var p = B.p;
    try {
      B.p = 8, Nf(e, n, a, o);
    } finally {
      B.p = p, z.T = d;
    }
  }
  function Nf(e, n, a, o) {
    if (fo) {
      var d = zf(o);
      if (d === null)
        yf(
          e,
          n,
          o,
          ho,
          a
        ), Kb(e, o);
      else if (LR(
        d,
        e,
        n,
        a,
        o
      ))
        o.stopPropagation();
      else if (Kb(e, o), n & 4 && -1 < DR.indexOf(e)) {
        for (; d !== null; ) {
          var p = rr(d);
          if (p !== null)
            switch (p.tag) {
              case 3:
                if (p = p.stateNode, p.current.memoizedState.isDehydrated) {
                  var x = Rn(p.pendingLanes);
                  if (x !== 0) {
                    var C = p;
                    for (C.pendingLanes |= 2, C.entangledLanes |= 2; x; ) {
                      var V = 1 << 31 - Mt(x);
                      C.entanglements[1] |= V, x &= ~V;
                    }
                    En(p), (He & 6) === 0 && (Ql = ke() + 500, lu(0));
                  }
                }
                break;
              case 31:
              case 13:
                C = kr(p, 2), C !== null && jt(C, p, 2), Fl(), Rf(p, 2);
            }
          if (p = zf(o), p === null && yf(
            e,
            n,
            o,
            ho,
            a
          ), p === d) break;
          d = p;
        }
        d !== null && o.stopPropagation();
      } else
        yf(
          e,
          n,
          o,
          null,
          a
        );
    }
  }
  function zf(e) {
    return e = Os(e), Of(e);
  }
  var ho = null;
  function Of(e) {
    if (ho = null, e = nr(e), e !== null) {
      var n = s(e);
      if (n === null) e = null;
      else {
        var a = n.tag;
        if (a === 13) {
          if (e = c(n), e !== null) return e;
          e = null;
        } else if (a === 31) {
          if (e = f(n), e !== null) return e;
          e = null;
        } else if (a === 3) {
          if (n.stateNode.current.memoizedState.isDehydrated)
            return n.tag === 3 ? n.stateNode.containerInfo : null;
          e = null;
        } else n !== e && (e = null);
      }
    }
    return ho = e, null;
  }
  function Zb(e) {
    switch (e) {
      case "beforetoggle":
      case "cancel":
      case "click":
      case "close":
      case "contextmenu":
      case "copy":
      case "cut":
      case "auxclick":
      case "dblclick":
      case "dragend":
      case "dragstart":
      case "drop":
      case "focusin":
      case "focusout":
      case "input":
      case "invalid":
      case "keydown":
      case "keypress":
      case "keyup":
      case "mousedown":
      case "mouseup":
      case "paste":
      case "pause":
      case "play":
      case "pointercancel":
      case "pointerdown":
      case "pointerup":
      case "ratechange":
      case "reset":
      case "resize":
      case "seeked":
      case "submit":
      case "toggle":
      case "touchcancel":
      case "touchend":
      case "touchstart":
      case "volumechange":
      case "change":
      case "selectionchange":
      case "textInput":
      case "compositionstart":
      case "compositionend":
      case "compositionupdate":
      case "beforeblur":
      case "afterblur":
      case "beforeinput":
      case "blur":
      case "fullscreenchange":
      case "focus":
      case "hashchange":
      case "popstate":
      case "select":
      case "selectstart":
        return 2;
      case "drag":
      case "dragenter":
      case "dragexit":
      case "dragleave":
      case "dragover":
      case "mousemove":
      case "mouseout":
      case "mouseover":
      case "pointermove":
      case "pointerout":
      case "pointerover":
      case "scroll":
      case "touchmove":
      case "wheel":
      case "mouseenter":
      case "mouseleave":
      case "pointerenter":
      case "pointerleave":
        return 8;
      case "message":
        switch (vt()) {
          case qt:
            return 2;
          case Tt:
            return 8;
          case ft:
          case tr:
            return 32;
          case qn:
            return 268435456;
          default:
            return 32;
        }
      default:
        return 32;
    }
  }
  var Df = !1, Tr = null, Mr = null, Cr = null, gu = /* @__PURE__ */ new Map(), vu = /* @__PURE__ */ new Map(), qr = [], DR = "mousedown mouseup touchcancel touchend touchstart auxclick dblclick pointercancel pointerdown pointerup dragend dragstart drop compositionend compositionstart keydown keypress keyup input textInput copy cut paste click change contextmenu reset".split(
    " "
  );
  function Kb(e, n) {
    switch (e) {
      case "focusin":
      case "focusout":
        Tr = null;
        break;
      case "dragenter":
      case "dragleave":
        Mr = null;
        break;
      case "mouseover":
      case "mouseout":
        Cr = null;
        break;
      case "pointerover":
      case "pointerout":
        gu.delete(n.pointerId);
        break;
      case "gotpointercapture":
      case "lostpointercapture":
        vu.delete(n.pointerId);
    }
  }
  function pu(e, n, a, o, d, p) {
    return e === null || e.nativeEvent !== p ? (e = {
      blockedOn: n,
      domEventName: a,
      eventSystemFlags: o,
      nativeEvent: p,
      targetContainers: [d]
    }, n !== null && (n = rr(n), n !== null && Xb(n)), e) : (e.eventSystemFlags |= o, n = e.targetContainers, d !== null && n.indexOf(d) === -1 && n.push(d), e);
  }
  function LR(e, n, a, o, d) {
    switch (n) {
      case "focusin":
        return Tr = pu(
          Tr,
          e,
          n,
          a,
          o,
          d
        ), !0;
      case "dragenter":
        return Mr = pu(
          Mr,
          e,
          n,
          a,
          o,
          d
        ), !0;
      case "mouseover":
        return Cr = pu(
          Cr,
          e,
          n,
          a,
          o,
          d
        ), !0;
      case "pointerover":
        var p = d.pointerId;
        return gu.set(
          p,
          pu(
            gu.get(p) || null,
            e,
            n,
            a,
            o,
            d
          )
        ), !0;
      case "gotpointercapture":
        return p = d.pointerId, vu.set(
          p,
          pu(
            vu.get(p) || null,
            e,
            n,
            a,
            o,
            d
          )
        ), !0;
    }
    return !1;
  }
  function Qb(e) {
    var n = nr(e.target);
    if (n !== null) {
      var a = s(n);
      if (a !== null) {
        if (n = a.tag, n === 13) {
          if (n = c(a), n !== null) {
            e.blockedOn = n, tl(e.priority, function() {
              Ib(a);
            });
            return;
          }
        } else if (n === 31) {
          if (n = f(a), n !== null) {
            e.blockedOn = n, tl(e.priority, function() {
              Ib(a);
            });
            return;
          }
        } else if (n === 3 && a.stateNode.current.memoizedState.isDehydrated) {
          e.blockedOn = a.tag === 3 ? a.stateNode.containerInfo : null;
          return;
        }
      }
    }
    e.blockedOn = null;
  }
  function go(e) {
    if (e.blockedOn !== null) return !1;
    for (var n = e.targetContainers; 0 < n.length; ) {
      var a = zf(e.nativeEvent);
      if (a === null) {
        a = e.nativeEvent;
        var o = new a.constructor(
          a.type,
          a
        );
        zs = o, a.target.dispatchEvent(o), zs = null;
      } else
        return n = rr(a), n !== null && Xb(n), e.blockedOn = a, !1;
      n.shift();
    }
    return !0;
  }
  function $b(e, n, a) {
    go(e) && a.delete(n);
  }
  function HR() {
    Df = !1, Tr !== null && go(Tr) && (Tr = null), Mr !== null && go(Mr) && (Mr = null), Cr !== null && go(Cr) && (Cr = null), gu.forEach($b), vu.forEach($b);
  }
  function vo(e, n) {
    e.blockedOn === n && (e.blockedOn = null, Df || (Df = !0, t.unstable_scheduleCallback(
      t.unstable_NormalPriority,
      HR
    )));
  }
  var po = null;
  function Fb(e) {
    po !== e && (po = e, t.unstable_scheduleCallback(
      t.unstable_NormalPriority,
      function() {
        po === e && (po = null);
        for (var n = 0; n < e.length; n += 3) {
          var a = e[n], o = e[n + 1], d = e[n + 2];
          if (typeof o != "function") {
            if (Of(o || a) === null)
              continue;
            break;
          }
          var p = rr(a);
          p !== null && (e.splice(n, 3), n -= 3, Nc(
            p,
            {
              pending: !0,
              data: d,
              method: a.method,
              action: o
            },
            o,
            d
          ));
        }
      }
    ));
  }
  function ni(e) {
    function n(V) {
      return vo(V, e);
    }
    Tr !== null && vo(Tr, e), Mr !== null && vo(Mr, e), Cr !== null && vo(Cr, e), gu.forEach(n), vu.forEach(n);
    for (var a = 0; a < qr.length; a++) {
      var o = qr[a];
      o.blockedOn === e && (o.blockedOn = null);
    }
    for (; 0 < qr.length && (a = qr[0], a.blockedOn === null); )
      Qb(a), a.blockedOn === null && qr.shift();
    if (a = (e.ownerDocument || e).$$reactFormReplay, a != null)
      for (o = 0; o < a.length; o += 3) {
        var d = a[o], p = a[o + 1], x = d[wt] || null;
        if (typeof p == "function")
          x || Fb(a);
        else if (x) {
          var C = null;
          if (p && p.hasAttribute("formAction")) {
            if (d = p, x = p[wt] || null)
              C = x.formAction;
            else if (Of(d) !== null) continue;
          } else C = x.action;
          typeof C == "function" ? a[o + 1] = C : (a.splice(o, 3), o -= 3), Fb(a);
        }
      }
  }
  function Jb() {
    function e(p) {
      p.canIntercept && p.info === "react-transition" && p.intercept({
        handler: function() {
          return new Promise(function(x) {
            return d = x;
          });
        },
        focusReset: "manual",
        scroll: "manual"
      });
    }
    function n() {
      d !== null && (d(), d = null), o || setTimeout(a, 20);
    }
    function a() {
      if (!o && !navigation.transition) {
        var p = navigation.currentEntry;
        p && p.url != null && navigation.navigate(p.url, {
          state: p.getState(),
          info: "react-transition",
          history: "replace"
        });
      }
    }
    if (typeof navigation == "object") {
      var o = !1, d = null;
      return navigation.addEventListener("navigate", e), navigation.addEventListener("navigatesuccess", n), navigation.addEventListener("navigateerror", n), setTimeout(a, 100), function() {
        o = !0, navigation.removeEventListener("navigate", e), navigation.removeEventListener("navigatesuccess", n), navigation.removeEventListener("navigateerror", n), d !== null && (d(), d = null);
      };
    }
  }
  function Lf(e) {
    this._internalRoot = e;
  }
  yo.prototype.render = Lf.prototype.render = function(e) {
    var n = this._internalRoot;
    if (n === null) throw Error(u(409));
    var a = n.current, o = Zt();
    Yb(a, o, e, n, null, null);
  }, yo.prototype.unmount = Lf.prototype.unmount = function() {
    var e = this._internalRoot;
    if (e !== null) {
      this._internalRoot = null;
      var n = e.containerInfo;
      Yb(e.current, 2, null, e, null, null), Fl(), n[Nn] = null;
    }
  };
  function yo(e) {
    this._internalRoot = e;
  }
  yo.prototype.unstable_scheduleHydration = function(e) {
    if (e) {
      var n = el();
      e = { blockedOn: null, target: e, priority: n };
      for (var a = 0; a < qr.length && n !== 0 && n < qr[a].priority; a++) ;
      qr.splice(a, 0, e), a === 0 && Qb(e);
    }
  };
  var Pb = r.version;
  if (Pb !== "19.2.8")
    throw Error(
      u(
        527,
        Pb,
        "19.2.8"
      )
    );
  B.findDOMNode = function(e) {
    var n = e._reactInternals;
    if (n === void 0)
      throw typeof e.render == "function" ? Error(u(188)) : (e = Object.keys(e).join(","), Error(u(268, e)));
    return e = h(n), e = e !== null ? v(e) : null, e = e === null ? null : e.stateNode, e;
  };
  var jR = {
    bundleType: 0,
    version: "19.2.8",
    rendererPackageName: "react-dom",
    currentDispatcherRef: z,
    reconcilerVersion: "19.2.8"
  };
  if (typeof __REACT_DEVTOOLS_GLOBAL_HOOK__ < "u") {
    var mo = __REACT_DEVTOOLS_GLOBAL_HOOK__;
    if (!mo.isDisabled && mo.supportsFiber)
      try {
        Et = mo.inject(
          jR
        ), Je = mo;
      } catch {
      }
  }
  return mu.createRoot = function(e, n) {
    if (!l(e)) throw Error(u(299));
    var a = !1, o = "", d = i1, p = u1, x = l1;
    return n != null && (n.unstable_strictMode === !0 && (a = !0), n.identifierPrefix !== void 0 && (o = n.identifierPrefix), n.onUncaughtError !== void 0 && (d = n.onUncaughtError), n.onCaughtError !== void 0 && (p = n.onCaughtError), n.onRecoverableError !== void 0 && (x = n.onRecoverableError)), n = Gb(
      e,
      1,
      !1,
      null,
      null,
      a,
      o,
      null,
      d,
      p,
      x,
      Jb
    ), e[Nn] = n.current, pf(e), new Lf(n);
  }, mu.hydrateRoot = function(e, n, a) {
    if (!l(e)) throw Error(u(299));
    var o = !1, d = "", p = i1, x = u1, C = l1, V = null;
    return a != null && (a.unstable_strictMode === !0 && (o = !0), a.identifierPrefix !== void 0 && (d = a.identifierPrefix), a.onUncaughtError !== void 0 && (p = a.onUncaughtError), a.onCaughtError !== void 0 && (x = a.onCaughtError), a.onRecoverableError !== void 0 && (C = a.onRecoverableError), a.formState !== void 0 && (V = a.formState)), n = Gb(
      e,
      1,
      !0,
      n,
      a ?? null,
      o,
      d,
      V,
      p,
      x,
      C,
      Jb
    ), n.context = Vb(null), a = n.current, o = Zt(), o = Si(o), d = gr(o), d.callback = null, vr(a, d, o), a = o, n.current.lanes = a, jr(n, a), En(n), e[Nn] = n.current, pf(e), new yo(n);
  }, mu.version = "19.2.8", mu;
}
var s_;
function JR() {
  if (s_) return Uf.exports;
  s_ = 1;
  function t() {
    if (!(typeof __REACT_DEVTOOLS_GLOBAL_HOOK__ > "u" || typeof __REACT_DEVTOOLS_GLOBAL_HOOK__.checkDCE != "function"))
      try {
        __REACT_DEVTOOLS_GLOBAL_HOOK__.checkDCE(t);
      } catch (r) {
        console.error(r);
      }
  }
  return t(), Uf.exports = FR(), Uf.exports;
}
var PR = JR();
function Hy(t) {
  throw new Error('Could not dynamically require "' + t + '". Please configure the dynamicRequireTargets or/and ignoreDynamicRequires option of @rollup/plugin-commonjs appropriately for this require call to work.');
}
var kf, c_;
function WR() {
  if (c_) return kf;
  c_ = 1;
  function t() {
    this.__data__ = [], this.size = 0;
  }
  return kf = t, kf;
}
var Xf, f_;
function vi() {
  if (f_) return Xf;
  f_ = 1;
  function t(r, i) {
    return r === i || r !== r && i !== i;
  }
  return Xf = t, Xf;
}
var If, d_;
function Ko() {
  if (d_) return If;
  d_ = 1;
  var t = vi();
  function r(i, u) {
    for (var l = i.length; l--; )
      if (t(i[l][0], u))
        return l;
    return -1;
  }
  return If = r, If;
}
var Zf, h_;
function eN() {
  if (h_) return Zf;
  h_ = 1;
  var t = Ko(), r = Array.prototype, i = r.splice;
  function u(l) {
    var s = this.__data__, c = t(s, l);
    if (c < 0)
      return !1;
    var f = s.length - 1;
    return c == f ? s.pop() : i.call(s, c, 1), --this.size, !0;
  }
  return Zf = u, Zf;
}
var Kf, g_;
function tN() {
  if (g_) return Kf;
  g_ = 1;
  var t = Ko();
  function r(i) {
    var u = this.__data__, l = t(u, i);
    return l < 0 ? void 0 : u[l][1];
  }
  return Kf = r, Kf;
}
var Qf, v_;
function nN() {
  if (v_) return Qf;
  v_ = 1;
  var t = Ko();
  function r(i) {
    return t(this.__data__, i) > -1;
  }
  return Qf = r, Qf;
}
var $f, p_;
function rN() {
  if (p_) return $f;
  p_ = 1;
  var t = Ko();
  function r(i, u) {
    var l = this.__data__, s = t(l, i);
    return s < 0 ? (++this.size, l.push([i, u])) : l[s][1] = u, this;
  }
  return $f = r, $f;
}
var Ff, y_;
function Qo() {
  if (y_) return Ff;
  y_ = 1;
  var t = WR(), r = eN(), i = tN(), u = nN(), l = rN();
  function s(c) {
    var f = -1, g = c == null ? 0 : c.length;
    for (this.clear(); ++f < g; ) {
      var h = c[f];
      this.set(h[0], h[1]);
    }
  }
  return s.prototype.clear = t, s.prototype.delete = r, s.prototype.get = i, s.prototype.has = u, s.prototype.set = l, Ff = s, Ff;
}
var Jf, m_;
function aN() {
  if (m_) return Jf;
  m_ = 1;
  var t = Qo();
  function r() {
    this.__data__ = new t(), this.size = 0;
  }
  return Jf = r, Jf;
}
var Pf, b_;
function iN() {
  if (b_) return Pf;
  b_ = 1;
  function t(r) {
    var i = this.__data__, u = i.delete(r);
    return this.size = i.size, u;
  }
  return Pf = t, Pf;
}
var Wf, __;
function uN() {
  if (__) return Wf;
  __ = 1;
  function t(r) {
    return this.__data__.get(r);
  }
  return Wf = t, Wf;
}
var ed, x_;
function lN() {
  if (x_) return ed;
  x_ = 1;
  function t(r) {
    return this.__data__.has(r);
  }
  return ed = t, ed;
}
var td, S_;
function XA() {
  if (S_) return td;
  S_ = 1;
  var t = typeof bo == "object" && bo && bo.Object === Object && bo;
  return td = t, td;
}
var nd, E_;
function pn() {
  if (E_) return nd;
  E_ = 1;
  var t = XA(), r = typeof self == "object" && self && self.Object === Object && self, i = t || r || Function("return this")();
  return nd = i, nd;
}
var rd, w_;
function pi() {
  if (w_) return rd;
  w_ = 1;
  var t = pn(), r = t.Symbol;
  return rd = r, rd;
}
var ad, A_;
function oN() {
  if (A_) return ad;
  A_ = 1;
  var t = pi(), r = Object.prototype, i = r.hasOwnProperty, u = r.toString, l = t ? t.toStringTag : void 0;
  function s(c) {
    var f = i.call(c, l), g = c[l];
    try {
      c[l] = void 0;
      var h = !0;
    } catch {
    }
    var v = u.call(c);
    return h && (f ? c[l] = g : delete c[l]), v;
  }
  return ad = s, ad;
}
var id, T_;
function sN() {
  if (T_) return id;
  T_ = 1;
  var t = Object.prototype, r = t.toString;
  function i(u) {
    return r.call(u);
  }
  return id = i, id;
}
var ud, M_;
function fa() {
  if (M_) return ud;
  M_ = 1;
  var t = pi(), r = oN(), i = sN(), u = "[object Null]", l = "[object Undefined]", s = t ? t.toStringTag : void 0;
  function c(f) {
    return f == null ? f === void 0 ? l : u : s && s in Object(f) ? r(f) : i(f);
  }
  return ud = c, ud;
}
var ld, C_;
function rn() {
  if (C_) return ld;
  C_ = 1;
  function t(r) {
    var i = typeof r;
    return r != null && (i == "object" || i == "function");
  }
  return ld = t, ld;
}
var od, q_;
function Bu() {
  if (q_) return od;
  q_ = 1;
  var t = fa(), r = rn(), i = "[object AsyncFunction]", u = "[object Function]", l = "[object GeneratorFunction]", s = "[object Proxy]";
  function c(f) {
    if (!r(f))
      return !1;
    var g = t(f);
    return g == u || g == l || g == i || g == s;
  }
  return od = c, od;
}
var sd, R_;
function cN() {
  if (R_) return sd;
  R_ = 1;
  var t = pn(), r = t["__core-js_shared__"];
  return sd = r, sd;
}
var cd, N_;
function fN() {
  if (N_) return cd;
  N_ = 1;
  var t = cN(), r = (function() {
    var u = /[^.]+$/.exec(t && t.keys && t.keys.IE_PROTO || "");
    return u ? "Symbol(src)_1." + u : "";
  })();
  function i(u) {
    return !!r && r in u;
  }
  return cd = i, cd;
}
var fd, z_;
function IA() {
  if (z_) return fd;
  z_ = 1;
  var t = Function.prototype, r = t.toString;
  function i(u) {
    if (u != null) {
      try {
        return r.call(u);
      } catch {
      }
      try {
        return u + "";
      } catch {
      }
    }
    return "";
  }
  return fd = i, fd;
}
var dd, O_;
function dN() {
  if (O_) return dd;
  O_ = 1;
  var t = Bu(), r = fN(), i = rn(), u = IA(), l = /[\\^$.*+?()[\]{}|]/g, s = /^\[object .+?Constructor\]$/, c = Function.prototype, f = Object.prototype, g = c.toString, h = f.hasOwnProperty, v = RegExp(
    "^" + g.call(h).replace(l, "\\$&").replace(/hasOwnProperty|(function).*?(?=\\\()| for .+?(?=\\\])/g, "$1.*?") + "$"
  );
  function y(m) {
    if (!i(m) || r(m))
      return !1;
    var b = t(m) ? v : s;
    return b.test(u(m));
  }
  return dd = y, dd;
}
var hd, D_;
function hN() {
  if (D_) return hd;
  D_ = 1;
  function t(r, i) {
    return r == null ? void 0 : r[i];
  }
  return hd = t, hd;
}
var gd, L_;
function da() {
  if (L_) return gd;
  L_ = 1;
  var t = dN(), r = hN();
  function i(u, l) {
    var s = r(u, l);
    return t(s) ? s : void 0;
  }
  return gd = i, gd;
}
var vd, H_;
function jy() {
  if (H_) return vd;
  H_ = 1;
  var t = da(), r = pn(), i = t(r, "Map");
  return vd = i, vd;
}
var pd, j_;
function $o() {
  if (j_) return pd;
  j_ = 1;
  var t = da(), r = t(Object, "create");
  return pd = r, pd;
}
var yd, B_;
function gN() {
  if (B_) return yd;
  B_ = 1;
  var t = $o();
  function r() {
    this.__data__ = t ? t(null) : {}, this.size = 0;
  }
  return yd = r, yd;
}
var md, U_;
function vN() {
  if (U_) return md;
  U_ = 1;
  function t(r) {
    var i = this.has(r) && delete this.__data__[r];
    return this.size -= i ? 1 : 0, i;
  }
  return md = t, md;
}
var bd, G_;
function pN() {
  if (G_) return bd;
  G_ = 1;
  var t = $o(), r = "__lodash_hash_undefined__", i = Object.prototype, u = i.hasOwnProperty;
  function l(s) {
    var c = this.__data__;
    if (t) {
      var f = c[s];
      return f === r ? void 0 : f;
    }
    return u.call(c, s) ? c[s] : void 0;
  }
  return bd = l, bd;
}
var _d, V_;
function yN() {
  if (V_) return _d;
  V_ = 1;
  var t = $o(), r = Object.prototype, i = r.hasOwnProperty;
  function u(l) {
    var s = this.__data__;
    return t ? s[l] !== void 0 : i.call(s, l);
  }
  return _d = u, _d;
}
var xd, Y_;
function mN() {
  if (Y_) return xd;
  Y_ = 1;
  var t = $o(), r = "__lodash_hash_undefined__";
  function i(u, l) {
    var s = this.__data__;
    return this.size += this.has(u) ? 0 : 1, s[u] = t && l === void 0 ? r : l, this;
  }
  return xd = i, xd;
}
var Sd, k_;
function bN() {
  if (k_) return Sd;
  k_ = 1;
  var t = gN(), r = vN(), i = pN(), u = yN(), l = mN();
  function s(c) {
    var f = -1, g = c == null ? 0 : c.length;
    for (this.clear(); ++f < g; ) {
      var h = c[f];
      this.set(h[0], h[1]);
    }
  }
  return s.prototype.clear = t, s.prototype.delete = r, s.prototype.get = i, s.prototype.has = u, s.prototype.set = l, Sd = s, Sd;
}
var Ed, X_;
function _N() {
  if (X_) return Ed;
  X_ = 1;
  var t = bN(), r = Qo(), i = jy();
  function u() {
    this.size = 0, this.__data__ = {
      hash: new t(),
      map: new (i || r)(),
      string: new t()
    };
  }
  return Ed = u, Ed;
}
var wd, I_;
function xN() {
  if (I_) return wd;
  I_ = 1;
  function t(r) {
    var i = typeof r;
    return i == "string" || i == "number" || i == "symbol" || i == "boolean" ? r !== "__proto__" : r === null;
  }
  return wd = t, wd;
}
var Ad, Z_;
function Fo() {
  if (Z_) return Ad;
  Z_ = 1;
  var t = xN();
  function r(i, u) {
    var l = i.__data__;
    return t(u) ? l[typeof u == "string" ? "string" : "hash"] : l.map;
  }
  return Ad = r, Ad;
}
var Td, K_;
function SN() {
  if (K_) return Td;
  K_ = 1;
  var t = Fo();
  function r(i) {
    var u = t(this, i).delete(i);
    return this.size -= u ? 1 : 0, u;
  }
  return Td = r, Td;
}
var Md, Q_;
function EN() {
  if (Q_) return Md;
  Q_ = 1;
  var t = Fo();
  function r(i) {
    return t(this, i).get(i);
  }
  return Md = r, Md;
}
var Cd, $_;
function wN() {
  if ($_) return Cd;
  $_ = 1;
  var t = Fo();
  function r(i) {
    return t(this, i).has(i);
  }
  return Cd = r, Cd;
}
var qd, F_;
function AN() {
  if (F_) return qd;
  F_ = 1;
  var t = Fo();
  function r(i, u) {
    var l = t(this, i), s = l.size;
    return l.set(i, u), this.size += l.size == s ? 0 : 1, this;
  }
  return qd = r, qd;
}
var Rd, J_;
function By() {
  if (J_) return Rd;
  J_ = 1;
  var t = _N(), r = SN(), i = EN(), u = wN(), l = AN();
  function s(c) {
    var f = -1, g = c == null ? 0 : c.length;
    for (this.clear(); ++f < g; ) {
      var h = c[f];
      this.set(h[0], h[1]);
    }
  }
  return s.prototype.clear = t, s.prototype.delete = r, s.prototype.get = i, s.prototype.has = u, s.prototype.set = l, Rd = s, Rd;
}
var Nd, P_;
function TN() {
  if (P_) return Nd;
  P_ = 1;
  var t = Qo(), r = jy(), i = By(), u = 200;
  function l(s, c) {
    var f = this.__data__;
    if (f instanceof t) {
      var g = f.__data__;
      if (!r || g.length < u - 1)
        return g.push([s, c]), this.size = ++f.size, this;
      f = this.__data__ = new i(g);
    }
    return f.set(s, c), this.size = f.size, this;
  }
  return Nd = l, Nd;
}
var zd, W_;
function Jo() {
  if (W_) return zd;
  W_ = 1;
  var t = Qo(), r = aN(), i = iN(), u = uN(), l = lN(), s = TN();
  function c(f) {
    var g = this.__data__ = new t(f);
    this.size = g.size;
  }
  return c.prototype.clear = r, c.prototype.delete = i, c.prototype.get = u, c.prototype.has = l, c.prototype.set = s, zd = c, zd;
}
var Od, ex;
function Uy() {
  if (ex) return Od;
  ex = 1;
  function t(r, i) {
    for (var u = -1, l = r == null ? 0 : r.length; ++u < l && i(r[u], u, r) !== !1; )
      ;
    return r;
  }
  return Od = t, Od;
}
var Dd, tx;
function ZA() {
  if (tx) return Dd;
  tx = 1;
  var t = da(), r = (function() {
    try {
      var i = t(Object, "defineProperty");
      return i({}, "", {}), i;
    } catch {
    }
  })();
  return Dd = r, Dd;
}
var Ld, nx;
function Po() {
  if (nx) return Ld;
  nx = 1;
  var t = ZA();
  function r(i, u, l) {
    u == "__proto__" && t ? t(i, u, {
      configurable: !0,
      enumerable: !0,
      value: l,
      writable: !0
    }) : i[u] = l;
  }
  return Ld = r, Ld;
}
var Hd, rx;
function Wo() {
  if (rx) return Hd;
  rx = 1;
  var t = Po(), r = vi(), i = Object.prototype, u = i.hasOwnProperty;
  function l(s, c, f) {
    var g = s[c];
    (!(u.call(s, c) && r(g, f)) || f === void 0 && !(c in s)) && t(s, c, f);
  }
  return Hd = l, Hd;
}
var jd, ax;
function Uu() {
  if (ax) return jd;
  ax = 1;
  var t = Wo(), r = Po();
  function i(u, l, s, c) {
    var f = !s;
    s || (s = {});
    for (var g = -1, h = l.length; ++g < h; ) {
      var v = l[g], y = c ? c(s[v], u[v], v, s, u) : void 0;
      y === void 0 && (y = u[v]), f ? r(s, v, y) : t(s, v, y);
    }
    return s;
  }
  return jd = i, jd;
}
var Bd, ix;
function MN() {
  if (ix) return Bd;
  ix = 1;
  function t(r, i) {
    for (var u = -1, l = Array(r); ++u < r; )
      l[u] = i(u);
    return l;
  }
  return Bd = t, Bd;
}
var Ud, ux;
function Tn() {
  if (ux) return Ud;
  ux = 1;
  function t(r) {
    return r != null && typeof r == "object";
  }
  return Ud = t, Ud;
}
var Gd, lx;
function CN() {
  if (lx) return Gd;
  lx = 1;
  var t = fa(), r = Tn(), i = "[object Arguments]";
  function u(l) {
    return r(l) && t(l) == i;
  }
  return Gd = u, Gd;
}
var Vd, ox;
function Gu() {
  if (ox) return Vd;
  ox = 1;
  var t = CN(), r = Tn(), i = Object.prototype, u = i.hasOwnProperty, l = i.propertyIsEnumerable, s = t(/* @__PURE__ */ (function() {
    return arguments;
  })()) ? t : function(c) {
    return r(c) && u.call(c, "callee") && !l.call(c, "callee");
  };
  return Vd = s, Vd;
}
var Yd, sx;
function st() {
  if (sx) return Yd;
  sx = 1;
  var t = Array.isArray;
  return Yd = t, Yd;
}
var xu = { exports: {} }, kd, cx;
function qN() {
  if (cx) return kd;
  cx = 1;
  function t() {
    return !1;
  }
  return kd = t, kd;
}
xu.exports;
var fx;
function yi() {
  return fx || (fx = 1, (function(t, r) {
    var i = pn(), u = qN(), l = r && !r.nodeType && r, s = l && !0 && t && !t.nodeType && t, c = s && s.exports === l, f = c ? i.Buffer : void 0, g = f ? f.isBuffer : void 0, h = g || u;
    t.exports = h;
  })(xu, xu.exports)), xu.exports;
}
var Xd, dx;
function es() {
  if (dx) return Xd;
  dx = 1;
  var t = 9007199254740991, r = /^(?:0|[1-9]\d*)$/;
  function i(u, l) {
    var s = typeof u;
    return l = l ?? t, !!l && (s == "number" || s != "symbol" && r.test(u)) && u > -1 && u % 1 == 0 && u < l;
  }
  return Xd = i, Xd;
}
var Id, hx;
function Gy() {
  if (hx) return Id;
  hx = 1;
  var t = 9007199254740991;
  function r(i) {
    return typeof i == "number" && i > -1 && i % 1 == 0 && i <= t;
  }
  return Id = r, Id;
}
var Zd, gx;
function RN() {
  if (gx) return Zd;
  gx = 1;
  var t = fa(), r = Gy(), i = Tn(), u = "[object Arguments]", l = "[object Array]", s = "[object Boolean]", c = "[object Date]", f = "[object Error]", g = "[object Function]", h = "[object Map]", v = "[object Number]", y = "[object Object]", m = "[object RegExp]", b = "[object Set]", _ = "[object String]", A = "[object WeakMap]", w = "[object ArrayBuffer]", E = "[object DataView]", M = "[object Float32Array]", S = "[object Float64Array]", T = "[object Int8Array]", N = "[object Int16Array]", R = "[object Int32Array]", q = "[object Uint8Array]", L = "[object Uint8ClampedArray]", j = "[object Uint16Array]", Z = "[object Uint32Array]", Y = {};
  Y[M] = Y[S] = Y[T] = Y[N] = Y[R] = Y[q] = Y[L] = Y[j] = Y[Z] = !0, Y[u] = Y[l] = Y[w] = Y[s] = Y[E] = Y[c] = Y[f] = Y[g] = Y[h] = Y[v] = Y[y] = Y[m] = Y[b] = Y[_] = Y[A] = !1;
  function J(Q) {
    return i(Q) && r(Q.length) && !!Y[t(Q)];
  }
  return Zd = J, Zd;
}
var Kd, vx;
function ts() {
  if (vx) return Kd;
  vx = 1;
  function t(r) {
    return function(i) {
      return r(i);
    };
  }
  return Kd = t, Kd;
}
var Su = { exports: {} };
Su.exports;
var px;
function Vy() {
  return px || (px = 1, (function(t, r) {
    var i = XA(), u = r && !r.nodeType && r, l = u && !0 && t && !t.nodeType && t, s = l && l.exports === u, c = s && i.process, f = (function() {
      try {
        var g = l && l.require && l.require("util").types;
        return g || c && c.binding && c.binding("util");
      } catch {
      }
    })();
    t.exports = f;
  })(Su, Su.exports)), Su.exports;
}
var Qd, yx;
function Vu() {
  if (yx) return Qd;
  yx = 1;
  var t = RN(), r = ts(), i = Vy(), u = i && i.isTypedArray, l = u ? r(u) : t;
  return Qd = l, Qd;
}
var $d, mx;
function KA() {
  if (mx) return $d;
  mx = 1;
  var t = MN(), r = Gu(), i = st(), u = yi(), l = es(), s = Vu(), c = Object.prototype, f = c.hasOwnProperty;
  function g(h, v) {
    var y = i(h), m = !y && r(h), b = !y && !m && u(h), _ = !y && !m && !b && s(h), A = y || m || b || _, w = A ? t(h.length, String) : [], E = w.length;
    for (var M in h)
      (v || f.call(h, M)) && !(A && // Safari 9 has enumerable `arguments.length` in strict mode.
      (M == "length" || // Node.js 0.10 has enumerable non-index properties on buffers.
      b && (M == "offset" || M == "parent") || // PhantomJS 2 has enumerable non-index properties on typed arrays.
      _ && (M == "buffer" || M == "byteLength" || M == "byteOffset") || // Skip index properties.
      l(M, E))) && w.push(M);
    return w;
  }
  return $d = g, $d;
}
var Fd, bx;
function ns() {
  if (bx) return Fd;
  bx = 1;
  var t = Object.prototype;
  function r(i) {
    var u = i && i.constructor, l = typeof u == "function" && u.prototype || t;
    return i === l;
  }
  return Fd = r, Fd;
}
var Jd, _x;
function QA() {
  if (_x) return Jd;
  _x = 1;
  function t(r, i) {
    return function(u) {
      return r(i(u));
    };
  }
  return Jd = t, Jd;
}
var Pd, xx;
function NN() {
  if (xx) return Pd;
  xx = 1;
  var t = QA(), r = t(Object.keys, Object);
  return Pd = r, Pd;
}
var Wd, Sx;
function Yy() {
  if (Sx) return Wd;
  Sx = 1;
  var t = ns(), r = NN(), i = Object.prototype, u = i.hasOwnProperty;
  function l(s) {
    if (!t(s))
      return r(s);
    var c = [];
    for (var f in Object(s))
      u.call(s, f) && f != "constructor" && c.push(f);
    return c;
  }
  return Wd = l, Wd;
}
var eh, Ex;
function Wn() {
  if (Ex) return eh;
  Ex = 1;
  var t = Bu(), r = Gy();
  function i(u) {
    return u != null && r(u.length) && !t(u);
  }
  return eh = i, eh;
}
var th, wx;
function Lr() {
  if (wx) return th;
  wx = 1;
  var t = KA(), r = Yy(), i = Wn();
  function u(l) {
    return i(l) ? t(l) : r(l);
  }
  return th = u, th;
}
var nh, Ax;
function zN() {
  if (Ax) return nh;
  Ax = 1;
  var t = Uu(), r = Lr();
  function i(u, l) {
    return u && t(l, r(l), u);
  }
  return nh = i, nh;
}
var rh, Tx;
function ON() {
  if (Tx) return rh;
  Tx = 1;
  function t(r) {
    var i = [];
    if (r != null)
      for (var u in Object(r))
        i.push(u);
    return i;
  }
  return rh = t, rh;
}
var ah, Mx;
function DN() {
  if (Mx) return ah;
  Mx = 1;
  var t = rn(), r = ns(), i = ON(), u = Object.prototype, l = u.hasOwnProperty;
  function s(c) {
    if (!t(c))
      return i(c);
    var f = r(c), g = [];
    for (var h in c)
      h == "constructor" && (f || !l.call(c, h)) || g.push(h);
    return g;
  }
  return ah = s, ah;
}
var ih, Cx;
function ha() {
  if (Cx) return ih;
  Cx = 1;
  var t = KA(), r = DN(), i = Wn();
  function u(l) {
    return i(l) ? t(l, !0) : r(l);
  }
  return ih = u, ih;
}
var uh, qx;
function LN() {
  if (qx) return uh;
  qx = 1;
  var t = Uu(), r = ha();
  function i(u, l) {
    return u && t(l, r(l), u);
  }
  return uh = i, uh;
}
var Eu = { exports: {} };
Eu.exports;
var Rx;
function $A() {
  return Rx || (Rx = 1, (function(t, r) {
    var i = pn(), u = r && !r.nodeType && r, l = u && !0 && t && !t.nodeType && t, s = l && l.exports === u, c = s ? i.Buffer : void 0, f = c ? c.allocUnsafe : void 0;
    function g(h, v) {
      if (v)
        return h.slice();
      var y = h.length, m = f ? f(y) : new h.constructor(y);
      return h.copy(m), m;
    }
    t.exports = g;
  })(Eu, Eu.exports)), Eu.exports;
}
var lh, Nx;
function FA() {
  if (Nx) return lh;
  Nx = 1;
  function t(r, i) {
    var u = -1, l = r.length;
    for (i || (i = Array(l)); ++u < l; )
      i[u] = r[u];
    return i;
  }
  return lh = t, lh;
}
var oh, zx;
function JA() {
  if (zx) return oh;
  zx = 1;
  function t(r, i) {
    for (var u = -1, l = r == null ? 0 : r.length, s = 0, c = []; ++u < l; ) {
      var f = r[u];
      i(f, u, r) && (c[s++] = f);
    }
    return c;
  }
  return oh = t, oh;
}
var sh, Ox;
function PA() {
  if (Ox) return sh;
  Ox = 1;
  function t() {
    return [];
  }
  return sh = t, sh;
}
var ch, Dx;
function ky() {
  if (Dx) return ch;
  Dx = 1;
  var t = JA(), r = PA(), i = Object.prototype, u = i.propertyIsEnumerable, l = Object.getOwnPropertySymbols, s = l ? function(c) {
    return c == null ? [] : (c = Object(c), t(l(c), function(f) {
      return u.call(c, f);
    }));
  } : r;
  return ch = s, ch;
}
var fh, Lx;
function HN() {
  if (Lx) return fh;
  Lx = 1;
  var t = Uu(), r = ky();
  function i(u, l) {
    return t(u, r(u), l);
  }
  return fh = i, fh;
}
var dh, Hx;
function Xy() {
  if (Hx) return dh;
  Hx = 1;
  function t(r, i) {
    for (var u = -1, l = i.length, s = r.length; ++u < l; )
      r[s + u] = i[u];
    return r;
  }
  return dh = t, dh;
}
var hh, jx;
function rs() {
  if (jx) return hh;
  jx = 1;
  var t = QA(), r = t(Object.getPrototypeOf, Object);
  return hh = r, hh;
}
var gh, Bx;
function WA() {
  if (Bx) return gh;
  Bx = 1;
  var t = Xy(), r = rs(), i = ky(), u = PA(), l = Object.getOwnPropertySymbols, s = l ? function(c) {
    for (var f = []; c; )
      t(f, i(c)), c = r(c);
    return f;
  } : u;
  return gh = s, gh;
}
var vh, Ux;
function jN() {
  if (Ux) return vh;
  Ux = 1;
  var t = Uu(), r = WA();
  function i(u, l) {
    return t(u, r(u), l);
  }
  return vh = i, vh;
}
var ph, Gx;
function eT() {
  if (Gx) return ph;
  Gx = 1;
  var t = Xy(), r = st();
  function i(u, l, s) {
    var c = l(u);
    return r(u) ? c : t(c, s(u));
  }
  return ph = i, ph;
}
var yh, Vx;
function tT() {
  if (Vx) return yh;
  Vx = 1;
  var t = eT(), r = ky(), i = Lr();
  function u(l) {
    return t(l, i, r);
  }
  return yh = u, yh;
}
var mh, Yx;
function BN() {
  if (Yx) return mh;
  Yx = 1;
  var t = eT(), r = WA(), i = ha();
  function u(l) {
    return t(l, i, r);
  }
  return mh = u, mh;
}
var bh, kx;
function UN() {
  if (kx) return bh;
  kx = 1;
  var t = da(), r = pn(), i = t(r, "DataView");
  return bh = i, bh;
}
var _h, Xx;
function GN() {
  if (Xx) return _h;
  Xx = 1;
  var t = da(), r = pn(), i = t(r, "Promise");
  return _h = i, _h;
}
var xh, Ix;
function nT() {
  if (Ix) return xh;
  Ix = 1;
  var t = da(), r = pn(), i = t(r, "Set");
  return xh = i, xh;
}
var Sh, Zx;
function VN() {
  if (Zx) return Sh;
  Zx = 1;
  var t = da(), r = pn(), i = t(r, "WeakMap");
  return Sh = i, Sh;
}
var Eh, Kx;
function mi() {
  if (Kx) return Eh;
  Kx = 1;
  var t = UN(), r = jy(), i = GN(), u = nT(), l = VN(), s = fa(), c = IA(), f = "[object Map]", g = "[object Object]", h = "[object Promise]", v = "[object Set]", y = "[object WeakMap]", m = "[object DataView]", b = c(t), _ = c(r), A = c(i), w = c(u), E = c(l), M = s;
  return (t && M(new t(new ArrayBuffer(1))) != m || r && M(new r()) != f || i && M(i.resolve()) != h || u && M(new u()) != v || l && M(new l()) != y) && (M = function(S) {
    var T = s(S), N = T == g ? S.constructor : void 0, R = N ? c(N) : "";
    if (R)
      switch (R) {
        case b:
          return m;
        case _:
          return f;
        case A:
          return h;
        case w:
          return v;
        case E:
          return y;
      }
    return T;
  }), Eh = M, Eh;
}
var wh, Qx;
function YN() {
  if (Qx) return wh;
  Qx = 1;
  var t = Object.prototype, r = t.hasOwnProperty;
  function i(u) {
    var l = u.length, s = new u.constructor(l);
    return l && typeof u[0] == "string" && r.call(u, "index") && (s.index = u.index, s.input = u.input), s;
  }
  return wh = i, wh;
}
var Ah, $x;
function rT() {
  if ($x) return Ah;
  $x = 1;
  var t = pn(), r = t.Uint8Array;
  return Ah = r, Ah;
}
var Th, Fx;
function Iy() {
  if (Fx) return Th;
  Fx = 1;
  var t = rT();
  function r(i) {
    var u = new i.constructor(i.byteLength);
    return new t(u).set(new t(i)), u;
  }
  return Th = r, Th;
}
var Mh, Jx;
function kN() {
  if (Jx) return Mh;
  Jx = 1;
  var t = Iy();
  function r(i, u) {
    var l = u ? t(i.buffer) : i.buffer;
    return new i.constructor(l, i.byteOffset, i.byteLength);
  }
  return Mh = r, Mh;
}
var Ch, Px;
function XN() {
  if (Px) return Ch;
  Px = 1;
  var t = /\w*$/;
  function r(i) {
    var u = new i.constructor(i.source, t.exec(i));
    return u.lastIndex = i.lastIndex, u;
  }
  return Ch = r, Ch;
}
var qh, Wx;
function IN() {
  if (Wx) return qh;
  Wx = 1;
  var t = pi(), r = t ? t.prototype : void 0, i = r ? r.valueOf : void 0;
  function u(l) {
    return i ? Object(i.call(l)) : {};
  }
  return qh = u, qh;
}
var Rh, eS;
function aT() {
  if (eS) return Rh;
  eS = 1;
  var t = Iy();
  function r(i, u) {
    var l = u ? t(i.buffer) : i.buffer;
    return new i.constructor(l, i.byteOffset, i.length);
  }
  return Rh = r, Rh;
}
var Nh, tS;
function ZN() {
  if (tS) return Nh;
  tS = 1;
  var t = Iy(), r = kN(), i = XN(), u = IN(), l = aT(), s = "[object Boolean]", c = "[object Date]", f = "[object Map]", g = "[object Number]", h = "[object RegExp]", v = "[object Set]", y = "[object String]", m = "[object Symbol]", b = "[object ArrayBuffer]", _ = "[object DataView]", A = "[object Float32Array]", w = "[object Float64Array]", E = "[object Int8Array]", M = "[object Int16Array]", S = "[object Int32Array]", T = "[object Uint8Array]", N = "[object Uint8ClampedArray]", R = "[object Uint16Array]", q = "[object Uint32Array]";
  function L(j, Z, Y) {
    var J = j.constructor;
    switch (Z) {
      case b:
        return t(j);
      case s:
      case c:
        return new J(+j);
      case _:
        return r(j, Y);
      case A:
      case w:
      case E:
      case M:
      case S:
      case T:
      case N:
      case R:
      case q:
        return l(j, Y);
      case f:
        return new J();
      case g:
      case y:
        return new J(j);
      case h:
        return i(j);
      case v:
        return new J();
      case m:
        return u(j);
    }
  }
  return Nh = L, Nh;
}
var zh, nS;
function iT() {
  if (nS) return zh;
  nS = 1;
  var t = rn(), r = Object.create, i = /* @__PURE__ */ (function() {
    function u() {
    }
    return function(l) {
      if (!t(l))
        return {};
      if (r)
        return r(l);
      u.prototype = l;
      var s = new u();
      return u.prototype = void 0, s;
    };
  })();
  return zh = i, zh;
}
var Oh, rS;
function uT() {
  if (rS) return Oh;
  rS = 1;
  var t = iT(), r = rs(), i = ns();
  function u(l) {
    return typeof l.constructor == "function" && !i(l) ? t(r(l)) : {};
  }
  return Oh = u, Oh;
}
var Dh, aS;
function KN() {
  if (aS) return Dh;
  aS = 1;
  var t = mi(), r = Tn(), i = "[object Map]";
  function u(l) {
    return r(l) && t(l) == i;
  }
  return Dh = u, Dh;
}
var Lh, iS;
function QN() {
  if (iS) return Lh;
  iS = 1;
  var t = KN(), r = ts(), i = Vy(), u = i && i.isMap, l = u ? r(u) : t;
  return Lh = l, Lh;
}
var Hh, uS;
function $N() {
  if (uS) return Hh;
  uS = 1;
  var t = mi(), r = Tn(), i = "[object Set]";
  function u(l) {
    return r(l) && t(l) == i;
  }
  return Hh = u, Hh;
}
var jh, lS;
function FN() {
  if (lS) return jh;
  lS = 1;
  var t = $N(), r = ts(), i = Vy(), u = i && i.isSet, l = u ? r(u) : t;
  return jh = l, jh;
}
var Bh, oS;
function lT() {
  if (oS) return Bh;
  oS = 1;
  var t = Jo(), r = Uy(), i = Wo(), u = zN(), l = LN(), s = $A(), c = FA(), f = HN(), g = jN(), h = tT(), v = BN(), y = mi(), m = YN(), b = ZN(), _ = uT(), A = st(), w = yi(), E = QN(), M = rn(), S = FN(), T = Lr(), N = ha(), R = 1, q = 2, L = 4, j = "[object Arguments]", Z = "[object Array]", Y = "[object Boolean]", J = "[object Date]", Q = "[object Error]", D = "[object Function]", X = "[object GeneratorFunction]", z = "[object Map]", B = "[object Number]", $ = "[object Object]", U = "[object RegExp]", ae = "[object Set]", O = "[object String]", G = "[object Symbol]", ie = "[object WeakMap]", H = "[object ArrayBuffer]", K = "[object DataView]", P = "[object Float32Array]", ue = "[object Float64Array]", W = "[object Int8Array]", se = "[object Int16Array]", de = "[object Int32Array]", ve = "[object Uint8Array]", ye = "[object Uint8ClampedArray]", he = "[object Uint16Array]", me = "[object Uint32Array]", ge = {};
  ge[j] = ge[Z] = ge[H] = ge[K] = ge[Y] = ge[J] = ge[P] = ge[ue] = ge[W] = ge[se] = ge[de] = ge[z] = ge[B] = ge[$] = ge[U] = ge[ae] = ge[O] = ge[G] = ge[ve] = ge[ye] = ge[he] = ge[me] = !0, ge[Q] = ge[D] = ge[ie] = !1;
  function Ae(xe, Pe, tt, xt, gt, St) {
    var Ke, ke = Pe & R, vt = Pe & q, qt = Pe & L;
    if (tt && (Ke = gt ? tt(xe, xt, gt, St) : tt(xe)), Ke !== void 0)
      return Ke;
    if (!M(xe))
      return xe;
    var Tt = A(xe);
    if (Tt) {
      if (Ke = m(xe), !ke)
        return c(xe, Ke);
    } else {
      var ft = y(xe), tr = ft == D || ft == X;
      if (w(xe))
        return s(xe, ke);
      if (ft == $ || ft == j || tr && !gt) {
        if (Ke = vt || tr ? {} : _(xe), !ke)
          return vt ? g(xe, l(Ke, xe)) : f(xe, u(Ke, xe));
      } else {
        if (!ge[ft])
          return gt ? xe : {};
        Ke = b(xe, ft, ke);
      }
    }
    St || (St = new t());
    var qn = St.get(xe);
    if (qn)
      return qn;
    St.set(xe, Ke), S(xe) ? xe.forEach(function(Et) {
      Ke.add(Ae(Et, Pe, tt, Et, xe, St));
    }) : E(xe) && xe.forEach(function(Et, Je) {
      Ke.set(Je, Ae(Et, Pe, tt, Je, xe, St));
    });
    var _i = qt ? vt ? v : h : vt ? N : T, va = Tt ? void 0 : _i(xe);
    return r(va || xe, function(Et, Je) {
      va && (Je = Et, Et = xe[Je]), i(Ke, Je, Ae(Et, Pe, tt, Je, xe, St));
    }), Ke;
  }
  return Bh = Ae, Bh;
}
var Uh, sS;
function JN() {
  if (sS) return Uh;
  sS = 1;
  var t = lT(), r = 4;
  function i(u) {
    return t(u, r);
  }
  return Uh = i, Uh;
}
var Gh, cS;
function Zy() {
  if (cS) return Gh;
  cS = 1;
  function t(r) {
    return function() {
      return r;
    };
  }
  return Gh = t, Gh;
}
var Vh, fS;
function PN() {
  if (fS) return Vh;
  fS = 1;
  function t(r) {
    return function(i, u, l) {
      for (var s = -1, c = Object(i), f = l(i), g = f.length; g--; ) {
        var h = f[r ? g : ++s];
        if (u(c[h], h, c) === !1)
          break;
      }
      return i;
    };
  }
  return Vh = t, Vh;
}
var Yh, dS;
function Ky() {
  if (dS) return Yh;
  dS = 1;
  var t = PN(), r = t();
  return Yh = r, Yh;
}
var kh, hS;
function Qy() {
  if (hS) return kh;
  hS = 1;
  var t = Ky(), r = Lr();
  function i(u, l) {
    return u && t(u, l, r);
  }
  return kh = i, kh;
}
var Xh, gS;
function WN() {
  if (gS) return Xh;
  gS = 1;
  var t = Wn();
  function r(i, u) {
    return function(l, s) {
      if (l == null)
        return l;
      if (!t(l))
        return i(l, s);
      for (var c = l.length, f = u ? c : -1, g = Object(l); (u ? f-- : ++f < c) && s(g[f], f, g) !== !1; )
        ;
      return l;
    };
  }
  return Xh = r, Xh;
}
var Ih, vS;
function as() {
  if (vS) return Ih;
  vS = 1;
  var t = Qy(), r = WN(), i = r(t);
  return Ih = i, Ih;
}
var Zh, pS;
function ga() {
  if (pS) return Zh;
  pS = 1;
  function t(r) {
    return r;
  }
  return Zh = t, Zh;
}
var Kh, yS;
function oT() {
  if (yS) return Kh;
  yS = 1;
  var t = ga();
  function r(i) {
    return typeof i == "function" ? i : t;
  }
  return Kh = r, Kh;
}
var Qh, mS;
function sT() {
  if (mS) return Qh;
  mS = 1;
  var t = Uy(), r = as(), i = oT(), u = st();
  function l(s, c) {
    var f = u(s) ? t : r;
    return f(s, i(c));
  }
  return Qh = l, Qh;
}
var $h, bS;
function cT() {
  return bS || (bS = 1, $h = sT()), $h;
}
var Fh, _S;
function e3() {
  if (_S) return Fh;
  _S = 1;
  var t = as();
  function r(i, u) {
    var l = [];
    return t(i, function(s, c, f) {
      u(s, c, f) && l.push(s);
    }), l;
  }
  return Fh = r, Fh;
}
var Jh, xS;
function t3() {
  if (xS) return Jh;
  xS = 1;
  var t = "__lodash_hash_undefined__";
  function r(i) {
    return this.__data__.set(i, t), this;
  }
  return Jh = r, Jh;
}
var Ph, SS;
function n3() {
  if (SS) return Ph;
  SS = 1;
  function t(r) {
    return this.__data__.has(r);
  }
  return Ph = t, Ph;
}
var Wh, ES;
function fT() {
  if (ES) return Wh;
  ES = 1;
  var t = By(), r = t3(), i = n3();
  function u(l) {
    var s = -1, c = l == null ? 0 : l.length;
    for (this.__data__ = new t(); ++s < c; )
      this.add(l[s]);
  }
  return u.prototype.add = u.prototype.push = r, u.prototype.has = i, Wh = u, Wh;
}
var eg, wS;
function r3() {
  if (wS) return eg;
  wS = 1;
  function t(r, i) {
    for (var u = -1, l = r == null ? 0 : r.length; ++u < l; )
      if (i(r[u], u, r))
        return !0;
    return !1;
  }
  return eg = t, eg;
}
var tg, AS;
function dT() {
  if (AS) return tg;
  AS = 1;
  function t(r, i) {
    return r.has(i);
  }
  return tg = t, tg;
}
var ng, TS;
function hT() {
  if (TS) return ng;
  TS = 1;
  var t = fT(), r = r3(), i = dT(), u = 1, l = 2;
  function s(c, f, g, h, v, y) {
    var m = g & u, b = c.length, _ = f.length;
    if (b != _ && !(m && _ > b))
      return !1;
    var A = y.get(c), w = y.get(f);
    if (A && w)
      return A == f && w == c;
    var E = -1, M = !0, S = g & l ? new t() : void 0;
    for (y.set(c, f), y.set(f, c); ++E < b; ) {
      var T = c[E], N = f[E];
      if (h)
        var R = m ? h(N, T, E, f, c, y) : h(T, N, E, c, f, y);
      if (R !== void 0) {
        if (R)
          continue;
        M = !1;
        break;
      }
      if (S) {
        if (!r(f, function(q, L) {
          if (!i(S, L) && (T === q || v(T, q, g, h, y)))
            return S.push(L);
        })) {
          M = !1;
          break;
        }
      } else if (!(T === N || v(T, N, g, h, y))) {
        M = !1;
        break;
      }
    }
    return y.delete(c), y.delete(f), M;
  }
  return ng = s, ng;
}
var rg, MS;
function a3() {
  if (MS) return rg;
  MS = 1;
  function t(r) {
    var i = -1, u = Array(r.size);
    return r.forEach(function(l, s) {
      u[++i] = [s, l];
    }), u;
  }
  return rg = t, rg;
}
var ag, CS;
function $y() {
  if (CS) return ag;
  CS = 1;
  function t(r) {
    var i = -1, u = Array(r.size);
    return r.forEach(function(l) {
      u[++i] = l;
    }), u;
  }
  return ag = t, ag;
}
var ig, qS;
function i3() {
  if (qS) return ig;
  qS = 1;
  var t = pi(), r = rT(), i = vi(), u = hT(), l = a3(), s = $y(), c = 1, f = 2, g = "[object Boolean]", h = "[object Date]", v = "[object Error]", y = "[object Map]", m = "[object Number]", b = "[object RegExp]", _ = "[object Set]", A = "[object String]", w = "[object Symbol]", E = "[object ArrayBuffer]", M = "[object DataView]", S = t ? t.prototype : void 0, T = S ? S.valueOf : void 0;
  function N(R, q, L, j, Z, Y, J) {
    switch (L) {
      case M:
        if (R.byteLength != q.byteLength || R.byteOffset != q.byteOffset)
          return !1;
        R = R.buffer, q = q.buffer;
      case E:
        return !(R.byteLength != q.byteLength || !Y(new r(R), new r(q)));
      case g:
      case h:
      case m:
        return i(+R, +q);
      case v:
        return R.name == q.name && R.message == q.message;
      case b:
      case A:
        return R == q + "";
      case y:
        var Q = l;
      case _:
        var D = j & c;
        if (Q || (Q = s), R.size != q.size && !D)
          return !1;
        var X = J.get(R);
        if (X)
          return X == q;
        j |= f, J.set(R, q);
        var z = u(Q(R), Q(q), j, Z, Y, J);
        return J.delete(R), z;
      case w:
        if (T)
          return T.call(R) == T.call(q);
    }
    return !1;
  }
  return ig = N, ig;
}
var ug, RS;
function u3() {
  if (RS) return ug;
  RS = 1;
  var t = tT(), r = 1, i = Object.prototype, u = i.hasOwnProperty;
  function l(s, c, f, g, h, v) {
    var y = f & r, m = t(s), b = m.length, _ = t(c), A = _.length;
    if (b != A && !y)
      return !1;
    for (var w = b; w--; ) {
      var E = m[w];
      if (!(y ? E in c : u.call(c, E)))
        return !1;
    }
    var M = v.get(s), S = v.get(c);
    if (M && S)
      return M == c && S == s;
    var T = !0;
    v.set(s, c), v.set(c, s);
    for (var N = y; ++w < b; ) {
      E = m[w];
      var R = s[E], q = c[E];
      if (g)
        var L = y ? g(q, R, E, c, s, v) : g(R, q, E, s, c, v);
      if (!(L === void 0 ? R === q || h(R, q, f, g, v) : L)) {
        T = !1;
        break;
      }
      N || (N = E == "constructor");
    }
    if (T && !N) {
      var j = s.constructor, Z = c.constructor;
      j != Z && "constructor" in s && "constructor" in c && !(typeof j == "function" && j instanceof j && typeof Z == "function" && Z instanceof Z) && (T = !1);
    }
    return v.delete(s), v.delete(c), T;
  }
  return ug = l, ug;
}
var lg, NS;
function l3() {
  if (NS) return lg;
  NS = 1;
  var t = Jo(), r = hT(), i = i3(), u = u3(), l = mi(), s = st(), c = yi(), f = Vu(), g = 1, h = "[object Arguments]", v = "[object Array]", y = "[object Object]", m = Object.prototype, b = m.hasOwnProperty;
  function _(A, w, E, M, S, T) {
    var N = s(A), R = s(w), q = N ? v : l(A), L = R ? v : l(w);
    q = q == h ? y : q, L = L == h ? y : L;
    var j = q == y, Z = L == y, Y = q == L;
    if (Y && c(A)) {
      if (!c(w))
        return !1;
      N = !0, j = !1;
    }
    if (Y && !j)
      return T || (T = new t()), N || f(A) ? r(A, w, E, M, S, T) : i(A, w, q, E, M, S, T);
    if (!(E & g)) {
      var J = j && b.call(A, "__wrapped__"), Q = Z && b.call(w, "__wrapped__");
      if (J || Q) {
        var D = J ? A.value() : A, X = Q ? w.value() : w;
        return T || (T = new t()), S(D, X, E, M, T);
      }
    }
    return Y ? (T || (T = new t()), u(A, w, E, M, S, T)) : !1;
  }
  return lg = _, lg;
}
var og, zS;
function gT() {
  if (zS) return og;
  zS = 1;
  var t = l3(), r = Tn();
  function i(u, l, s, c, f) {
    return u === l ? !0 : u == null || l == null || !r(u) && !r(l) ? u !== u && l !== l : t(u, l, s, c, i, f);
  }
  return og = i, og;
}
var sg, OS;
function o3() {
  if (OS) return sg;
  OS = 1;
  var t = Jo(), r = gT(), i = 1, u = 2;
  function l(s, c, f, g) {
    var h = f.length, v = h, y = !g;
    if (s == null)
      return !v;
    for (s = Object(s); h--; ) {
      var m = f[h];
      if (y && m[2] ? m[1] !== s[m[0]] : !(m[0] in s))
        return !1;
    }
    for (; ++h < v; ) {
      m = f[h];
      var b = m[0], _ = s[b], A = m[1];
      if (y && m[2]) {
        if (_ === void 0 && !(b in s))
          return !1;
      } else {
        var w = new t();
        if (g)
          var E = g(_, A, b, s, c, w);
        if (!(E === void 0 ? r(A, _, i | u, g, w) : E))
          return !1;
      }
    }
    return !0;
  }
  return sg = l, sg;
}
var cg, DS;
function vT() {
  if (DS) return cg;
  DS = 1;
  var t = rn();
  function r(i) {
    return i === i && !t(i);
  }
  return cg = r, cg;
}
var fg, LS;
function s3() {
  if (LS) return fg;
  LS = 1;
  var t = vT(), r = Lr();
  function i(u) {
    for (var l = r(u), s = l.length; s--; ) {
      var c = l[s], f = u[c];
      l[s] = [c, f, t(f)];
    }
    return l;
  }
  return fg = i, fg;
}
var dg, HS;
function pT() {
  if (HS) return dg;
  HS = 1;
  function t(r, i) {
    return function(u) {
      return u == null ? !1 : u[r] === i && (i !== void 0 || r in Object(u));
    };
  }
  return dg = t, dg;
}
var hg, jS;
function c3() {
  if (jS) return hg;
  jS = 1;
  var t = o3(), r = s3(), i = pT();
  function u(l) {
    var s = r(l);
    return s.length == 1 && s[0][2] ? i(s[0][0], s[0][1]) : function(c) {
      return c === l || t(c, l, s);
    };
  }
  return hg = u, hg;
}
var gg, BS;
function bi() {
  if (BS) return gg;
  BS = 1;
  var t = fa(), r = Tn(), i = "[object Symbol]";
  function u(l) {
    return typeof l == "symbol" || r(l) && t(l) == i;
  }
  return gg = u, gg;
}
var vg, US;
function Fy() {
  if (US) return vg;
  US = 1;
  var t = st(), r = bi(), i = /\.|\[(?:[^[\]]*|(["'])(?:(?!\1)[^\\]|\\.)*?\1)\]/, u = /^\w*$/;
  function l(s, c) {
    if (t(s))
      return !1;
    var f = typeof s;
    return f == "number" || f == "symbol" || f == "boolean" || s == null || r(s) ? !0 : u.test(s) || !i.test(s) || c != null && s in Object(c);
  }
  return vg = l, vg;
}
var pg, GS;
function f3() {
  if (GS) return pg;
  GS = 1;
  var t = By(), r = "Expected a function";
  function i(u, l) {
    if (typeof u != "function" || l != null && typeof l != "function")
      throw new TypeError(r);
    var s = function() {
      var c = arguments, f = l ? l.apply(this, c) : c[0], g = s.cache;
      if (g.has(f))
        return g.get(f);
      var h = u.apply(this, c);
      return s.cache = g.set(f, h) || g, h;
    };
    return s.cache = new (i.Cache || t)(), s;
  }
  return i.Cache = t, pg = i, pg;
}
var yg, VS;
function d3() {
  if (VS) return yg;
  VS = 1;
  var t = f3(), r = 500;
  function i(u) {
    var l = t(u, function(c) {
      return s.size === r && s.clear(), c;
    }), s = l.cache;
    return l;
  }
  return yg = i, yg;
}
var mg, YS;
function h3() {
  if (YS) return mg;
  YS = 1;
  var t = d3(), r = /[^.[\]]+|\[(?:(-?\d+(?:\.\d+)?)|(["'])((?:(?!\2)[^\\]|\\.)*?)\2)\]|(?=(?:\.|\[\])(?:\.|\[\]|$))/g, i = /\\(\\)?/g, u = t(function(l) {
    var s = [];
    return l.charCodeAt(0) === 46 && s.push(""), l.replace(r, function(c, f, g, h) {
      s.push(g ? h.replace(i, "$1") : f || c);
    }), s;
  });
  return mg = u, mg;
}
var bg, kS;
function is() {
  if (kS) return bg;
  kS = 1;
  function t(r, i) {
    for (var u = -1, l = r == null ? 0 : r.length, s = Array(l); ++u < l; )
      s[u] = i(r[u], u, r);
    return s;
  }
  return bg = t, bg;
}
var _g, XS;
function g3() {
  if (XS) return _g;
  XS = 1;
  var t = pi(), r = is(), i = st(), u = bi(), l = t ? t.prototype : void 0, s = l ? l.toString : void 0;
  function c(f) {
    if (typeof f == "string")
      return f;
    if (i(f))
      return r(f, c) + "";
    if (u(f))
      return s ? s.call(f) : "";
    var g = f + "";
    return g == "0" && 1 / f == -1 / 0 ? "-0" : g;
  }
  return _g = c, _g;
}
var xg, IS;
function yT() {
  if (IS) return xg;
  IS = 1;
  var t = g3();
  function r(i) {
    return i == null ? "" : t(i);
  }
  return xg = r, xg;
}
var Sg, ZS;
function us() {
  if (ZS) return Sg;
  ZS = 1;
  var t = st(), r = Fy(), i = h3(), u = yT();
  function l(s, c) {
    return t(s) ? s : r(s, c) ? [s] : i(u(s));
  }
  return Sg = l, Sg;
}
var Eg, KS;
function Yu() {
  if (KS) return Eg;
  KS = 1;
  var t = bi();
  function r(i) {
    if (typeof i == "string" || t(i))
      return i;
    var u = i + "";
    return u == "0" && 1 / i == -1 / 0 ? "-0" : u;
  }
  return Eg = r, Eg;
}
var wg, QS;
function ls() {
  if (QS) return wg;
  QS = 1;
  var t = us(), r = Yu();
  function i(u, l) {
    l = t(l, u);
    for (var s = 0, c = l.length; u != null && s < c; )
      u = u[r(l[s++])];
    return s && s == c ? u : void 0;
  }
  return wg = i, wg;
}
var Ag, $S;
function v3() {
  if ($S) return Ag;
  $S = 1;
  var t = ls();
  function r(i, u, l) {
    var s = i == null ? void 0 : t(i, u);
    return s === void 0 ? l : s;
  }
  return Ag = r, Ag;
}
var Tg, FS;
function p3() {
  if (FS) return Tg;
  FS = 1;
  function t(r, i) {
    return r != null && i in Object(r);
  }
  return Tg = t, Tg;
}
var Mg, JS;
function mT() {
  if (JS) return Mg;
  JS = 1;
  var t = us(), r = Gu(), i = st(), u = es(), l = Gy(), s = Yu();
  function c(f, g, h) {
    g = t(g, f);
    for (var v = -1, y = g.length, m = !1; ++v < y; ) {
      var b = s(g[v]);
      if (!(m = f != null && h(f, b)))
        break;
      f = f[b];
    }
    return m || ++v != y ? m : (y = f == null ? 0 : f.length, !!y && l(y) && u(b, y) && (i(f) || r(f)));
  }
  return Mg = c, Mg;
}
var Cg, PS;
function bT() {
  if (PS) return Cg;
  PS = 1;
  var t = p3(), r = mT();
  function i(u, l) {
    return u != null && r(u, l, t);
  }
  return Cg = i, Cg;
}
var qg, WS;
function y3() {
  if (WS) return qg;
  WS = 1;
  var t = gT(), r = v3(), i = bT(), u = Fy(), l = vT(), s = pT(), c = Yu(), f = 1, g = 2;
  function h(v, y) {
    return u(v) && l(y) ? s(c(v), y) : function(m) {
      var b = r(m, v);
      return b === void 0 && b === y ? i(m, v) : t(y, b, f | g);
    };
  }
  return qg = h, qg;
}
var Rg, eE;
function _T() {
  if (eE) return Rg;
  eE = 1;
  function t(r) {
    return function(i) {
      return i == null ? void 0 : i[r];
    };
  }
  return Rg = t, Rg;
}
var Ng, tE;
function m3() {
  if (tE) return Ng;
  tE = 1;
  var t = ls();
  function r(i) {
    return function(u) {
      return t(u, i);
    };
  }
  return Ng = r, Ng;
}
var zg, nE;
function b3() {
  if (nE) return zg;
  nE = 1;
  var t = _T(), r = m3(), i = Fy(), u = Yu();
  function l(s) {
    return i(s) ? t(u(s)) : r(s);
  }
  return zg = l, zg;
}
var Og, rE;
function er() {
  if (rE) return Og;
  rE = 1;
  var t = c3(), r = y3(), i = ga(), u = st(), l = b3();
  function s(c) {
    return typeof c == "function" ? c : c == null ? i : typeof c == "object" ? u(c) ? r(c[0], c[1]) : t(c) : l(c);
  }
  return Og = s, Og;
}
var Dg, aE;
function xT() {
  if (aE) return Dg;
  aE = 1;
  var t = JA(), r = e3(), i = er(), u = st();
  function l(s, c) {
    var f = u(s) ? t : r;
    return f(s, i(c, 3));
  }
  return Dg = l, Dg;
}
var Lg, iE;
function _3() {
  if (iE) return Lg;
  iE = 1;
  var t = Object.prototype, r = t.hasOwnProperty;
  function i(u, l) {
    return u != null && r.call(u, l);
  }
  return Lg = i, Lg;
}
var Hg, uE;
function ST() {
  if (uE) return Hg;
  uE = 1;
  var t = _3(), r = mT();
  function i(u, l) {
    return u != null && r(u, l, t);
  }
  return Hg = i, Hg;
}
var jg, lE;
function x3() {
  if (lE) return jg;
  lE = 1;
  var t = Yy(), r = mi(), i = Gu(), u = st(), l = Wn(), s = yi(), c = ns(), f = Vu(), g = "[object Map]", h = "[object Set]", v = Object.prototype, y = v.hasOwnProperty;
  function m(b) {
    if (b == null)
      return !0;
    if (l(b) && (u(b) || typeof b == "string" || typeof b.splice == "function" || s(b) || f(b) || i(b)))
      return !b.length;
    var _ = r(b);
    if (_ == g || _ == h)
      return !b.size;
    if (c(b))
      return !t(b).length;
    for (var A in b)
      if (y.call(b, A))
        return !1;
    return !0;
  }
  return jg = m, jg;
}
var Bg, oE;
function ET() {
  if (oE) return Bg;
  oE = 1;
  function t(r) {
    return r === void 0;
  }
  return Bg = t, Bg;
}
var Ug, sE;
function wT() {
  if (sE) return Ug;
  sE = 1;
  var t = as(), r = Wn();
  function i(u, l) {
    var s = -1, c = r(u) ? Array(u.length) : [];
    return t(u, function(f, g, h) {
      c[++s] = l(f, g, h);
    }), c;
  }
  return Ug = i, Ug;
}
var Gg, cE;
function AT() {
  if (cE) return Gg;
  cE = 1;
  var t = is(), r = er(), i = wT(), u = st();
  function l(s, c) {
    var f = u(s) ? t : i;
    return f(s, r(c, 3));
  }
  return Gg = l, Gg;
}
var Vg, fE;
function S3() {
  if (fE) return Vg;
  fE = 1;
  function t(r, i, u, l) {
    var s = -1, c = r == null ? 0 : r.length;
    for (l && c && (u = r[++s]); ++s < c; )
      u = i(u, r[s], s, r);
    return u;
  }
  return Vg = t, Vg;
}
var Yg, dE;
function E3() {
  if (dE) return Yg;
  dE = 1;
  function t(r, i, u, l, s) {
    return s(r, function(c, f, g) {
      u = l ? (l = !1, c) : i(u, c, f, g);
    }), u;
  }
  return Yg = t, Yg;
}
var kg, hE;
function TT() {
  if (hE) return kg;
  hE = 1;
  var t = S3(), r = as(), i = er(), u = E3(), l = st();
  function s(c, f, g) {
    var h = l(c) ? t : u, v = arguments.length < 3;
    return h(c, i(f, 4), g, v, r);
  }
  return kg = s, kg;
}
var Xg, gE;
function w3() {
  if (gE) return Xg;
  gE = 1;
  var t = fa(), r = st(), i = Tn(), u = "[object String]";
  function l(s) {
    return typeof s == "string" || !r(s) && i(s) && t(s) == u;
  }
  return Xg = l, Xg;
}
var Ig, vE;
function A3() {
  if (vE) return Ig;
  vE = 1;
  var t = _T(), r = t("length");
  return Ig = r, Ig;
}
var Zg, pE;
function T3() {
  if (pE) return Zg;
  pE = 1;
  var t = "\\ud800-\\udfff", r = "\\u0300-\\u036f", i = "\\ufe20-\\ufe2f", u = "\\u20d0-\\u20ff", l = r + i + u, s = "\\ufe0e\\ufe0f", c = "\\u200d", f = RegExp("[" + c + t + l + s + "]");
  function g(h) {
    return f.test(h);
  }
  return Zg = g, Zg;
}
var Kg, yE;
function M3() {
  if (yE) return Kg;
  yE = 1;
  var t = "\\ud800-\\udfff", r = "\\u0300-\\u036f", i = "\\ufe20-\\ufe2f", u = "\\u20d0-\\u20ff", l = r + i + u, s = "\\ufe0e\\ufe0f", c = "[" + t + "]", f = "[" + l + "]", g = "\\ud83c[\\udffb-\\udfff]", h = "(?:" + f + "|" + g + ")", v = "[^" + t + "]", y = "(?:\\ud83c[\\udde6-\\uddff]){2}", m = "[\\ud800-\\udbff][\\udc00-\\udfff]", b = "\\u200d", _ = h + "?", A = "[" + s + "]?", w = "(?:" + b + "(?:" + [v, y, m].join("|") + ")" + A + _ + ")*", E = A + _ + w, M = "(?:" + [v + f + "?", f, y, m, c].join("|") + ")", S = RegExp(g + "(?=" + g + ")|" + M + E, "g");
  function T(N) {
    for (var R = S.lastIndex = 0; S.test(N); )
      ++R;
    return R;
  }
  return Kg = T, Kg;
}
var Qg, mE;
function C3() {
  if (mE) return Qg;
  mE = 1;
  var t = A3(), r = T3(), i = M3();
  function u(l) {
    return r(l) ? i(l) : t(l);
  }
  return Qg = u, Qg;
}
var $g, bE;
function q3() {
  if (bE) return $g;
  bE = 1;
  var t = Yy(), r = mi(), i = Wn(), u = w3(), l = C3(), s = "[object Map]", c = "[object Set]";
  function f(g) {
    if (g == null)
      return 0;
    if (i(g))
      return u(g) ? l(g) : g.length;
    var h = r(g);
    return h == s || h == c ? g.size : t(g).length;
  }
  return $g = f, $g;
}
var Fg, _E;
function R3() {
  if (_E) return Fg;
  _E = 1;
  var t = Uy(), r = iT(), i = Qy(), u = er(), l = rs(), s = st(), c = yi(), f = Bu(), g = rn(), h = Vu();
  function v(y, m, b) {
    var _ = s(y), A = _ || c(y) || h(y);
    if (m = u(m, 4), b == null) {
      var w = y && y.constructor;
      A ? b = _ ? new w() : [] : g(y) ? b = f(w) ? r(l(y)) : {} : b = {};
    }
    return (A ? t : i)(y, function(E, M, S) {
      return m(b, E, M, S);
    }), b;
  }
  return Fg = v, Fg;
}
var Jg, xE;
function N3() {
  if (xE) return Jg;
  xE = 1;
  var t = pi(), r = Gu(), i = st(), u = t ? t.isConcatSpreadable : void 0;
  function l(s) {
    return i(s) || r(s) || !!(u && s && s[u]);
  }
  return Jg = l, Jg;
}
var Pg, SE;
function Jy() {
  if (SE) return Pg;
  SE = 1;
  var t = Xy(), r = N3();
  function i(u, l, s, c, f) {
    var g = -1, h = u.length;
    for (s || (s = r), f || (f = []); ++g < h; ) {
      var v = u[g];
      l > 0 && s(v) ? l > 1 ? i(v, l - 1, s, c, f) : t(f, v) : c || (f[f.length] = v);
    }
    return f;
  }
  return Pg = i, Pg;
}
var Wg, EE;
function z3() {
  if (EE) return Wg;
  EE = 1;
  function t(r, i, u) {
    switch (u.length) {
      case 0:
        return r.call(i);
      case 1:
        return r.call(i, u[0]);
      case 2:
        return r.call(i, u[0], u[1]);
      case 3:
        return r.call(i, u[0], u[1], u[2]);
    }
    return r.apply(i, u);
  }
  return Wg = t, Wg;
}
var ev, wE;
function MT() {
  if (wE) return ev;
  wE = 1;
  var t = z3(), r = Math.max;
  function i(u, l, s) {
    return l = r(l === void 0 ? u.length - 1 : l, 0), function() {
      for (var c = arguments, f = -1, g = r(c.length - l, 0), h = Array(g); ++f < g; )
        h[f] = c[l + f];
      f = -1;
      for (var v = Array(l + 1); ++f < l; )
        v[f] = c[f];
      return v[l] = s(h), t(u, this, v);
    };
  }
  return ev = i, ev;
}
var tv, AE;
function O3() {
  if (AE) return tv;
  AE = 1;
  var t = Zy(), r = ZA(), i = ga(), u = r ? function(l, s) {
    return r(l, "toString", {
      configurable: !0,
      enumerable: !1,
      value: t(s),
      writable: !0
    });
  } : i;
  return tv = u, tv;
}
var nv, TE;
function D3() {
  if (TE) return nv;
  TE = 1;
  var t = 800, r = 16, i = Date.now;
  function u(l) {
    var s = 0, c = 0;
    return function() {
      var f = i(), g = r - (f - c);
      if (c = f, g > 0) {
        if (++s >= t)
          return arguments[0];
      } else
        s = 0;
      return l.apply(void 0, arguments);
    };
  }
  return nv = u, nv;
}
var rv, ME;
function CT() {
  if (ME) return rv;
  ME = 1;
  var t = O3(), r = D3(), i = r(t);
  return rv = i, rv;
}
var av, CE;
function os() {
  if (CE) return av;
  CE = 1;
  var t = ga(), r = MT(), i = CT();
  function u(l, s) {
    return i(r(l, s, t), l + "");
  }
  return av = u, av;
}
var iv, qE;
function qT() {
  if (qE) return iv;
  qE = 1;
  function t(r, i, u, l) {
    for (var s = r.length, c = u + (l ? 1 : -1); l ? c-- : ++c < s; )
      if (i(r[c], c, r))
        return c;
    return -1;
  }
  return iv = t, iv;
}
var uv, RE;
function L3() {
  if (RE) return uv;
  RE = 1;
  function t(r) {
    return r !== r;
  }
  return uv = t, uv;
}
var lv, NE;
function H3() {
  if (NE) return lv;
  NE = 1;
  function t(r, i, u) {
    for (var l = u - 1, s = r.length; ++l < s; )
      if (r[l] === i)
        return l;
    return -1;
  }
  return lv = t, lv;
}
var ov, zE;
function j3() {
  if (zE) return ov;
  zE = 1;
  var t = qT(), r = L3(), i = H3();
  function u(l, s, c) {
    return s === s ? i(l, s, c) : t(l, r, c);
  }
  return ov = u, ov;
}
var sv, OE;
function B3() {
  if (OE) return sv;
  OE = 1;
  var t = j3();
  function r(i, u) {
    var l = i == null ? 0 : i.length;
    return !!l && t(i, u, 0) > -1;
  }
  return sv = r, sv;
}
var cv, DE;
function U3() {
  if (DE) return cv;
  DE = 1;
  function t(r, i, u) {
    for (var l = -1, s = r == null ? 0 : r.length; ++l < s; )
      if (u(i, r[l]))
        return !0;
    return !1;
  }
  return cv = t, cv;
}
var fv, LE;
function G3() {
  if (LE) return fv;
  LE = 1;
  function t() {
  }
  return fv = t, fv;
}
var dv, HE;
function V3() {
  if (HE) return dv;
  HE = 1;
  var t = nT(), r = G3(), i = $y(), u = 1 / 0, l = t && 1 / i(new t([, -0]))[1] == u ? function(s) {
    return new t(s);
  } : r;
  return dv = l, dv;
}
var hv, jE;
function Y3() {
  if (jE) return hv;
  jE = 1;
  var t = fT(), r = B3(), i = U3(), u = dT(), l = V3(), s = $y(), c = 200;
  function f(g, h, v) {
    var y = -1, m = r, b = g.length, _ = !0, A = [], w = A;
    if (v)
      _ = !1, m = i;
    else if (b >= c) {
      var E = h ? null : l(g);
      if (E)
        return s(E);
      _ = !1, m = u, w = new t();
    } else
      w = h ? [] : A;
    e:
      for (; ++y < b; ) {
        var M = g[y], S = h ? h(M) : M;
        if (M = v || M !== 0 ? M : 0, _ && S === S) {
          for (var T = w.length; T--; )
            if (w[T] === S)
              continue e;
          h && w.push(S), A.push(M);
        } else m(w, S, v) || (w !== A && w.push(S), A.push(M));
      }
    return A;
  }
  return hv = f, hv;
}
var gv, BE;
function RT() {
  if (BE) return gv;
  BE = 1;
  var t = Wn(), r = Tn();
  function i(u) {
    return r(u) && t(u);
  }
  return gv = i, gv;
}
var vv, UE;
function k3() {
  if (UE) return vv;
  UE = 1;
  var t = Jy(), r = os(), i = Y3(), u = RT(), l = r(function(s) {
    return i(t(s, 1, u, !0));
  });
  return vv = l, vv;
}
var pv, GE;
function X3() {
  if (GE) return pv;
  GE = 1;
  var t = is();
  function r(i, u) {
    return t(u, function(l) {
      return i[l];
    });
  }
  return pv = r, pv;
}
var yv, VE;
function NT() {
  if (VE) return yv;
  VE = 1;
  var t = X3(), r = Lr();
  function i(u) {
    return u == null ? [] : t(u, r(u));
  }
  return yv = i, yv;
}
var mv, YE;
function an() {
  if (YE) return mv;
  YE = 1;
  var t;
  if (typeof Hy == "function")
    try {
      t = {
        clone: JN(),
        constant: Zy(),
        each: cT(),
        filter: xT(),
        has: ST(),
        isArray: st(),
        isEmpty: x3(),
        isFunction: Bu(),
        isUndefined: ET(),
        keys: Lr(),
        map: AT(),
        reduce: TT(),
        size: q3(),
        transform: R3(),
        union: k3(),
        values: NT()
      };
    } catch {
    }
  return t || (t = window._), mv = t, mv;
}
var bv, kE;
function Py() {
  if (kE) return bv;
  kE = 1;
  var t = an();
  bv = l;
  var r = "\0", i = "\0", u = "";
  function l(v) {
    this._isDirected = t.has(v, "directed") ? v.directed : !0, this._isMultigraph = t.has(v, "multigraph") ? v.multigraph : !1, this._isCompound = t.has(v, "compound") ? v.compound : !1, this._label = void 0, this._defaultNodeLabelFn = t.constant(void 0), this._defaultEdgeLabelFn = t.constant(void 0), this._nodes = {}, this._isCompound && (this._parent = {}, this._children = {}, this._children[i] = {}), this._in = {}, this._preds = {}, this._out = {}, this._sucs = {}, this._edgeObjs = {}, this._edgeLabels = {};
  }
  l.prototype._nodeCount = 0, l.prototype._edgeCount = 0, l.prototype.isDirected = function() {
    return this._isDirected;
  }, l.prototype.isMultigraph = function() {
    return this._isMultigraph;
  }, l.prototype.isCompound = function() {
    return this._isCompound;
  }, l.prototype.setGraph = function(v) {
    return this._label = v, this;
  }, l.prototype.graph = function() {
    return this._label;
  }, l.prototype.setDefaultNodeLabel = function(v) {
    return t.isFunction(v) || (v = t.constant(v)), this._defaultNodeLabelFn = v, this;
  }, l.prototype.nodeCount = function() {
    return this._nodeCount;
  }, l.prototype.nodes = function() {
    return t.keys(this._nodes);
  }, l.prototype.sources = function() {
    var v = this;
    return t.filter(this.nodes(), function(y) {
      return t.isEmpty(v._in[y]);
    });
  }, l.prototype.sinks = function() {
    var v = this;
    return t.filter(this.nodes(), function(y) {
      return t.isEmpty(v._out[y]);
    });
  }, l.prototype.setNodes = function(v, y) {
    var m = arguments, b = this;
    return t.each(v, function(_) {
      m.length > 1 ? b.setNode(_, y) : b.setNode(_);
    }), this;
  }, l.prototype.setNode = function(v, y) {
    return t.has(this._nodes, v) ? (arguments.length > 1 && (this._nodes[v] = y), this) : (this._nodes[v] = arguments.length > 1 ? y : this._defaultNodeLabelFn(v), this._isCompound && (this._parent[v] = i, this._children[v] = {}, this._children[i][v] = !0), this._in[v] = {}, this._preds[v] = {}, this._out[v] = {}, this._sucs[v] = {}, ++this._nodeCount, this);
  }, l.prototype.node = function(v) {
    return this._nodes[v];
  }, l.prototype.hasNode = function(v) {
    return t.has(this._nodes, v);
  }, l.prototype.removeNode = function(v) {
    var y = this;
    if (t.has(this._nodes, v)) {
      var m = function(b) {
        y.removeEdge(y._edgeObjs[b]);
      };
      delete this._nodes[v], this._isCompound && (this._removeFromParentsChildList(v), delete this._parent[v], t.each(this.children(v), function(b) {
        y.setParent(b);
      }), delete this._children[v]), t.each(t.keys(this._in[v]), m), delete this._in[v], delete this._preds[v], t.each(t.keys(this._out[v]), m), delete this._out[v], delete this._sucs[v], --this._nodeCount;
    }
    return this;
  }, l.prototype.setParent = function(v, y) {
    if (!this._isCompound)
      throw new Error("Cannot set parent in a non-compound graph");
    if (t.isUndefined(y))
      y = i;
    else {
      y += "";
      for (var m = y; !t.isUndefined(m); m = this.parent(m))
        if (m === v)
          throw new Error("Setting " + y + " as parent of " + v + " would create a cycle");
      this.setNode(y);
    }
    return this.setNode(v), this._removeFromParentsChildList(v), this._parent[v] = y, this._children[y][v] = !0, this;
  }, l.prototype._removeFromParentsChildList = function(v) {
    delete this._children[this._parent[v]][v];
  }, l.prototype.parent = function(v) {
    if (this._isCompound) {
      var y = this._parent[v];
      if (y !== i)
        return y;
    }
  }, l.prototype.children = function(v) {
    if (t.isUndefined(v) && (v = i), this._isCompound) {
      var y = this._children[v];
      if (y)
        return t.keys(y);
    } else {
      if (v === i)
        return this.nodes();
      if (this.hasNode(v))
        return [];
    }
  }, l.prototype.predecessors = function(v) {
    var y = this._preds[v];
    if (y)
      return t.keys(y);
  }, l.prototype.successors = function(v) {
    var y = this._sucs[v];
    if (y)
      return t.keys(y);
  }, l.prototype.neighbors = function(v) {
    var y = this.predecessors(v);
    if (y)
      return t.union(y, this.successors(v));
  }, l.prototype.isLeaf = function(v) {
    var y;
    return this.isDirected() ? y = this.successors(v) : y = this.neighbors(v), y.length === 0;
  }, l.prototype.filterNodes = function(v) {
    var y = new this.constructor({
      directed: this._isDirected,
      multigraph: this._isMultigraph,
      compound: this._isCompound
    });
    y.setGraph(this.graph());
    var m = this;
    t.each(this._nodes, function(A, w) {
      v(w) && y.setNode(w, A);
    }), t.each(this._edgeObjs, function(A) {
      y.hasNode(A.v) && y.hasNode(A.w) && y.setEdge(A, m.edge(A));
    });
    var b = {};
    function _(A) {
      var w = m.parent(A);
      return w === void 0 || y.hasNode(w) ? (b[A] = w, w) : w in b ? b[w] : _(w);
    }
    return this._isCompound && t.each(y.nodes(), function(A) {
      y.setParent(A, _(A));
    }), y;
  }, l.prototype.setDefaultEdgeLabel = function(v) {
    return t.isFunction(v) || (v = t.constant(v)), this._defaultEdgeLabelFn = v, this;
  }, l.prototype.edgeCount = function() {
    return this._edgeCount;
  }, l.prototype.edges = function() {
    return t.values(this._edgeObjs);
  }, l.prototype.setPath = function(v, y) {
    var m = this, b = arguments;
    return t.reduce(v, function(_, A) {
      return b.length > 1 ? m.setEdge(_, A, y) : m.setEdge(_, A), A;
    }), this;
  }, l.prototype.setEdge = function() {
    var v, y, m, b, _ = !1, A = arguments[0];
    typeof A == "object" && A !== null && "v" in A ? (v = A.v, y = A.w, m = A.name, arguments.length === 2 && (b = arguments[1], _ = !0)) : (v = A, y = arguments[1], m = arguments[3], arguments.length > 2 && (b = arguments[2], _ = !0)), v = "" + v, y = "" + y, t.isUndefined(m) || (m = "" + m);
    var w = f(this._isDirected, v, y, m);
    if (t.has(this._edgeLabels, w))
      return _ && (this._edgeLabels[w] = b), this;
    if (!t.isUndefined(m) && !this._isMultigraph)
      throw new Error("Cannot set a named edge when isMultigraph = false");
    this.setNode(v), this.setNode(y), this._edgeLabels[w] = _ ? b : this._defaultEdgeLabelFn(v, y, m);
    var E = g(this._isDirected, v, y, m);
    return v = E.v, y = E.w, Object.freeze(E), this._edgeObjs[w] = E, s(this._preds[y], v), s(this._sucs[v], y), this._in[y][w] = E, this._out[v][w] = E, this._edgeCount++, this;
  }, l.prototype.edge = function(v, y, m) {
    var b = arguments.length === 1 ? h(this._isDirected, arguments[0]) : f(this._isDirected, v, y, m);
    return this._edgeLabels[b];
  }, l.prototype.hasEdge = function(v, y, m) {
    var b = arguments.length === 1 ? h(this._isDirected, arguments[0]) : f(this._isDirected, v, y, m);
    return t.has(this._edgeLabels, b);
  }, l.prototype.removeEdge = function(v, y, m) {
    var b = arguments.length === 1 ? h(this._isDirected, arguments[0]) : f(this._isDirected, v, y, m), _ = this._edgeObjs[b];
    return _ && (v = _.v, y = _.w, delete this._edgeLabels[b], delete this._edgeObjs[b], c(this._preds[y], v), c(this._sucs[v], y), delete this._in[y][b], delete this._out[v][b], this._edgeCount--), this;
  }, l.prototype.inEdges = function(v, y) {
    var m = this._in[v];
    if (m) {
      var b = t.values(m);
      return y ? t.filter(b, function(_) {
        return _.v === y;
      }) : b;
    }
  }, l.prototype.outEdges = function(v, y) {
    var m = this._out[v];
    if (m) {
      var b = t.values(m);
      return y ? t.filter(b, function(_) {
        return _.w === y;
      }) : b;
    }
  }, l.prototype.nodeEdges = function(v, y) {
    var m = this.inEdges(v, y);
    if (m)
      return m.concat(this.outEdges(v, y));
  };
  function s(v, y) {
    v[y] ? v[y]++ : v[y] = 1;
  }
  function c(v, y) {
    --v[y] || delete v[y];
  }
  function f(v, y, m, b) {
    var _ = "" + y, A = "" + m;
    if (!v && _ > A) {
      var w = _;
      _ = A, A = w;
    }
    return _ + u + A + u + (t.isUndefined(b) ? r : b);
  }
  function g(v, y, m, b) {
    var _ = "" + y, A = "" + m;
    if (!v && _ > A) {
      var w = _;
      _ = A, A = w;
    }
    var E = { v: _, w: A };
    return b && (E.name = b), E;
  }
  function h(v, y) {
    return f(v, y.v, y.w, y.name);
  }
  return bv;
}
var _v, XE;
function I3() {
  return XE || (XE = 1, _v = "2.1.8"), _v;
}
var xv, IE;
function Z3() {
  return IE || (IE = 1, xv = {
    Graph: Py(),
    version: I3()
  }), xv;
}
var Sv, ZE;
function K3() {
  if (ZE) return Sv;
  ZE = 1;
  var t = an(), r = Py();
  Sv = {
    write: i,
    read: s
  };
  function i(c) {
    var f = {
      options: {
        directed: c.isDirected(),
        multigraph: c.isMultigraph(),
        compound: c.isCompound()
      },
      nodes: u(c),
      edges: l(c)
    };
    return t.isUndefined(c.graph()) || (f.value = t.clone(c.graph())), f;
  }
  function u(c) {
    return t.map(c.nodes(), function(f) {
      var g = c.node(f), h = c.parent(f), v = { v: f };
      return t.isUndefined(g) || (v.value = g), t.isUndefined(h) || (v.parent = h), v;
    });
  }
  function l(c) {
    return t.map(c.edges(), function(f) {
      var g = c.edge(f), h = { v: f.v, w: f.w };
      return t.isUndefined(f.name) || (h.name = f.name), t.isUndefined(g) || (h.value = g), h;
    });
  }
  function s(c) {
    var f = new r(c.options).setGraph(c.value);
    return t.each(c.nodes, function(g) {
      f.setNode(g.v, g.value), g.parent && f.setParent(g.v, g.parent);
    }), t.each(c.edges, function(g) {
      f.setEdge({ v: g.v, w: g.w, name: g.name }, g.value);
    }), f;
  }
  return Sv;
}
var Ev, KE;
function Q3() {
  if (KE) return Ev;
  KE = 1;
  var t = an();
  Ev = r;
  function r(i) {
    var u = {}, l = [], s;
    function c(f) {
      t.has(u, f) || (u[f] = !0, s.push(f), t.each(i.successors(f), c), t.each(i.predecessors(f), c));
    }
    return t.each(i.nodes(), function(f) {
      s = [], c(f), s.length && l.push(s);
    }), l;
  }
  return Ev;
}
var wv, QE;
function zT() {
  if (QE) return wv;
  QE = 1;
  var t = an();
  wv = r;
  function r() {
    this._arr = [], this._keyIndices = {};
  }
  return r.prototype.size = function() {
    return this._arr.length;
  }, r.prototype.keys = function() {
    return this._arr.map(function(i) {
      return i.key;
    });
  }, r.prototype.has = function(i) {
    return t.has(this._keyIndices, i);
  }, r.prototype.priority = function(i) {
    var u = this._keyIndices[i];
    if (u !== void 0)
      return this._arr[u].priority;
  }, r.prototype.min = function() {
    if (this.size() === 0)
      throw new Error("Queue underflow");
    return this._arr[0].key;
  }, r.prototype.add = function(i, u) {
    var l = this._keyIndices;
    if (i = String(i), !t.has(l, i)) {
      var s = this._arr, c = s.length;
      return l[i] = c, s.push({ key: i, priority: u }), this._decrease(c), !0;
    }
    return !1;
  }, r.prototype.removeMin = function() {
    this._swap(0, this._arr.length - 1);
    var i = this._arr.pop();
    return delete this._keyIndices[i.key], this._heapify(0), i.key;
  }, r.prototype.decrease = function(i, u) {
    var l = this._keyIndices[i];
    if (u > this._arr[l].priority)
      throw new Error("New priority is greater than current priority. Key: " + i + " Old: " + this._arr[l].priority + " New: " + u);
    this._arr[l].priority = u, this._decrease(l);
  }, r.prototype._heapify = function(i) {
    var u = this._arr, l = 2 * i, s = l + 1, c = i;
    l < u.length && (c = u[l].priority < u[c].priority ? l : c, s < u.length && (c = u[s].priority < u[c].priority ? s : c), c !== i && (this._swap(i, c), this._heapify(c)));
  }, r.prototype._decrease = function(i) {
    for (var u = this._arr, l = u[i].priority, s; i !== 0 && (s = i >> 1, !(u[s].priority < l)); )
      this._swap(i, s), i = s;
  }, r.prototype._swap = function(i, u) {
    var l = this._arr, s = this._keyIndices, c = l[i], f = l[u];
    l[i] = f, l[u] = c, s[f.key] = i, s[c.key] = u;
  }, wv;
}
var Av, $E;
function OT() {
  if ($E) return Av;
  $E = 1;
  var t = an(), r = zT();
  Av = u;
  var i = t.constant(1);
  function u(s, c, f, g) {
    return l(
      s,
      String(c),
      f || i,
      g || function(h) {
        return s.outEdges(h);
      }
    );
  }
  function l(s, c, f, g) {
    var h = {}, v = new r(), y, m, b = function(_) {
      var A = _.v !== y ? _.v : _.w, w = h[A], E = f(_), M = m.distance + E;
      if (E < 0)
        throw new Error("dijkstra does not allow negative edge weights. Bad edge: " + _ + " Weight: " + E);
      M < w.distance && (w.distance = M, w.predecessor = y, v.decrease(A, M));
    };
    for (s.nodes().forEach(function(_) {
      var A = _ === c ? 0 : Number.POSITIVE_INFINITY;
      h[_] = { distance: A }, v.add(_, A);
    }); v.size() > 0 && (y = v.removeMin(), m = h[y], m.distance !== Number.POSITIVE_INFINITY); )
      g(y).forEach(b);
    return h;
  }
  return Av;
}
var Tv, FE;
function $3() {
  if (FE) return Tv;
  FE = 1;
  var t = OT(), r = an();
  Tv = i;
  function i(u, l, s) {
    return r.transform(u.nodes(), function(c, f) {
      c[f] = t(u, f, l, s);
    }, {});
  }
  return Tv;
}
var Mv, JE;
function DT() {
  if (JE) return Mv;
  JE = 1;
  var t = an();
  Mv = r;
  function r(i) {
    var u = 0, l = [], s = {}, c = [];
    function f(g) {
      var h = s[g] = {
        onStack: !0,
        lowlink: u,
        index: u++
      };
      if (l.push(g), i.successors(g).forEach(function(m) {
        t.has(s, m) ? s[m].onStack && (h.lowlink = Math.min(h.lowlink, s[m].index)) : (f(m), h.lowlink = Math.min(h.lowlink, s[m].lowlink));
      }), h.lowlink === h.index) {
        var v = [], y;
        do
          y = l.pop(), s[y].onStack = !1, v.push(y);
        while (g !== y);
        c.push(v);
      }
    }
    return i.nodes().forEach(function(g) {
      t.has(s, g) || f(g);
    }), c;
  }
  return Mv;
}
var Cv, PE;
function F3() {
  if (PE) return Cv;
  PE = 1;
  var t = an(), r = DT();
  Cv = i;
  function i(u) {
    return t.filter(r(u), function(l) {
      return l.length > 1 || l.length === 1 && u.hasEdge(l[0], l[0]);
    });
  }
  return Cv;
}
var qv, WE;
function J3() {
  if (WE) return qv;
  WE = 1;
  var t = an();
  qv = i;
  var r = t.constant(1);
  function i(l, s, c) {
    return u(
      l,
      s || r,
      c || function(f) {
        return l.outEdges(f);
      }
    );
  }
  function u(l, s, c) {
    var f = {}, g = l.nodes();
    return g.forEach(function(h) {
      f[h] = {}, f[h][h] = { distance: 0 }, g.forEach(function(v) {
        h !== v && (f[h][v] = { distance: Number.POSITIVE_INFINITY });
      }), c(h).forEach(function(v) {
        var y = v.v === h ? v.w : v.v, m = s(v);
        f[h][y] = { distance: m, predecessor: h };
      });
    }), g.forEach(function(h) {
      var v = f[h];
      g.forEach(function(y) {
        var m = f[y];
        g.forEach(function(b) {
          var _ = m[h], A = v[b], w = m[b], E = _.distance + A.distance;
          E < w.distance && (w.distance = E, w.predecessor = A.predecessor);
        });
      });
    }), f;
  }
  return qv;
}
var Rv, ew;
function LT() {
  if (ew) return Rv;
  ew = 1;
  var t = an();
  Rv = r, r.CycleException = i;
  function r(u) {
    var l = {}, s = {}, c = [];
    function f(g) {
      if (t.has(s, g))
        throw new i();
      t.has(l, g) || (s[g] = !0, l[g] = !0, t.each(u.predecessors(g), f), delete s[g], c.push(g));
    }
    if (t.each(u.sinks(), f), t.size(l) !== u.nodeCount())
      throw new i();
    return c;
  }
  function i() {
  }
  return i.prototype = new Error(), Rv;
}
var Nv, tw;
function P3() {
  if (tw) return Nv;
  tw = 1;
  var t = LT();
  Nv = r;
  function r(i) {
    try {
      t(i);
    } catch (u) {
      if (u instanceof t.CycleException)
        return !1;
      throw u;
    }
    return !0;
  }
  return Nv;
}
var zv, nw;
function HT() {
  if (nw) return zv;
  nw = 1;
  var t = an();
  zv = r;
  function r(u, l, s) {
    t.isArray(l) || (l = [l]);
    var c = (u.isDirected() ? u.successors : u.neighbors).bind(u), f = [], g = {};
    return t.each(l, function(h) {
      if (!u.hasNode(h))
        throw new Error("Graph does not have node: " + h);
      i(u, h, s === "post", g, c, f);
    }), f;
  }
  function i(u, l, s, c, f, g) {
    t.has(c, l) || (c[l] = !0, s || g.push(l), t.each(f(l), function(h) {
      i(u, h, s, c, f, g);
    }), s && g.push(l));
  }
  return zv;
}
var Ov, rw;
function W3() {
  if (rw) return Ov;
  rw = 1;
  var t = HT();
  Ov = r;
  function r(i, u) {
    return t(i, u, "post");
  }
  return Ov;
}
var Dv, aw;
function ez() {
  if (aw) return Dv;
  aw = 1;
  var t = HT();
  Dv = r;
  function r(i, u) {
    return t(i, u, "pre");
  }
  return Dv;
}
var Lv, iw;
function tz() {
  if (iw) return Lv;
  iw = 1;
  var t = an(), r = Py(), i = zT();
  Lv = u;
  function u(l, s) {
    var c = new r(), f = {}, g = new i(), h;
    function v(m) {
      var b = m.v === h ? m.w : m.v, _ = g.priority(b);
      if (_ !== void 0) {
        var A = s(m);
        A < _ && (f[b] = h, g.decrease(b, A));
      }
    }
    if (l.nodeCount() === 0)
      return c;
    t.each(l.nodes(), function(m) {
      g.add(m, Number.POSITIVE_INFINITY), c.setNode(m);
    }), g.decrease(l.nodes()[0], 0);
    for (var y = !1; g.size() > 0; ) {
      if (h = g.removeMin(), t.has(f, h))
        c.setEdge(h, f[h]);
      else {
        if (y)
          throw new Error("Input graph is not connected: " + l);
        y = !0;
      }
      l.nodeEdges(h).forEach(v);
    }
    return c;
  }
  return Lv;
}
var Hv, uw;
function nz() {
  return uw || (uw = 1, Hv = {
    components: Q3(),
    dijkstra: OT(),
    dijkstraAll: $3(),
    findCycles: F3(),
    floydWarshall: J3(),
    isAcyclic: P3(),
    postorder: W3(),
    preorder: ez(),
    prim: tz(),
    tarjan: DT(),
    topsort: LT()
  }), Hv;
}
var jv, lw;
function rz() {
  if (lw) return jv;
  lw = 1;
  var t = Z3();
  return jv = {
    Graph: t.Graph,
    json: K3(),
    alg: nz(),
    version: t.version
  }, jv;
}
var Bv, ow;
function gn() {
  if (ow) return Bv;
  ow = 1;
  var t;
  if (typeof Hy == "function")
    try {
      t = rz();
    } catch {
    }
  return t || (t = window.graphlib), Bv = t, Bv;
}
var Uv, sw;
function az() {
  if (sw) return Uv;
  sw = 1;
  var t = lT(), r = 1, i = 4;
  function u(l) {
    return t(l, r | i);
  }
  return Uv = u, Uv;
}
var Gv, cw;
function ss() {
  if (cw) return Gv;
  cw = 1;
  var t = vi(), r = Wn(), i = es(), u = rn();
  function l(s, c, f) {
    if (!u(f))
      return !1;
    var g = typeof c;
    return (g == "number" ? r(f) && i(c, f.length) : g == "string" && c in f) ? t(f[c], s) : !1;
  }
  return Gv = l, Gv;
}
var Vv, fw;
function iz() {
  if (fw) return Vv;
  fw = 1;
  var t = os(), r = vi(), i = ss(), u = ha(), l = Object.prototype, s = l.hasOwnProperty, c = t(function(f, g) {
    f = Object(f);
    var h = -1, v = g.length, y = v > 2 ? g[2] : void 0;
    for (y && i(g[0], g[1], y) && (v = 1); ++h < v; )
      for (var m = g[h], b = u(m), _ = -1, A = b.length; ++_ < A; ) {
        var w = b[_], E = f[w];
        (E === void 0 || r(E, l[w]) && !s.call(f, w)) && (f[w] = m[w]);
      }
    return f;
  });
  return Vv = c, Vv;
}
var Yv, dw;
function uz() {
  if (dw) return Yv;
  dw = 1;
  var t = er(), r = Wn(), i = Lr();
  function u(l) {
    return function(s, c, f) {
      var g = Object(s);
      if (!r(s)) {
        var h = t(c, 3);
        s = i(s), c = function(y) {
          return h(g[y], y, g);
        };
      }
      var v = l(s, c, f);
      return v > -1 ? g[h ? s[v] : v] : void 0;
    };
  }
  return Yv = u, Yv;
}
var kv, hw;
function lz() {
  if (hw) return kv;
  hw = 1;
  var t = /\s/;
  function r(i) {
    for (var u = i.length; u-- && t.test(i.charAt(u)); )
      ;
    return u;
  }
  return kv = r, kv;
}
var Xv, gw;
function oz() {
  if (gw) return Xv;
  gw = 1;
  var t = lz(), r = /^\s+/;
  function i(u) {
    return u && u.slice(0, t(u) + 1).replace(r, "");
  }
  return Xv = i, Xv;
}
var Iv, vw;
function sz() {
  if (vw) return Iv;
  vw = 1;
  var t = oz(), r = rn(), i = bi(), u = NaN, l = /^[-+]0x[0-9a-f]+$/i, s = /^0b[01]+$/i, c = /^0o[0-7]+$/i, f = parseInt;
  function g(h) {
    if (typeof h == "number")
      return h;
    if (i(h))
      return u;
    if (r(h)) {
      var v = typeof h.valueOf == "function" ? h.valueOf() : h;
      h = r(v) ? v + "" : v;
    }
    if (typeof h != "string")
      return h === 0 ? h : +h;
    h = t(h);
    var y = s.test(h);
    return y || c.test(h) ? f(h.slice(2), y ? 2 : 8) : l.test(h) ? u : +h;
  }
  return Iv = g, Iv;
}
var Zv, pw;
function jT() {
  if (pw) return Zv;
  pw = 1;
  var t = sz(), r = 1 / 0, i = 17976931348623157e292;
  function u(l) {
    if (!l)
      return l === 0 ? l : 0;
    if (l = t(l), l === r || l === -r) {
      var s = l < 0 ? -1 : 1;
      return s * i;
    }
    return l === l ? l : 0;
  }
  return Zv = u, Zv;
}
var Kv, yw;
function cz() {
  if (yw) return Kv;
  yw = 1;
  var t = jT();
  function r(i) {
    var u = t(i), l = u % 1;
    return u === u ? l ? u - l : u : 0;
  }
  return Kv = r, Kv;
}
var Qv, mw;
function fz() {
  if (mw) return Qv;
  mw = 1;
  var t = qT(), r = er(), i = cz(), u = Math.max;
  function l(s, c, f) {
    var g = s == null ? 0 : s.length;
    if (!g)
      return -1;
    var h = f == null ? 0 : i(f);
    return h < 0 && (h = u(g + h, 0)), t(s, r(c, 3), h);
  }
  return Qv = l, Qv;
}
var $v, bw;
function dz() {
  if (bw) return $v;
  bw = 1;
  var t = uz(), r = fz(), i = t(r);
  return $v = i, $v;
}
var Fv, _w;
function BT() {
  if (_w) return Fv;
  _w = 1;
  var t = Jy();
  function r(i) {
    var u = i == null ? 0 : i.length;
    return u ? t(i, 1) : [];
  }
  return Fv = r, Fv;
}
var Jv, xw;
function hz() {
  if (xw) return Jv;
  xw = 1;
  var t = Ky(), r = oT(), i = ha();
  function u(l, s) {
    return l == null ? l : t(l, r(s), i);
  }
  return Jv = u, Jv;
}
var Pv, Sw;
function gz() {
  if (Sw) return Pv;
  Sw = 1;
  function t(r) {
    var i = r == null ? 0 : r.length;
    return i ? r[i - 1] : void 0;
  }
  return Pv = t, Pv;
}
var Wv, Ew;
function vz() {
  if (Ew) return Wv;
  Ew = 1;
  var t = Po(), r = Qy(), i = er();
  function u(l, s) {
    var c = {};
    return s = i(s, 3), r(l, function(f, g, h) {
      t(c, g, s(f, g, h));
    }), c;
  }
  return Wv = u, Wv;
}
var ep, ww;
function Wy() {
  if (ww) return ep;
  ww = 1;
  var t = bi();
  function r(i, u, l) {
    for (var s = -1, c = i.length; ++s < c; ) {
      var f = i[s], g = u(f);
      if (g != null && (h === void 0 ? g === g && !t(g) : l(g, h)))
        var h = g, v = f;
    }
    return v;
  }
  return ep = r, ep;
}
var tp, Aw;
function pz() {
  if (Aw) return tp;
  Aw = 1;
  function t(r, i) {
    return r > i;
  }
  return tp = t, tp;
}
var np, Tw;
function yz() {
  if (Tw) return np;
  Tw = 1;
  var t = Wy(), r = pz(), i = ga();
  function u(l) {
    return l && l.length ? t(l, i, r) : void 0;
  }
  return np = u, np;
}
var rp, Mw;
function UT() {
  if (Mw) return rp;
  Mw = 1;
  var t = Po(), r = vi();
  function i(u, l, s) {
    (s !== void 0 && !r(u[l], s) || s === void 0 && !(l in u)) && t(u, l, s);
  }
  return rp = i, rp;
}
var ap, Cw;
function mz() {
  if (Cw) return ap;
  Cw = 1;
  var t = fa(), r = rs(), i = Tn(), u = "[object Object]", l = Function.prototype, s = Object.prototype, c = l.toString, f = s.hasOwnProperty, g = c.call(Object);
  function h(v) {
    if (!i(v) || t(v) != u)
      return !1;
    var y = r(v);
    if (y === null)
      return !0;
    var m = f.call(y, "constructor") && y.constructor;
    return typeof m == "function" && m instanceof m && c.call(m) == g;
  }
  return ap = h, ap;
}
var ip, qw;
function GT() {
  if (qw) return ip;
  qw = 1;
  function t(r, i) {
    if (!(i === "constructor" && typeof r[i] == "function") && i != "__proto__")
      return r[i];
  }
  return ip = t, ip;
}
var up, Rw;
function bz() {
  if (Rw) return up;
  Rw = 1;
  var t = Uu(), r = ha();
  function i(u) {
    return t(u, r(u));
  }
  return up = i, up;
}
var lp, Nw;
function _z() {
  if (Nw) return lp;
  Nw = 1;
  var t = UT(), r = $A(), i = aT(), u = FA(), l = uT(), s = Gu(), c = st(), f = RT(), g = yi(), h = Bu(), v = rn(), y = mz(), m = Vu(), b = GT(), _ = bz();
  function A(w, E, M, S, T, N, R) {
    var q = b(w, M), L = b(E, M), j = R.get(L);
    if (j) {
      t(w, M, j);
      return;
    }
    var Z = N ? N(q, L, M + "", w, E, R) : void 0, Y = Z === void 0;
    if (Y) {
      var J = c(L), Q = !J && g(L), D = !J && !Q && m(L);
      Z = L, J || Q || D ? c(q) ? Z = q : f(q) ? Z = u(q) : Q ? (Y = !1, Z = r(L, !0)) : D ? (Y = !1, Z = i(L, !0)) : Z = [] : y(L) || s(L) ? (Z = q, s(q) ? Z = _(q) : (!v(q) || h(q)) && (Z = l(L))) : Y = !1;
    }
    Y && (R.set(L, Z), T(Z, L, S, N, R), R.delete(L)), t(w, M, Z);
  }
  return lp = A, lp;
}
var op, zw;
function xz() {
  if (zw) return op;
  zw = 1;
  var t = Jo(), r = UT(), i = Ky(), u = _z(), l = rn(), s = ha(), c = GT();
  function f(g, h, v, y, m) {
    g !== h && i(h, function(b, _) {
      if (m || (m = new t()), l(b))
        u(g, h, _, v, f, y, m);
      else {
        var A = y ? y(c(g, _), b, _ + "", g, h, m) : void 0;
        A === void 0 && (A = b), r(g, _, A);
      }
    }, s);
  }
  return op = f, op;
}
var sp, Ow;
function Sz() {
  if (Ow) return sp;
  Ow = 1;
  var t = os(), r = ss();
  function i(u) {
    return t(function(l, s) {
      var c = -1, f = s.length, g = f > 1 ? s[f - 1] : void 0, h = f > 2 ? s[2] : void 0;
      for (g = u.length > 3 && typeof g == "function" ? (f--, g) : void 0, h && r(s[0], s[1], h) && (g = f < 3 ? void 0 : g, f = 1), l = Object(l); ++c < f; ) {
        var v = s[c];
        v && u(l, v, c, g);
      }
      return l;
    });
  }
  return sp = i, sp;
}
var cp, Dw;
function Ez() {
  if (Dw) return cp;
  Dw = 1;
  var t = xz(), r = Sz(), i = r(function(u, l, s) {
    t(u, l, s);
  });
  return cp = i, cp;
}
var fp, Lw;
function VT() {
  if (Lw) return fp;
  Lw = 1;
  function t(r, i) {
    return r < i;
  }
  return fp = t, fp;
}
var dp, Hw;
function wz() {
  if (Hw) return dp;
  Hw = 1;
  var t = Wy(), r = VT(), i = ga();
  function u(l) {
    return l && l.length ? t(l, i, r) : void 0;
  }
  return dp = u, dp;
}
var hp, jw;
function Az() {
  if (jw) return hp;
  jw = 1;
  var t = Wy(), r = er(), i = VT();
  function u(l, s) {
    return l && l.length ? t(l, r(s, 2), i) : void 0;
  }
  return hp = u, hp;
}
var gp, Bw;
function Tz() {
  if (Bw) return gp;
  Bw = 1;
  var t = pn(), r = function() {
    return t.Date.now();
  };
  return gp = r, gp;
}
var vp, Uw;
function Mz() {
  if (Uw) return vp;
  Uw = 1;
  var t = Wo(), r = us(), i = es(), u = rn(), l = Yu();
  function s(c, f, g, h) {
    if (!u(c))
      return c;
    f = r(f, c);
    for (var v = -1, y = f.length, m = y - 1, b = c; b != null && ++v < y; ) {
      var _ = l(f[v]), A = g;
      if (_ === "__proto__" || _ === "constructor" || _ === "prototype")
        return c;
      if (v != m) {
        var w = b[_];
        A = h ? h(w, _, b) : void 0, A === void 0 && (A = u(w) ? w : i(f[v + 1]) ? [] : {});
      }
      t(b, _, A), b = b[_];
    }
    return c;
  }
  return vp = s, vp;
}
var pp, Gw;
function Cz() {
  if (Gw) return pp;
  Gw = 1;
  var t = ls(), r = Mz(), i = us();
  function u(l, s, c) {
    for (var f = -1, g = s.length, h = {}; ++f < g; ) {
      var v = s[f], y = t(l, v);
      c(y, v) && r(h, i(v, l), y);
    }
    return h;
  }
  return pp = u, pp;
}
var yp, Vw;
function qz() {
  if (Vw) return yp;
  Vw = 1;
  var t = Cz(), r = bT();
  function i(u, l) {
    return t(u, l, function(s, c) {
      return r(u, c);
    });
  }
  return yp = i, yp;
}
var mp, Yw;
function Rz() {
  if (Yw) return mp;
  Yw = 1;
  var t = BT(), r = MT(), i = CT();
  function u(l) {
    return i(r(l, void 0, t), l + "");
  }
  return mp = u, mp;
}
var bp, kw;
function Nz() {
  if (kw) return bp;
  kw = 1;
  var t = qz(), r = Rz(), i = r(function(u, l) {
    return u == null ? {} : t(u, l);
  });
  return bp = i, bp;
}
var _p, Xw;
function zz() {
  if (Xw) return _p;
  Xw = 1;
  var t = Math.ceil, r = Math.max;
  function i(u, l, s, c) {
    for (var f = -1, g = r(t((l - u) / (s || 1)), 0), h = Array(g); g--; )
      h[c ? g : ++f] = u, u += s;
    return h;
  }
  return _p = i, _p;
}
var xp, Iw;
function Oz() {
  if (Iw) return xp;
  Iw = 1;
  var t = zz(), r = ss(), i = jT();
  function u(l) {
    return function(s, c, f) {
      return f && typeof f != "number" && r(s, c, f) && (c = f = void 0), s = i(s), c === void 0 ? (c = s, s = 0) : c = i(c), f = f === void 0 ? s < c ? 1 : -1 : i(f), t(s, c, f, l);
    };
  }
  return xp = u, xp;
}
var Sp, Zw;
function Dz() {
  if (Zw) return Sp;
  Zw = 1;
  var t = Oz(), r = t();
  return Sp = r, Sp;
}
var Ep, Kw;
function Lz() {
  if (Kw) return Ep;
  Kw = 1;
  function t(r, i) {
    var u = r.length;
    for (r.sort(i); u--; )
      r[u] = r[u].value;
    return r;
  }
  return Ep = t, Ep;
}
var wp, Qw;
function Hz() {
  if (Qw) return wp;
  Qw = 1;
  var t = bi();
  function r(i, u) {
    if (i !== u) {
      var l = i !== void 0, s = i === null, c = i === i, f = t(i), g = u !== void 0, h = u === null, v = u === u, y = t(u);
      if (!h && !y && !f && i > u || f && g && v && !h && !y || s && g && v || !l && v || !c)
        return 1;
      if (!s && !f && !y && i < u || y && l && c && !s && !f || h && l && c || !g && c || !v)
        return -1;
    }
    return 0;
  }
  return wp = r, wp;
}
var Ap, $w;
function jz() {
  if ($w) return Ap;
  $w = 1;
  var t = Hz();
  function r(i, u, l) {
    for (var s = -1, c = i.criteria, f = u.criteria, g = c.length, h = l.length; ++s < g; ) {
      var v = t(c[s], f[s]);
      if (v) {
        if (s >= h)
          return v;
        var y = l[s];
        return v * (y == "desc" ? -1 : 1);
      }
    }
    return i.index - u.index;
  }
  return Ap = r, Ap;
}
var Tp, Fw;
function Bz() {
  if (Fw) return Tp;
  Fw = 1;
  var t = is(), r = ls(), i = er(), u = wT(), l = Lz(), s = ts(), c = jz(), f = ga(), g = st();
  function h(v, y, m) {
    y.length ? y = t(y, function(A) {
      return g(A) ? function(w) {
        return r(w, A.length === 1 ? A[0] : A);
      } : A;
    }) : y = [f];
    var b = -1;
    y = t(y, s(i));
    var _ = u(v, function(A, w, E) {
      var M = t(y, function(S) {
        return S(A);
      });
      return { criteria: M, index: ++b, value: A };
    });
    return l(_, function(A, w) {
      return c(A, w, m);
    });
  }
  return Tp = h, Tp;
}
var Mp, Jw;
function Uz() {
  if (Jw) return Mp;
  Jw = 1;
  var t = Jy(), r = Bz(), i = os(), u = ss(), l = i(function(s, c) {
    if (s == null)
      return [];
    var f = c.length;
    return f > 1 && u(s, c[0], c[1]) ? c = [] : f > 2 && u(c[0], c[1], c[2]) && (c = [c[0]]), r(s, t(c, 1), []);
  });
  return Mp = l, Mp;
}
var Cp, Pw;
function Gz() {
  if (Pw) return Cp;
  Pw = 1;
  var t = yT(), r = 0;
  function i(u) {
    var l = ++r;
    return t(u) + l;
  }
  return Cp = i, Cp;
}
var qp, Ww;
function Vz() {
  if (Ww) return qp;
  Ww = 1;
  function t(r, i, u) {
    for (var l = -1, s = r.length, c = i.length, f = {}; ++l < s; ) {
      var g = l < c ? i[l] : void 0;
      u(f, r[l], g);
    }
    return f;
  }
  return qp = t, qp;
}
var Rp, e2;
function Yz() {
  if (e2) return Rp;
  e2 = 1;
  var t = Wo(), r = Vz();
  function i(u, l) {
    return r(u || [], l || [], t);
  }
  return Rp = i, Rp;
}
var Np, t2;
function Fe() {
  if (t2) return Np;
  t2 = 1;
  var t;
  if (typeof Hy == "function")
    try {
      t = {
        cloneDeep: az(),
        constant: Zy(),
        defaults: iz(),
        each: cT(),
        filter: xT(),
        find: dz(),
        flatten: BT(),
        forEach: sT(),
        forIn: hz(),
        has: ST(),
        isUndefined: ET(),
        last: gz(),
        map: AT(),
        mapValues: vz(),
        max: yz(),
        merge: Ez(),
        min: wz(),
        minBy: Az(),
        now: Tz(),
        pick: Nz(),
        range: Dz(),
        reduce: TT(),
        sortBy: Uz(),
        uniqueId: Gz(),
        values: NT(),
        zipObject: Yz()
      };
    } catch {
    }
  return t || (t = window._), Np = t, Np;
}
var zp, n2;
function kz() {
  if (n2) return zp;
  n2 = 1, zp = t;
  function t() {
    var u = {};
    u._next = u._prev = u, this._sentinel = u;
  }
  t.prototype.dequeue = function() {
    var u = this._sentinel, l = u._prev;
    if (l !== u)
      return r(l), l;
  }, t.prototype.enqueue = function(u) {
    var l = this._sentinel;
    u._prev && u._next && r(u), u._next = l._next, l._next._prev = u, l._next = u, u._prev = l;
  }, t.prototype.toString = function() {
    for (var u = [], l = this._sentinel, s = l._prev; s !== l; )
      u.push(JSON.stringify(s, i)), s = s._prev;
    return "[" + u.join(", ") + "]";
  };
  function r(u) {
    u._prev._next = u._next, u._next._prev = u._prev, delete u._next, delete u._prev;
  }
  function i(u, l) {
    if (u !== "_next" && u !== "_prev")
      return l;
  }
  return zp;
}
var Op, r2;
function Xz() {
  if (r2) return Op;
  r2 = 1;
  var t = Fe(), r = gn().Graph, i = kz();
  Op = l;
  var u = t.constant(1);
  function l(h, v) {
    if (h.nodeCount() <= 1)
      return [];
    var y = f(h, v || u), m = s(y.graph, y.buckets, y.zeroIdx);
    return t.flatten(t.map(m, function(b) {
      return h.outEdges(b.v, b.w);
    }), !0);
  }
  function s(h, v, y) {
    for (var m = [], b = v[v.length - 1], _ = v[0], A; h.nodeCount(); ) {
      for (; A = _.dequeue(); )
        c(h, v, y, A);
      for (; A = b.dequeue(); )
        c(h, v, y, A);
      if (h.nodeCount()) {
        for (var w = v.length - 2; w > 0; --w)
          if (A = v[w].dequeue(), A) {
            m = m.concat(c(h, v, y, A, !0));
            break;
          }
      }
    }
    return m;
  }
  function c(h, v, y, m, b) {
    var _ = b ? [] : void 0;
    return t.forEach(h.inEdges(m.v), function(A) {
      var w = h.edge(A), E = h.node(A.v);
      b && _.push({ v: A.v, w: A.w }), E.out -= w, g(v, y, E);
    }), t.forEach(h.outEdges(m.v), function(A) {
      var w = h.edge(A), E = A.w, M = h.node(E);
      M.in -= w, g(v, y, M);
    }), h.removeNode(m.v), _;
  }
  function f(h, v) {
    var y = new r(), m = 0, b = 0;
    t.forEach(h.nodes(), function(w) {
      y.setNode(w, { v: w, in: 0, out: 0 });
    }), t.forEach(h.edges(), function(w) {
      var E = y.edge(w.v, w.w) || 0, M = v(w), S = E + M;
      y.setEdge(w.v, w.w, S), b = Math.max(b, y.node(w.v).out += M), m = Math.max(m, y.node(w.w).in += M);
    });
    var _ = t.range(b + m + 3).map(function() {
      return new i();
    }), A = m + 1;
    return t.forEach(y.nodes(), function(w) {
      g(_, A, y.node(w));
    }), { graph: y, buckets: _, zeroIdx: A };
  }
  function g(h, v, y) {
    y.out ? y.in ? h[y.out - y.in + v].enqueue(y) : h[h.length - 1].enqueue(y) : h[0].enqueue(y);
  }
  return Op;
}
var Dp, a2;
function Iz() {
  if (a2) return Dp;
  a2 = 1;
  var t = Fe(), r = Xz();
  Dp = {
    run: i,
    undo: l
  };
  function i(s) {
    var c = s.graph().acyclicer === "greedy" ? r(s, f(s)) : u(s);
    t.forEach(c, function(g) {
      var h = s.edge(g);
      s.removeEdge(g), h.forwardName = g.name, h.reversed = !0, s.setEdge(g.w, g.v, h, t.uniqueId("rev"));
    });
    function f(g) {
      return function(h) {
        return g.edge(h).weight;
      };
    }
  }
  function u(s) {
    var c = [], f = {}, g = {};
    function h(v) {
      t.has(g, v) || (g[v] = !0, f[v] = !0, t.forEach(s.outEdges(v), function(y) {
        t.has(f, y.w) ? c.push(y) : h(y.w);
      }), delete f[v]);
    }
    return t.forEach(s.nodes(), h), c;
  }
  function l(s) {
    t.forEach(s.edges(), function(c) {
      var f = s.edge(c);
      if (f.reversed) {
        s.removeEdge(c);
        var g = f.forwardName;
        delete f.reversed, delete f.forwardName, s.setEdge(c.w, c.v, f, g);
      }
    });
  }
  return Dp;
}
var Lp, i2;
function Ut() {
  if (i2) return Lp;
  i2 = 1;
  var t = Fe(), r = gn().Graph;
  Lp = {
    addDummyNode: i,
    simplify: u,
    asNonCompoundGraph: l,
    successorWeights: s,
    predecessorWeights: c,
    intersectRect: f,
    buildLayerMatrix: g,
    normalizeRanks: h,
    removeEmptyRanks: v,
    addBorderNode: y,
    maxRank: m,
    partition: b,
    time: _,
    notime: A
  };
  function i(w, E, M, S) {
    var T;
    do
      T = t.uniqueId(S);
    while (w.hasNode(T));
    return M.dummy = E, w.setNode(T, M), T;
  }
  function u(w) {
    var E = new r().setGraph(w.graph());
    return t.forEach(w.nodes(), function(M) {
      E.setNode(M, w.node(M));
    }), t.forEach(w.edges(), function(M) {
      var S = E.edge(M.v, M.w) || { weight: 0, minlen: 1 }, T = w.edge(M);
      E.setEdge(M.v, M.w, {
        weight: S.weight + T.weight,
        minlen: Math.max(S.minlen, T.minlen)
      });
    }), E;
  }
  function l(w) {
    var E = new r({ multigraph: w.isMultigraph() }).setGraph(w.graph());
    return t.forEach(w.nodes(), function(M) {
      w.children(M).length || E.setNode(M, w.node(M));
    }), t.forEach(w.edges(), function(M) {
      E.setEdge(M, w.edge(M));
    }), E;
  }
  function s(w) {
    var E = t.map(w.nodes(), function(M) {
      var S = {};
      return t.forEach(w.outEdges(M), function(T) {
        S[T.w] = (S[T.w] || 0) + w.edge(T).weight;
      }), S;
    });
    return t.zipObject(w.nodes(), E);
  }
  function c(w) {
    var E = t.map(w.nodes(), function(M) {
      var S = {};
      return t.forEach(w.inEdges(M), function(T) {
        S[T.v] = (S[T.v] || 0) + w.edge(T).weight;
      }), S;
    });
    return t.zipObject(w.nodes(), E);
  }
  function f(w, E) {
    var M = w.x, S = w.y, T = E.x - M, N = E.y - S, R = w.width / 2, q = w.height / 2;
    if (!T && !N)
      throw new Error("Not possible to find intersection inside of the rectangle");
    var L, j;
    return Math.abs(N) * R > Math.abs(T) * q ? (N < 0 && (q = -q), L = q * T / N, j = q) : (T < 0 && (R = -R), L = R, j = R * N / T), { x: M + L, y: S + j };
  }
  function g(w) {
    var E = t.map(t.range(m(w) + 1), function() {
      return [];
    });
    return t.forEach(w.nodes(), function(M) {
      var S = w.node(M), T = S.rank;
      t.isUndefined(T) || (E[T][S.order] = M);
    }), E;
  }
  function h(w) {
    var E = t.min(t.map(w.nodes(), function(M) {
      return w.node(M).rank;
    }));
    t.forEach(w.nodes(), function(M) {
      var S = w.node(M);
      t.has(S, "rank") && (S.rank -= E);
    });
  }
  function v(w) {
    var E = t.min(t.map(w.nodes(), function(N) {
      return w.node(N).rank;
    })), M = [];
    t.forEach(w.nodes(), function(N) {
      var R = w.node(N).rank - E;
      M[R] || (M[R] = []), M[R].push(N);
    });
    var S = 0, T = w.graph().nodeRankFactor;
    t.forEach(M, function(N, R) {
      t.isUndefined(N) && R % T !== 0 ? --S : S && t.forEach(N, function(q) {
        w.node(q).rank += S;
      });
    });
  }
  function y(w, E, M, S) {
    var T = {
      width: 0,
      height: 0
    };
    return arguments.length >= 4 && (T.rank = M, T.order = S), i(w, "border", T, E);
  }
  function m(w) {
    return t.max(t.map(w.nodes(), function(E) {
      var M = w.node(E).rank;
      if (!t.isUndefined(M))
        return M;
    }));
  }
  function b(w, E) {
    var M = { lhs: [], rhs: [] };
    return t.forEach(w, function(S) {
      E(S) ? M.lhs.push(S) : M.rhs.push(S);
    }), M;
  }
  function _(w, E) {
    var M = t.now();
    try {
      return E();
    } finally {
      console.log(w + " time: " + (t.now() - M) + "ms");
    }
  }
  function A(w, E) {
    return E();
  }
  return Lp;
}
var Hp, u2;
function Zz() {
  if (u2) return Hp;
  u2 = 1;
  var t = Fe(), r = Ut();
  Hp = {
    run: i,
    undo: l
  };
  function i(s) {
    s.graph().dummyChains = [], t.forEach(s.edges(), function(c) {
      u(s, c);
    });
  }
  function u(s, c) {
    var f = c.v, g = s.node(f).rank, h = c.w, v = s.node(h).rank, y = c.name, m = s.edge(c), b = m.labelRank;
    if (v !== g + 1) {
      s.removeEdge(c);
      var _, A, w;
      for (w = 0, ++g; g < v; ++w, ++g)
        m.points = [], A = {
          width: 0,
          height: 0,
          edgeLabel: m,
          edgeObj: c,
          rank: g
        }, _ = r.addDummyNode(s, "edge", A, "_d"), g === b && (A.width = m.width, A.height = m.height, A.dummy = "edge-label", A.labelpos = m.labelpos), s.setEdge(f, _, { weight: m.weight }, y), w === 0 && s.graph().dummyChains.push(_), f = _;
      s.setEdge(f, h, { weight: m.weight }, y);
    }
  }
  function l(s) {
    t.forEach(s.graph().dummyChains, function(c) {
      var f = s.node(c), g = f.edgeLabel, h;
      for (s.setEdge(f.edgeObj, g); f.dummy; )
        h = s.successors(c)[0], s.removeNode(c), g.points.push({ x: f.x, y: f.y }), f.dummy === "edge-label" && (g.x = f.x, g.y = f.y, g.width = f.width, g.height = f.height), c = h, f = s.node(c);
    });
  }
  return Hp;
}
var jp, l2;
function Lo() {
  if (l2) return jp;
  l2 = 1;
  var t = Fe();
  jp = {
    longestPath: r,
    slack: i
  };
  function r(u) {
    var l = {};
    function s(c) {
      var f = u.node(c);
      if (t.has(l, c))
        return f.rank;
      l[c] = !0;
      var g = t.min(t.map(u.outEdges(c), function(h) {
        return s(h.w) - u.edge(h).minlen;
      }));
      return (g === Number.POSITIVE_INFINITY || // return value of _.map([]) for Lodash 3
      g === void 0 || // return value of _.map([]) for Lodash 4
      g === null) && (g = 0), f.rank = g;
    }
    t.forEach(u.sources(), s);
  }
  function i(u, l) {
    return u.node(l.w).rank - u.node(l.v).rank - u.edge(l).minlen;
  }
  return jp;
}
var Bp, o2;
function YT() {
  if (o2) return Bp;
  o2 = 1;
  var t = Fe(), r = gn().Graph, i = Lo().slack;
  Bp = u;
  function u(f) {
    var g = new r({ directed: !1 }), h = f.nodes()[0], v = f.nodeCount();
    g.setNode(h, {});
    for (var y, m; l(g, f) < v; )
      y = s(g, f), m = g.hasNode(y.v) ? i(f, y) : -i(f, y), c(g, f, m);
    return g;
  }
  function l(f, g) {
    function h(v) {
      t.forEach(g.nodeEdges(v), function(y) {
        var m = y.v, b = v === m ? y.w : m;
        !f.hasNode(b) && !i(g, y) && (f.setNode(b, {}), f.setEdge(v, b, {}), h(b));
      });
    }
    return t.forEach(f.nodes(), h), f.nodeCount();
  }
  function s(f, g) {
    return t.minBy(g.edges(), function(h) {
      if (f.hasNode(h.v) !== f.hasNode(h.w))
        return i(g, h);
    });
  }
  function c(f, g, h) {
    t.forEach(f.nodes(), function(v) {
      g.node(v).rank += h;
    });
  }
  return Bp;
}
var Up, s2;
function Kz() {
  if (s2) return Up;
  s2 = 1;
  var t = Fe(), r = YT(), i = Lo().slack, u = Lo().longestPath, l = gn().alg.preorder, s = gn().alg.postorder, c = Ut().simplify;
  Up = f, f.initLowLimValues = y, f.initCutValues = g, f.calcCutValue = v, f.leaveEdge = b, f.enterEdge = _, f.exchangeEdges = A;
  function f(S) {
    S = c(S), u(S);
    var T = r(S);
    y(T), g(T, S);
    for (var N, R; N = b(T); )
      R = _(T, S, N), A(T, S, N, R);
  }
  function g(S, T) {
    var N = s(S, S.nodes());
    N = N.slice(0, N.length - 1), t.forEach(N, function(R) {
      h(S, T, R);
    });
  }
  function h(S, T, N) {
    var R = S.node(N), q = R.parent;
    S.edge(N, q).cutvalue = v(S, T, N);
  }
  function v(S, T, N) {
    var R = S.node(N), q = R.parent, L = !0, j = T.edge(N, q), Z = 0;
    return j || (L = !1, j = T.edge(q, N)), Z = j.weight, t.forEach(T.nodeEdges(N), function(Y) {
      var J = Y.v === N, Q = J ? Y.w : Y.v;
      if (Q !== q) {
        var D = J === L, X = T.edge(Y).weight;
        if (Z += D ? X : -X, E(S, N, Q)) {
          var z = S.edge(N, Q).cutvalue;
          Z += D ? -z : z;
        }
      }
    }), Z;
  }
  function y(S, T) {
    arguments.length < 2 && (T = S.nodes()[0]), m(S, {}, 1, T);
  }
  function m(S, T, N, R, q) {
    var L = N, j = S.node(R);
    return T[R] = !0, t.forEach(S.neighbors(R), function(Z) {
      t.has(T, Z) || (N = m(S, T, N, Z, R));
    }), j.low = L, j.lim = N++, q ? j.parent = q : delete j.parent, N;
  }
  function b(S) {
    return t.find(S.edges(), function(T) {
      return S.edge(T).cutvalue < 0;
    });
  }
  function _(S, T, N) {
    var R = N.v, q = N.w;
    T.hasEdge(R, q) || (R = N.w, q = N.v);
    var L = S.node(R), j = S.node(q), Z = L, Y = !1;
    L.lim > j.lim && (Z = j, Y = !0);
    var J = t.filter(T.edges(), function(Q) {
      return Y === M(S, S.node(Q.v), Z) && Y !== M(S, S.node(Q.w), Z);
    });
    return t.minBy(J, function(Q) {
      return i(T, Q);
    });
  }
  function A(S, T, N, R) {
    var q = N.v, L = N.w;
    S.removeEdge(q, L), S.setEdge(R.v, R.w, {}), y(S), g(S, T), w(S, T);
  }
  function w(S, T) {
    var N = t.find(S.nodes(), function(q) {
      return !T.node(q).parent;
    }), R = l(S, N);
    R = R.slice(1), t.forEach(R, function(q) {
      var L = S.node(q).parent, j = T.edge(q, L), Z = !1;
      j || (j = T.edge(L, q), Z = !0), T.node(q).rank = T.node(L).rank + (Z ? j.minlen : -j.minlen);
    });
  }
  function E(S, T, N) {
    return S.hasEdge(T, N);
  }
  function M(S, T, N) {
    return N.low <= T.lim && T.lim <= N.lim;
  }
  return Up;
}
var Gp, c2;
function Qz() {
  if (c2) return Gp;
  c2 = 1;
  var t = Lo(), r = t.longestPath, i = YT(), u = Kz();
  Gp = l;
  function l(g) {
    switch (g.graph().ranker) {
      case "network-simplex":
        f(g);
        break;
      case "tight-tree":
        c(g);
        break;
      case "longest-path":
        s(g);
        break;
      default:
        f(g);
    }
  }
  var s = r;
  function c(g) {
    r(g), i(g);
  }
  function f(g) {
    u(g);
  }
  return Gp;
}
var Vp, f2;
function $z() {
  if (f2) return Vp;
  f2 = 1;
  var t = Fe();
  Vp = r;
  function r(l) {
    var s = u(l);
    t.forEach(l.graph().dummyChains, function(c) {
      for (var f = l.node(c), g = f.edgeObj, h = i(l, s, g.v, g.w), v = h.path, y = h.lca, m = 0, b = v[m], _ = !0; c !== g.w; ) {
        if (f = l.node(c), _) {
          for (; (b = v[m]) !== y && l.node(b).maxRank < f.rank; )
            m++;
          b === y && (_ = !1);
        }
        if (!_) {
          for (; m < v.length - 1 && l.node(b = v[m + 1]).minRank <= f.rank; )
            m++;
          b = v[m];
        }
        l.setParent(c, b), c = l.successors(c)[0];
      }
    });
  }
  function i(l, s, c, f) {
    var g = [], h = [], v = Math.min(s[c].low, s[f].low), y = Math.max(s[c].lim, s[f].lim), m, b;
    m = c;
    do
      m = l.parent(m), g.push(m);
    while (m && (s[m].low > v || y > s[m].lim));
    for (b = m, m = f; (m = l.parent(m)) !== b; )
      h.push(m);
    return { path: g.concat(h.reverse()), lca: b };
  }
  function u(l) {
    var s = {}, c = 0;
    function f(g) {
      var h = c;
      t.forEach(l.children(g), f), s[g] = { low: h, lim: c++ };
    }
    return t.forEach(l.children(), f), s;
  }
  return Vp;
}
var Yp, d2;
function Fz() {
  if (d2) return Yp;
  d2 = 1;
  var t = Fe(), r = Ut();
  Yp = {
    run: i,
    cleanup: c
  };
  function i(f) {
    var g = r.addDummyNode(f, "root", {}, "_root"), h = l(f), v = t.max(t.values(h)) - 1, y = 2 * v + 1;
    f.graph().nestingRoot = g, t.forEach(f.edges(), function(b) {
      f.edge(b).minlen *= y;
    });
    var m = s(f) + 1;
    t.forEach(f.children(), function(b) {
      u(f, g, y, m, v, h, b);
    }), f.graph().nodeRankFactor = y;
  }
  function u(f, g, h, v, y, m, b) {
    var _ = f.children(b);
    if (!_.length) {
      b !== g && f.setEdge(g, b, { weight: 0, minlen: h });
      return;
    }
    var A = r.addBorderNode(f, "_bt"), w = r.addBorderNode(f, "_bb"), E = f.node(b);
    f.setParent(A, b), E.borderTop = A, f.setParent(w, b), E.borderBottom = w, t.forEach(_, function(M) {
      u(f, g, h, v, y, m, M);
      var S = f.node(M), T = S.borderTop ? S.borderTop : M, N = S.borderBottom ? S.borderBottom : M, R = S.borderTop ? v : 2 * v, q = T !== N ? 1 : y - m[b] + 1;
      f.setEdge(A, T, {
        weight: R,
        minlen: q,
        nestingEdge: !0
      }), f.setEdge(N, w, {
        weight: R,
        minlen: q,
        nestingEdge: !0
      });
    }), f.parent(b) || f.setEdge(g, A, { weight: 0, minlen: y + m[b] });
  }
  function l(f) {
    var g = {};
    function h(v, y) {
      var m = f.children(v);
      m && m.length && t.forEach(m, function(b) {
        h(b, y + 1);
      }), g[v] = y;
    }
    return t.forEach(f.children(), function(v) {
      h(v, 1);
    }), g;
  }
  function s(f) {
    return t.reduce(f.edges(), function(g, h) {
      return g + f.edge(h).weight;
    }, 0);
  }
  function c(f) {
    var g = f.graph();
    f.removeNode(g.nestingRoot), delete g.nestingRoot, t.forEach(f.edges(), function(h) {
      var v = f.edge(h);
      v.nestingEdge && f.removeEdge(h);
    });
  }
  return Yp;
}
var kp, h2;
function Jz() {
  if (h2) return kp;
  h2 = 1;
  var t = Fe(), r = Ut();
  kp = i;
  function i(l) {
    function s(c) {
      var f = l.children(c), g = l.node(c);
      if (f.length && t.forEach(f, s), t.has(g, "minRank")) {
        g.borderLeft = [], g.borderRight = [];
        for (var h = g.minRank, v = g.maxRank + 1; h < v; ++h)
          u(l, "borderLeft", "_bl", c, g, h), u(l, "borderRight", "_br", c, g, h);
      }
    }
    t.forEach(l.children(), s);
  }
  function u(l, s, c, f, g, h) {
    var v = { width: 0, height: 0, rank: h, borderType: s }, y = g[s][h - 1], m = r.addDummyNode(l, "border", v, c);
    g[s][h] = m, l.setParent(m, f), y && l.setEdge(y, m, { weight: 1 });
  }
  return kp;
}
var Xp, g2;
function Pz() {
  if (g2) return Xp;
  g2 = 1;
  var t = Fe();
  Xp = {
    adjust: r,
    undo: i
  };
  function r(h) {
    var v = h.graph().rankdir.toLowerCase();
    (v === "lr" || v === "rl") && u(h);
  }
  function i(h) {
    var v = h.graph().rankdir.toLowerCase();
    (v === "bt" || v === "rl") && s(h), (v === "lr" || v === "rl") && (f(h), u(h));
  }
  function u(h) {
    t.forEach(h.nodes(), function(v) {
      l(h.node(v));
    }), t.forEach(h.edges(), function(v) {
      l(h.edge(v));
    });
  }
  function l(h) {
    var v = h.width;
    h.width = h.height, h.height = v;
  }
  function s(h) {
    t.forEach(h.nodes(), function(v) {
      c(h.node(v));
    }), t.forEach(h.edges(), function(v) {
      var y = h.edge(v);
      t.forEach(y.points, c), t.has(y, "y") && c(y);
    });
  }
  function c(h) {
    h.y = -h.y;
  }
  function f(h) {
    t.forEach(h.nodes(), function(v) {
      g(h.node(v));
    }), t.forEach(h.edges(), function(v) {
      var y = h.edge(v);
      t.forEach(y.points, g), t.has(y, "x") && g(y);
    });
  }
  function g(h) {
    var v = h.x;
    h.x = h.y, h.y = v;
  }
  return Xp;
}
var Ip, v2;
function Wz() {
  if (v2) return Ip;
  v2 = 1;
  var t = Fe();
  Ip = r;
  function r(i) {
    var u = {}, l = t.filter(i.nodes(), function(h) {
      return !i.children(h).length;
    }), s = t.max(t.map(l, function(h) {
      return i.node(h).rank;
    })), c = t.map(t.range(s + 1), function() {
      return [];
    });
    function f(h) {
      if (!t.has(u, h)) {
        u[h] = !0;
        var v = i.node(h);
        c[v.rank].push(h), t.forEach(i.successors(h), f);
      }
    }
    var g = t.sortBy(l, function(h) {
      return i.node(h).rank;
    });
    return t.forEach(g, f), c;
  }
  return Ip;
}
var Zp, p2;
function eO() {
  if (p2) return Zp;
  p2 = 1;
  var t = Fe();
  Zp = r;
  function r(u, l) {
    for (var s = 0, c = 1; c < l.length; ++c)
      s += i(u, l[c - 1], l[c]);
    return s;
  }
  function i(u, l, s) {
    for (var c = t.zipObject(
      s,
      t.map(s, function(m, b) {
        return b;
      })
    ), f = t.flatten(t.map(l, function(m) {
      return t.sortBy(t.map(u.outEdges(m), function(b) {
        return { pos: c[b.w], weight: u.edge(b).weight };
      }), "pos");
    }), !0), g = 1; g < s.length; ) g <<= 1;
    var h = 2 * g - 1;
    g -= 1;
    var v = t.map(new Array(h), function() {
      return 0;
    }), y = 0;
    return t.forEach(f.forEach(function(m) {
      var b = m.pos + g;
      v[b] += m.weight;
      for (var _ = 0; b > 0; )
        b % 2 && (_ += v[b + 1]), b = b - 1 >> 1, v[b] += m.weight;
      y += m.weight * _;
    })), y;
  }
  return Zp;
}
var Kp, y2;
function tO() {
  if (y2) return Kp;
  y2 = 1;
  var t = Fe();
  Kp = r;
  function r(i, u) {
    return t.map(u, function(l) {
      var s = i.inEdges(l);
      if (s.length) {
        var c = t.reduce(s, function(f, g) {
          var h = i.edge(g), v = i.node(g.v);
          return {
            sum: f.sum + h.weight * v.order,
            weight: f.weight + h.weight
          };
        }, { sum: 0, weight: 0 });
        return {
          v: l,
          barycenter: c.sum / c.weight,
          weight: c.weight
        };
      } else
        return { v: l };
    });
  }
  return Kp;
}
var Qp, m2;
function nO() {
  if (m2) return Qp;
  m2 = 1;
  var t = Fe();
  Qp = r;
  function r(l, s) {
    var c = {};
    t.forEach(l, function(g, h) {
      var v = c[g.v] = {
        indegree: 0,
        in: [],
        out: [],
        vs: [g.v],
        i: h
      };
      t.isUndefined(g.barycenter) || (v.barycenter = g.barycenter, v.weight = g.weight);
    }), t.forEach(s.edges(), function(g) {
      var h = c[g.v], v = c[g.w];
      !t.isUndefined(h) && !t.isUndefined(v) && (v.indegree++, h.out.push(c[g.w]));
    });
    var f = t.filter(c, function(g) {
      return !g.indegree;
    });
    return i(f);
  }
  function i(l) {
    var s = [];
    function c(h) {
      return function(v) {
        v.merged || (t.isUndefined(v.barycenter) || t.isUndefined(h.barycenter) || v.barycenter >= h.barycenter) && u(h, v);
      };
    }
    function f(h) {
      return function(v) {
        v.in.push(h), --v.indegree === 0 && l.push(v);
      };
    }
    for (; l.length; ) {
      var g = l.pop();
      s.push(g), t.forEach(g.in.reverse(), c(g)), t.forEach(g.out, f(g));
    }
    return t.map(
      t.filter(s, function(h) {
        return !h.merged;
      }),
      function(h) {
        return t.pick(h, ["vs", "i", "barycenter", "weight"]);
      }
    );
  }
  function u(l, s) {
    var c = 0, f = 0;
    l.weight && (c += l.barycenter * l.weight, f += l.weight), s.weight && (c += s.barycenter * s.weight, f += s.weight), l.vs = s.vs.concat(l.vs), l.barycenter = c / f, l.weight = f, l.i = Math.min(s.i, l.i), s.merged = !0;
  }
  return Qp;
}
var $p, b2;
function rO() {
  if (b2) return $p;
  b2 = 1;
  var t = Fe(), r = Ut();
  $p = i;
  function i(s, c) {
    var f = r.partition(s, function(A) {
      return t.has(A, "barycenter");
    }), g = f.lhs, h = t.sortBy(f.rhs, function(A) {
      return -A.i;
    }), v = [], y = 0, m = 0, b = 0;
    g.sort(l(!!c)), b = u(v, h, b), t.forEach(g, function(A) {
      b += A.vs.length, v.push(A.vs), y += A.barycenter * A.weight, m += A.weight, b = u(v, h, b);
    });
    var _ = { vs: t.flatten(v, !0) };
    return m && (_.barycenter = y / m, _.weight = m), _;
  }
  function u(s, c, f) {
    for (var g; c.length && (g = t.last(c)).i <= f; )
      c.pop(), s.push(g.vs), f++;
    return f;
  }
  function l(s) {
    return function(c, f) {
      return c.barycenter < f.barycenter ? -1 : c.barycenter > f.barycenter ? 1 : s ? f.i - c.i : c.i - f.i;
    };
  }
  return $p;
}
var Fp, _2;
function aO() {
  if (_2) return Fp;
  _2 = 1;
  var t = Fe(), r = tO(), i = nO(), u = rO();
  Fp = l;
  function l(f, g, h, v) {
    var y = f.children(g), m = f.node(g), b = m ? m.borderLeft : void 0, _ = m ? m.borderRight : void 0, A = {};
    b && (y = t.filter(y, function(N) {
      return N !== b && N !== _;
    }));
    var w = r(f, y);
    t.forEach(w, function(N) {
      if (f.children(N.v).length) {
        var R = l(f, N.v, h, v);
        A[N.v] = R, t.has(R, "barycenter") && c(N, R);
      }
    });
    var E = i(w, h);
    s(E, A);
    var M = u(E, v);
    if (b && (M.vs = t.flatten([b, M.vs, _], !0), f.predecessors(b).length)) {
      var S = f.node(f.predecessors(b)[0]), T = f.node(f.predecessors(_)[0]);
      t.has(M, "barycenter") || (M.barycenter = 0, M.weight = 0), M.barycenter = (M.barycenter * M.weight + S.order + T.order) / (M.weight + 2), M.weight += 2;
    }
    return M;
  }
  function s(f, g) {
    t.forEach(f, function(h) {
      h.vs = t.flatten(h.vs.map(function(v) {
        return g[v] ? g[v].vs : v;
      }), !0);
    });
  }
  function c(f, g) {
    t.isUndefined(f.barycenter) ? (f.barycenter = g.barycenter, f.weight = g.weight) : (f.barycenter = (f.barycenter * f.weight + g.barycenter * g.weight) / (f.weight + g.weight), f.weight += g.weight);
  }
  return Fp;
}
var Jp, x2;
function iO() {
  if (x2) return Jp;
  x2 = 1;
  var t = Fe(), r = gn().Graph;
  Jp = i;
  function i(l, s, c) {
    var f = u(l), g = new r({ compound: !0 }).setGraph({ root: f }).setDefaultNodeLabel(function(h) {
      return l.node(h);
    });
    return t.forEach(l.nodes(), function(h) {
      var v = l.node(h), y = l.parent(h);
      (v.rank === s || v.minRank <= s && s <= v.maxRank) && (g.setNode(h), g.setParent(h, y || f), t.forEach(l[c](h), function(m) {
        var b = m.v === h ? m.w : m.v, _ = g.edge(b, h), A = t.isUndefined(_) ? 0 : _.weight;
        g.setEdge(b, h, { weight: l.edge(m).weight + A });
      }), t.has(v, "minRank") && g.setNode(h, {
        borderLeft: v.borderLeft[s],
        borderRight: v.borderRight[s]
      }));
    }), g;
  }
  function u(l) {
    for (var s; l.hasNode(s = t.uniqueId("_root")); ) ;
    return s;
  }
  return Jp;
}
var Pp, S2;
function uO() {
  if (S2) return Pp;
  S2 = 1;
  var t = Fe();
  Pp = r;
  function r(i, u, l) {
    var s = {}, c;
    t.forEach(l, function(f) {
      for (var g = i.parent(f), h, v; g; ) {
        if (h = i.parent(g), h ? (v = s[h], s[h] = g) : (v = c, c = g), v && v !== g) {
          u.setEdge(v, g);
          return;
        }
        g = h;
      }
    });
  }
  return Pp;
}
var Wp, E2;
function lO() {
  if (E2) return Wp;
  E2 = 1;
  var t = Fe(), r = Wz(), i = eO(), u = aO(), l = iO(), s = uO(), c = gn().Graph, f = Ut();
  Wp = g;
  function g(m) {
    var b = f.maxRank(m), _ = h(m, t.range(1, b + 1), "inEdges"), A = h(m, t.range(b - 1, -1, -1), "outEdges"), w = r(m);
    y(m, w);
    for (var E = Number.POSITIVE_INFINITY, M, S = 0, T = 0; T < 4; ++S, ++T) {
      v(S % 2 ? _ : A, S % 4 >= 2), w = f.buildLayerMatrix(m);
      var N = i(m, w);
      N < E && (T = 0, M = t.cloneDeep(w), E = N);
    }
    y(m, M);
  }
  function h(m, b, _) {
    return t.map(b, function(A) {
      return l(m, A, _);
    });
  }
  function v(m, b) {
    var _ = new c();
    t.forEach(m, function(A) {
      var w = A.graph().root, E = u(A, w, _, b);
      t.forEach(E.vs, function(M, S) {
        A.node(M).order = S;
      }), s(A, _, E.vs);
    });
  }
  function y(m, b) {
    t.forEach(b, function(_) {
      t.forEach(_, function(A, w) {
        m.node(A).order = w;
      });
    });
  }
  return Wp;
}
var ey, w2;
function oO() {
  if (w2) return ey;
  w2 = 1;
  var t = Fe(), r = gn().Graph, i = Ut();
  ey = {
    positionX: _,
    findType1Conflicts: u,
    findType2Conflicts: l,
    addConflict: c,
    hasConflict: f,
    verticalAlignment: g,
    horizontalCompaction: h,
    alignCoordinates: m,
    findSmallestWidthAlignment: y,
    balance: b
  };
  function u(E, M) {
    var S = {};
    function T(N, R) {
      var q = 0, L = 0, j = N.length, Z = t.last(R);
      return t.forEach(R, function(Y, J) {
        var Q = s(E, Y), D = Q ? E.node(Q).order : j;
        (Q || Y === Z) && (t.forEach(R.slice(L, J + 1), function(X) {
          t.forEach(E.predecessors(X), function(z) {
            var B = E.node(z), $ = B.order;
            ($ < q || D < $) && !(B.dummy && E.node(X).dummy) && c(S, z, X);
          });
        }), L = J + 1, q = D);
      }), R;
    }
    return t.reduce(M, T), S;
  }
  function l(E, M) {
    var S = {};
    function T(R, q, L, j, Z) {
      var Y;
      t.forEach(t.range(q, L), function(J) {
        Y = R[J], E.node(Y).dummy && t.forEach(E.predecessors(Y), function(Q) {
          var D = E.node(Q);
          D.dummy && (D.order < j || D.order > Z) && c(S, Q, Y);
        });
      });
    }
    function N(R, q) {
      var L = -1, j, Z = 0;
      return t.forEach(q, function(Y, J) {
        if (E.node(Y).dummy === "border") {
          var Q = E.predecessors(Y);
          Q.length && (j = E.node(Q[0]).order, T(q, Z, J, L, j), Z = J, L = j);
        }
        T(q, Z, q.length, j, R.length);
      }), q;
    }
    return t.reduce(M, N), S;
  }
  function s(E, M) {
    if (E.node(M).dummy)
      return t.find(E.predecessors(M), function(S) {
        return E.node(S).dummy;
      });
  }
  function c(E, M, S) {
    if (M > S) {
      var T = M;
      M = S, S = T;
    }
    var N = E[M];
    N || (E[M] = N = {}), N[S] = !0;
  }
  function f(E, M, S) {
    if (M > S) {
      var T = M;
      M = S, S = T;
    }
    return t.has(E[M], S);
  }
  function g(E, M, S, T) {
    var N = {}, R = {}, q = {};
    return t.forEach(M, function(L) {
      t.forEach(L, function(j, Z) {
        N[j] = j, R[j] = j, q[j] = Z;
      });
    }), t.forEach(M, function(L) {
      var j = -1;
      t.forEach(L, function(Z) {
        var Y = T(Z);
        if (Y.length) {
          Y = t.sortBy(Y, function(z) {
            return q[z];
          });
          for (var J = (Y.length - 1) / 2, Q = Math.floor(J), D = Math.ceil(J); Q <= D; ++Q) {
            var X = Y[Q];
            R[Z] === Z && j < q[X] && !f(S, Z, X) && (R[X] = Z, R[Z] = N[Z] = N[X], j = q[X]);
          }
        }
      });
    }), { root: N, align: R };
  }
  function h(E, M, S, T, N) {
    var R = {}, q = v(E, M, S, N), L = N ? "borderLeft" : "borderRight";
    function j(J, Q) {
      for (var D = q.nodes(), X = D.pop(), z = {}; X; )
        z[X] ? J(X) : (z[X] = !0, D.push(X), D = D.concat(Q(X))), X = D.pop();
    }
    function Z(J) {
      R[J] = q.inEdges(J).reduce(function(Q, D) {
        return Math.max(Q, R[D.v] + q.edge(D));
      }, 0);
    }
    function Y(J) {
      var Q = q.outEdges(J).reduce(function(X, z) {
        return Math.min(X, R[z.w] - q.edge(z));
      }, Number.POSITIVE_INFINITY), D = E.node(J);
      Q !== Number.POSITIVE_INFINITY && D.borderType !== L && (R[J] = Math.max(R[J], Q));
    }
    return j(Z, q.predecessors.bind(q)), j(Y, q.successors.bind(q)), t.forEach(T, function(J) {
      R[J] = R[S[J]];
    }), R;
  }
  function v(E, M, S, T) {
    var N = new r(), R = E.graph(), q = A(R.nodesep, R.edgesep, T);
    return t.forEach(M, function(L) {
      var j;
      t.forEach(L, function(Z) {
        var Y = S[Z];
        if (N.setNode(Y), j) {
          var J = S[j], Q = N.edge(J, Y);
          N.setEdge(J, Y, Math.max(q(E, Z, j), Q || 0));
        }
        j = Z;
      });
    }), N;
  }
  function y(E, M) {
    return t.minBy(t.values(M), function(S) {
      var T = Number.NEGATIVE_INFINITY, N = Number.POSITIVE_INFINITY;
      return t.forIn(S, function(R, q) {
        var L = w(E, q) / 2;
        T = Math.max(R + L, T), N = Math.min(R - L, N);
      }), T - N;
    });
  }
  function m(E, M) {
    var S = t.values(M), T = t.min(S), N = t.max(S);
    t.forEach(["u", "d"], function(R) {
      t.forEach(["l", "r"], function(q) {
        var L = R + q, j = E[L], Z;
        if (j !== M) {
          var Y = t.values(j);
          Z = q === "l" ? T - t.min(Y) : N - t.max(Y), Z && (E[L] = t.mapValues(j, function(J) {
            return J + Z;
          }));
        }
      });
    });
  }
  function b(E, M) {
    return t.mapValues(E.ul, function(S, T) {
      if (M)
        return E[M.toLowerCase()][T];
      var N = t.sortBy(t.map(E, T));
      return (N[1] + N[2]) / 2;
    });
  }
  function _(E) {
    var M = i.buildLayerMatrix(E), S = t.merge(
      u(E, M),
      l(E, M)
    ), T = {}, N;
    t.forEach(["u", "d"], function(q) {
      N = q === "u" ? M : t.values(M).reverse(), t.forEach(["l", "r"], function(L) {
        L === "r" && (N = t.map(N, function(J) {
          return t.values(J).reverse();
        }));
        var j = (q === "u" ? E.predecessors : E.successors).bind(E), Z = g(E, N, S, j), Y = h(
          E,
          N,
          Z.root,
          Z.align,
          L === "r"
        );
        L === "r" && (Y = t.mapValues(Y, function(J) {
          return -J;
        })), T[q + L] = Y;
      });
    });
    var R = y(E, T);
    return m(T, R), b(T, E.graph().align);
  }
  function A(E, M, S) {
    return function(T, N, R) {
      var q = T.node(N), L = T.node(R), j = 0, Z;
      if (j += q.width / 2, t.has(q, "labelpos"))
        switch (q.labelpos.toLowerCase()) {
          case "l":
            Z = -q.width / 2;
            break;
          case "r":
            Z = q.width / 2;
            break;
        }
      if (Z && (j += S ? Z : -Z), Z = 0, j += (q.dummy ? M : E) / 2, j += (L.dummy ? M : E) / 2, j += L.width / 2, t.has(L, "labelpos"))
        switch (L.labelpos.toLowerCase()) {
          case "l":
            Z = L.width / 2;
            break;
          case "r":
            Z = -L.width / 2;
            break;
        }
      return Z && (j += S ? Z : -Z), Z = 0, j;
    };
  }
  function w(E, M) {
    return E.node(M).width;
  }
  return ey;
}
var ty, A2;
function sO() {
  if (A2) return ty;
  A2 = 1;
  var t = Fe(), r = Ut(), i = oO().positionX;
  ty = u;
  function u(s) {
    s = r.asNonCompoundGraph(s), l(s), t.forEach(i(s), function(c, f) {
      s.node(f).x = c;
    });
  }
  function l(s) {
    var c = r.buildLayerMatrix(s), f = s.graph().ranksep, g = 0;
    t.forEach(c, function(h) {
      var v = t.max(t.map(h, function(y) {
        return s.node(y).height;
      }));
      t.forEach(h, function(y) {
        s.node(y).y = g + v / 2;
      }), g += v + f;
    });
  }
  return ty;
}
var ny, T2;
function cO() {
  if (T2) return ny;
  T2 = 1;
  var t = Fe(), r = Iz(), i = Zz(), u = Qz(), l = Ut().normalizeRanks, s = $z(), c = Ut().removeEmptyRanks, f = Fz(), g = Jz(), h = Pz(), v = lO(), y = sO(), m = Ut(), b = gn().Graph;
  ny = _;
  function _(H, K) {
    var P = K && K.debugTiming ? m.time : m.notime;
    P("layout", function() {
      var ue = P("  buildLayoutGraph", function() {
        return j(H);
      });
      P("  runLayout", function() {
        A(ue, P);
      }), P("  updateInputGraph", function() {
        w(H, ue);
      });
    });
  }
  function A(H, K) {
    K("    makeSpaceForEdgeLabels", function() {
      Z(H);
    }), K("    removeSelfEdges", function() {
      U(H);
    }), K("    acyclic", function() {
      r.run(H);
    }), K("    nestingGraph.run", function() {
      f.run(H);
    }), K("    rank", function() {
      u(m.asNonCompoundGraph(H));
    }), K("    injectEdgeLabelProxies", function() {
      Y(H);
    }), K("    removeEmptyRanks", function() {
      c(H);
    }), K("    nestingGraph.cleanup", function() {
      f.cleanup(H);
    }), K("    normalizeRanks", function() {
      l(H);
    }), K("    assignRankMinMax", function() {
      J(H);
    }), K("    removeEdgeLabelProxies", function() {
      Q(H);
    }), K("    normalize.run", function() {
      i.run(H);
    }), K("    parentDummyChains", function() {
      s(H);
    }), K("    addBorderSegments", function() {
      g(H);
    }), K("    order", function() {
      v(H);
    }), K("    insertSelfEdges", function() {
      ae(H);
    }), K("    adjustCoordinateSystem", function() {
      h.adjust(H);
    }), K("    position", function() {
      y(H);
    }), K("    positionSelfEdges", function() {
      O(H);
    }), K("    removeBorderNodes", function() {
      $(H);
    }), K("    normalize.undo", function() {
      i.undo(H);
    }), K("    fixupEdgeLabelCoords", function() {
      z(H);
    }), K("    undoCoordinateSystem", function() {
      h.undo(H);
    }), K("    translateGraph", function() {
      D(H);
    }), K("    assignNodeIntersects", function() {
      X(H);
    }), K("    reversePoints", function() {
      B(H);
    }), K("    acyclic.undo", function() {
      r.undo(H);
    });
  }
  function w(H, K) {
    t.forEach(H.nodes(), function(P) {
      var ue = H.node(P), W = K.node(P);
      ue && (ue.x = W.x, ue.y = W.y, K.children(P).length && (ue.width = W.width, ue.height = W.height));
    }), t.forEach(H.edges(), function(P) {
      var ue = H.edge(P), W = K.edge(P);
      ue.points = W.points, t.has(W, "x") && (ue.x = W.x, ue.y = W.y);
    }), H.graph().width = K.graph().width, H.graph().height = K.graph().height;
  }
  var E = ["nodesep", "edgesep", "ranksep", "marginx", "marginy"], M = { ranksep: 50, edgesep: 20, nodesep: 50, rankdir: "tb" }, S = ["acyclicer", "ranker", "rankdir", "align"], T = ["width", "height"], N = { width: 0, height: 0 }, R = ["minlen", "weight", "width", "height", "labeloffset"], q = {
    minlen: 1,
    weight: 1,
    width: 0,
    height: 0,
    labeloffset: 10,
    labelpos: "r"
  }, L = ["labelpos"];
  function j(H) {
    var K = new b({ multigraph: !0, compound: !0 }), P = ie(H.graph());
    return K.setGraph(t.merge(
      {},
      M,
      G(P, E),
      t.pick(P, S)
    )), t.forEach(H.nodes(), function(ue) {
      var W = ie(H.node(ue));
      K.setNode(ue, t.defaults(G(W, T), N)), K.setParent(ue, H.parent(ue));
    }), t.forEach(H.edges(), function(ue) {
      var W = ie(H.edge(ue));
      K.setEdge(ue, t.merge(
        {},
        q,
        G(W, R),
        t.pick(W, L)
      ));
    }), K;
  }
  function Z(H) {
    var K = H.graph();
    K.ranksep /= 2, t.forEach(H.edges(), function(P) {
      var ue = H.edge(P);
      ue.minlen *= 2, ue.labelpos.toLowerCase() !== "c" && (K.rankdir === "TB" || K.rankdir === "BT" ? ue.width += ue.labeloffset : ue.height += ue.labeloffset);
    });
  }
  function Y(H) {
    t.forEach(H.edges(), function(K) {
      var P = H.edge(K);
      if (P.width && P.height) {
        var ue = H.node(K.v), W = H.node(K.w), se = { rank: (W.rank - ue.rank) / 2 + ue.rank, e: K };
        m.addDummyNode(H, "edge-proxy", se, "_ep");
      }
    });
  }
  function J(H) {
    var K = 0;
    t.forEach(H.nodes(), function(P) {
      var ue = H.node(P);
      ue.borderTop && (ue.minRank = H.node(ue.borderTop).rank, ue.maxRank = H.node(ue.borderBottom).rank, K = t.max(K, ue.maxRank));
    }), H.graph().maxRank = K;
  }
  function Q(H) {
    t.forEach(H.nodes(), function(K) {
      var P = H.node(K);
      P.dummy === "edge-proxy" && (H.edge(P.e).labelRank = P.rank, H.removeNode(K));
    });
  }
  function D(H) {
    var K = Number.POSITIVE_INFINITY, P = 0, ue = Number.POSITIVE_INFINITY, W = 0, se = H.graph(), de = se.marginx || 0, ve = se.marginy || 0;
    function ye(he) {
      var me = he.x, ge = he.y, Ae = he.width, xe = he.height;
      K = Math.min(K, me - Ae / 2), P = Math.max(P, me + Ae / 2), ue = Math.min(ue, ge - xe / 2), W = Math.max(W, ge + xe / 2);
    }
    t.forEach(H.nodes(), function(he) {
      ye(H.node(he));
    }), t.forEach(H.edges(), function(he) {
      var me = H.edge(he);
      t.has(me, "x") && ye(me);
    }), K -= de, ue -= ve, t.forEach(H.nodes(), function(he) {
      var me = H.node(he);
      me.x -= K, me.y -= ue;
    }), t.forEach(H.edges(), function(he) {
      var me = H.edge(he);
      t.forEach(me.points, function(ge) {
        ge.x -= K, ge.y -= ue;
      }), t.has(me, "x") && (me.x -= K), t.has(me, "y") && (me.y -= ue);
    }), se.width = P - K + de, se.height = W - ue + ve;
  }
  function X(H) {
    t.forEach(H.edges(), function(K) {
      var P = H.edge(K), ue = H.node(K.v), W = H.node(K.w), se, de;
      P.points ? (se = P.points[0], de = P.points[P.points.length - 1]) : (P.points = [], se = W, de = ue), P.points.unshift(m.intersectRect(ue, se)), P.points.push(m.intersectRect(W, de));
    });
  }
  function z(H) {
    t.forEach(H.edges(), function(K) {
      var P = H.edge(K);
      if (t.has(P, "x"))
        switch ((P.labelpos === "l" || P.labelpos === "r") && (P.width -= P.labeloffset), P.labelpos) {
          case "l":
            P.x -= P.width / 2 + P.labeloffset;
            break;
          case "r":
            P.x += P.width / 2 + P.labeloffset;
            break;
        }
    });
  }
  function B(H) {
    t.forEach(H.edges(), function(K) {
      var P = H.edge(K);
      P.reversed && P.points.reverse();
    });
  }
  function $(H) {
    t.forEach(H.nodes(), function(K) {
      if (H.children(K).length) {
        var P = H.node(K), ue = H.node(P.borderTop), W = H.node(P.borderBottom), se = H.node(t.last(P.borderLeft)), de = H.node(t.last(P.borderRight));
        P.width = Math.abs(de.x - se.x), P.height = Math.abs(W.y - ue.y), P.x = se.x + P.width / 2, P.y = ue.y + P.height / 2;
      }
    }), t.forEach(H.nodes(), function(K) {
      H.node(K).dummy === "border" && H.removeNode(K);
    });
  }
  function U(H) {
    t.forEach(H.edges(), function(K) {
      if (K.v === K.w) {
        var P = H.node(K.v);
        P.selfEdges || (P.selfEdges = []), P.selfEdges.push({ e: K, label: H.edge(K) }), H.removeEdge(K);
      }
    });
  }
  function ae(H) {
    var K = m.buildLayerMatrix(H);
    t.forEach(K, function(P) {
      var ue = 0;
      t.forEach(P, function(W, se) {
        var de = H.node(W);
        de.order = se + ue, t.forEach(de.selfEdges, function(ve) {
          m.addDummyNode(H, "selfedge", {
            width: ve.label.width,
            height: ve.label.height,
            rank: de.rank,
            order: se + ++ue,
            e: ve.e,
            label: ve.label
          }, "_se");
        }), delete de.selfEdges;
      });
    });
  }
  function O(H) {
    t.forEach(H.nodes(), function(K) {
      var P = H.node(K);
      if (P.dummy === "selfedge") {
        var ue = H.node(P.e.v), W = ue.x + ue.width / 2, se = ue.y, de = P.x - W, ve = ue.height / 2;
        H.setEdge(P.e, P.label), H.removeNode(K), P.label.points = [
          { x: W + 2 * de / 3, y: se - ve },
          { x: W + 5 * de / 6, y: se - ve },
          { x: W + de, y: se },
          { x: W + 5 * de / 6, y: se + ve },
          { x: W + 2 * de / 3, y: se + ve }
        ], P.label.x = P.x, P.label.y = P.y;
      }
    });
  }
  function G(H, K) {
    return t.mapValues(t.pick(H, K), Number);
  }
  function ie(H) {
    var K = {};
    return t.forEach(H, function(P, ue) {
      K[ue.toLowerCase()] = P;
    }), K;
  }
  return ny;
}
var ry, M2;
function fO() {
  if (M2) return ry;
  M2 = 1;
  var t = Fe(), r = Ut(), i = gn().Graph;
  ry = {
    debugOrdering: u
  };
  function u(l) {
    var s = r.buildLayerMatrix(l), c = new i({ compound: !0, multigraph: !0 }).setGraph({});
    return t.forEach(l.nodes(), function(f) {
      c.setNode(f, { label: f }), c.setParent(f, "layer" + l.node(f).rank);
    }), t.forEach(l.edges(), function(f) {
      c.setEdge(f.v, f.w, {}, f.name);
    }), t.forEach(s, function(f, g) {
      var h = "layer" + g;
      c.setNode(h, { rank: "same" }), t.reduce(f, function(v, y) {
        return c.setEdge(v, y, { style: "invis" }), y;
      });
    }), c;
  }
  return ry;
}
var ay, C2;
function dO() {
  return C2 || (C2 = 1, ay = "0.8.5"), ay;
}
var iy, q2;
function hO() {
  return q2 || (q2 = 1, iy = {
    graphlib: gn(),
    layout: cO(),
    debug: fO(),
    util: {
      time: Ut().time,
      notime: Ut().notime
    },
    version: dO()
  }), iy;
}
var gO = hO();
const R2 = /* @__PURE__ */ Ly(gO);
function ct(t) {
  if (typeof t == "string" || typeof t == "number") return "" + t;
  let r = "";
  if (Array.isArray(t))
    for (let i = 0, u; i < t.length; i++)
      (u = ct(t[i])) !== "" && (r += (r && " ") + u);
  else
    for (let i in t)
      t[i] && (r += (r && " ") + i);
  return r;
}
var vO = { value: () => {
} };
function cs() {
  for (var t = 0, r = arguments.length, i = {}, u; t < r; ++t) {
    if (!(u = arguments[t] + "") || u in i || /[\s.]/.test(u)) throw new Error("illegal type: " + u);
    i[u] = [];
  }
  return new Ro(i);
}
function Ro(t) {
  this._ = t;
}
function pO(t, r) {
  return t.trim().split(/^|\s+/).map(function(i) {
    var u = "", l = i.indexOf(".");
    if (l >= 0 && (u = i.slice(l + 1), i = i.slice(0, l)), i && !r.hasOwnProperty(i)) throw new Error("unknown type: " + i);
    return { type: i, name: u };
  });
}
Ro.prototype = cs.prototype = {
  constructor: Ro,
  on: function(t, r) {
    var i = this._, u = pO(t + "", i), l, s = -1, c = u.length;
    if (arguments.length < 2) {
      for (; ++s < c; ) if ((l = (t = u[s]).type) && (l = yO(i[l], t.name))) return l;
      return;
    }
    if (r != null && typeof r != "function") throw new Error("invalid callback: " + r);
    for (; ++s < c; )
      if (l = (t = u[s]).type) i[l] = N2(i[l], t.name, r);
      else if (r == null) for (l in i) i[l] = N2(i[l], t.name, null);
    return this;
  },
  copy: function() {
    var t = {}, r = this._;
    for (var i in r) t[i] = r[i].slice();
    return new Ro(t);
  },
  call: function(t, r) {
    if ((l = arguments.length - 2) > 0) for (var i = new Array(l), u = 0, l, s; u < l; ++u) i[u] = arguments[u + 2];
    if (!this._.hasOwnProperty(t)) throw new Error("unknown type: " + t);
    for (s = this._[t], u = 0, l = s.length; u < l; ++u) s[u].value.apply(r, i);
  },
  apply: function(t, r, i) {
    if (!this._.hasOwnProperty(t)) throw new Error("unknown type: " + t);
    for (var u = this._[t], l = 0, s = u.length; l < s; ++l) u[l].value.apply(r, i);
  }
};
function yO(t, r) {
  for (var i = 0, u = t.length, l; i < u; ++i)
    if ((l = t[i]).name === r)
      return l.value;
}
function N2(t, r, i) {
  for (var u = 0, l = t.length; u < l; ++u)
    if (t[u].name === r) {
      t[u] = vO, t = t.slice(0, u).concat(t.slice(u + 1));
      break;
    }
  return i != null && t.push({ name: r, value: i }), t;
}
var by = "http://www.w3.org/1999/xhtml";
const z2 = {
  svg: "http://www.w3.org/2000/svg",
  xhtml: by,
  xlink: "http://www.w3.org/1999/xlink",
  xml: "http://www.w3.org/XML/1998/namespace",
  xmlns: "http://www.w3.org/2000/xmlns/"
};
function fs(t) {
  var r = t += "", i = r.indexOf(":");
  return i >= 0 && (r = t.slice(0, i)) !== "xmlns" && (t = t.slice(i + 1)), z2.hasOwnProperty(r) ? { space: z2[r], local: t } : t;
}
function mO(t) {
  return function() {
    var r = this.ownerDocument, i = this.namespaceURI;
    return i === by && r.documentElement.namespaceURI === by ? r.createElement(t) : r.createElementNS(i, t);
  };
}
function bO(t) {
  return function() {
    return this.ownerDocument.createElementNS(t.space, t.local);
  };
}
function kT(t) {
  var r = fs(t);
  return (r.local ? bO : mO)(r);
}
function _O() {
}
function em(t) {
  return t == null ? _O : function() {
    return this.querySelector(t);
  };
}
function xO(t) {
  typeof t != "function" && (t = em(t));
  for (var r = this._groups, i = r.length, u = new Array(i), l = 0; l < i; ++l)
    for (var s = r[l], c = s.length, f = u[l] = new Array(c), g, h, v = 0; v < c; ++v)
      (g = s[v]) && (h = t.call(g, g.__data__, v, s)) && ("__data__" in g && (h.__data__ = g.__data__), f[v] = h);
  return new Qt(u, this._parents);
}
function SO(t) {
  return t == null ? [] : Array.isArray(t) ? t : Array.from(t);
}
function EO() {
  return [];
}
function XT(t) {
  return t == null ? EO : function() {
    return this.querySelectorAll(t);
  };
}
function wO(t) {
  return function() {
    return SO(t.apply(this, arguments));
  };
}
function AO(t) {
  typeof t == "function" ? t = wO(t) : t = XT(t);
  for (var r = this._groups, i = r.length, u = [], l = [], s = 0; s < i; ++s)
    for (var c = r[s], f = c.length, g, h = 0; h < f; ++h)
      (g = c[h]) && (u.push(t.call(g, g.__data__, h, c)), l.push(g));
  return new Qt(u, l);
}
function IT(t) {
  return function() {
    return this.matches(t);
  };
}
function ZT(t) {
  return function(r) {
    return r.matches(t);
  };
}
var TO = Array.prototype.find;
function MO(t) {
  return function() {
    return TO.call(this.children, t);
  };
}
function CO() {
  return this.firstElementChild;
}
function qO(t) {
  return this.select(t == null ? CO : MO(typeof t == "function" ? t : ZT(t)));
}
var RO = Array.prototype.filter;
function NO() {
  return Array.from(this.children);
}
function zO(t) {
  return function() {
    return RO.call(this.children, t);
  };
}
function OO(t) {
  return this.selectAll(t == null ? NO : zO(typeof t == "function" ? t : ZT(t)));
}
function DO(t) {
  typeof t != "function" && (t = IT(t));
  for (var r = this._groups, i = r.length, u = new Array(i), l = 0; l < i; ++l)
    for (var s = r[l], c = s.length, f = u[l] = [], g, h = 0; h < c; ++h)
      (g = s[h]) && t.call(g, g.__data__, h, s) && f.push(g);
  return new Qt(u, this._parents);
}
function KT(t) {
  return new Array(t.length);
}
function LO() {
  return new Qt(this._enter || this._groups.map(KT), this._parents);
}
function Ho(t, r) {
  this.ownerDocument = t.ownerDocument, this.namespaceURI = t.namespaceURI, this._next = null, this._parent = t, this.__data__ = r;
}
Ho.prototype = {
  constructor: Ho,
  appendChild: function(t) {
    return this._parent.insertBefore(t, this._next);
  },
  insertBefore: function(t, r) {
    return this._parent.insertBefore(t, r);
  },
  querySelector: function(t) {
    return this._parent.querySelector(t);
  },
  querySelectorAll: function(t) {
    return this._parent.querySelectorAll(t);
  }
};
function HO(t) {
  return function() {
    return t;
  };
}
function jO(t, r, i, u, l, s) {
  for (var c = 0, f, g = r.length, h = s.length; c < h; ++c)
    (f = r[c]) ? (f.__data__ = s[c], u[c] = f) : i[c] = new Ho(t, s[c]);
  for (; c < g; ++c)
    (f = r[c]) && (l[c] = f);
}
function BO(t, r, i, u, l, s, c) {
  var f, g, h = /* @__PURE__ */ new Map(), v = r.length, y = s.length, m = new Array(v), b;
  for (f = 0; f < v; ++f)
    (g = r[f]) && (m[f] = b = c.call(g, g.__data__, f, r) + "", h.has(b) ? l[f] = g : h.set(b, g));
  for (f = 0; f < y; ++f)
    b = c.call(t, s[f], f, s) + "", (g = h.get(b)) ? (u[f] = g, g.__data__ = s[f], h.delete(b)) : i[f] = new Ho(t, s[f]);
  for (f = 0; f < v; ++f)
    (g = r[f]) && h.get(m[f]) === g && (l[f] = g);
}
function UO(t) {
  return t.__data__;
}
function GO(t, r) {
  if (!arguments.length) return Array.from(this, UO);
  var i = r ? BO : jO, u = this._parents, l = this._groups;
  typeof t != "function" && (t = HO(t));
  for (var s = l.length, c = new Array(s), f = new Array(s), g = new Array(s), h = 0; h < s; ++h) {
    var v = u[h], y = l[h], m = y.length, b = VO(t.call(v, v && v.__data__, h, u)), _ = b.length, A = f[h] = new Array(_), w = c[h] = new Array(_), E = g[h] = new Array(m);
    i(v, y, A, w, E, b, r);
    for (var M = 0, S = 0, T, N; M < _; ++M)
      if (T = A[M]) {
        for (M >= S && (S = M + 1); !(N = w[S]) && ++S < _; ) ;
        T._next = N || null;
      }
  }
  return c = new Qt(c, u), c._enter = f, c._exit = g, c;
}
function VO(t) {
  return typeof t == "object" && "length" in t ? t : Array.from(t);
}
function YO() {
  return new Qt(this._exit || this._groups.map(KT), this._parents);
}
function kO(t, r, i) {
  var u = this.enter(), l = this, s = this.exit();
  return typeof t == "function" ? (u = t(u), u && (u = u.selection())) : u = u.append(t + ""), r != null && (l = r(l), l && (l = l.selection())), i == null ? s.remove() : i(s), u && l ? u.merge(l).order() : l;
}
function XO(t) {
  for (var r = t.selection ? t.selection() : t, i = this._groups, u = r._groups, l = i.length, s = u.length, c = Math.min(l, s), f = new Array(l), g = 0; g < c; ++g)
    for (var h = i[g], v = u[g], y = h.length, m = f[g] = new Array(y), b, _ = 0; _ < y; ++_)
      (b = h[_] || v[_]) && (m[_] = b);
  for (; g < l; ++g)
    f[g] = i[g];
  return new Qt(f, this._parents);
}
function IO() {
  for (var t = this._groups, r = -1, i = t.length; ++r < i; )
    for (var u = t[r], l = u.length - 1, s = u[l], c; --l >= 0; )
      (c = u[l]) && (s && c.compareDocumentPosition(s) ^ 4 && s.parentNode.insertBefore(c, s), s = c);
  return this;
}
function ZO(t) {
  t || (t = KO);
  function r(y, m) {
    return y && m ? t(y.__data__, m.__data__) : !y - !m;
  }
  for (var i = this._groups, u = i.length, l = new Array(u), s = 0; s < u; ++s) {
    for (var c = i[s], f = c.length, g = l[s] = new Array(f), h, v = 0; v < f; ++v)
      (h = c[v]) && (g[v] = h);
    g.sort(r);
  }
  return new Qt(l, this._parents).order();
}
function KO(t, r) {
  return t < r ? -1 : t > r ? 1 : t >= r ? 0 : NaN;
}
function QO() {
  var t = arguments[0];
  return arguments[0] = this, t.apply(null, arguments), this;
}
function $O() {
  return Array.from(this);
}
function FO() {
  for (var t = this._groups, r = 0, i = t.length; r < i; ++r)
    for (var u = t[r], l = 0, s = u.length; l < s; ++l) {
      var c = u[l];
      if (c) return c;
    }
  return null;
}
function JO() {
  let t = 0;
  for (const r of this) ++t;
  return t;
}
function PO() {
  return !this.node();
}
function WO(t) {
  for (var r = this._groups, i = 0, u = r.length; i < u; ++i)
    for (var l = r[i], s = 0, c = l.length, f; s < c; ++s)
      (f = l[s]) && t.call(f, f.__data__, s, l);
  return this;
}
function eD(t) {
  return function() {
    this.removeAttribute(t);
  };
}
function tD(t) {
  return function() {
    this.removeAttributeNS(t.space, t.local);
  };
}
function nD(t, r) {
  return function() {
    this.setAttribute(t, r);
  };
}
function rD(t, r) {
  return function() {
    this.setAttributeNS(t.space, t.local, r);
  };
}
function aD(t, r) {
  return function() {
    var i = r.apply(this, arguments);
    i == null ? this.removeAttribute(t) : this.setAttribute(t, i);
  };
}
function iD(t, r) {
  return function() {
    var i = r.apply(this, arguments);
    i == null ? this.removeAttributeNS(t.space, t.local) : this.setAttributeNS(t.space, t.local, i);
  };
}
function uD(t, r) {
  var i = fs(t);
  if (arguments.length < 2) {
    var u = this.node();
    return i.local ? u.getAttributeNS(i.space, i.local) : u.getAttribute(i);
  }
  return this.each((r == null ? i.local ? tD : eD : typeof r == "function" ? i.local ? iD : aD : i.local ? rD : nD)(i, r));
}
function QT(t) {
  return t.ownerDocument && t.ownerDocument.defaultView || t.document && t || t.defaultView;
}
function lD(t) {
  return function() {
    this.style.removeProperty(t);
  };
}
function oD(t, r, i) {
  return function() {
    this.style.setProperty(t, r, i);
  };
}
function sD(t, r, i) {
  return function() {
    var u = r.apply(this, arguments);
    u == null ? this.style.removeProperty(t) : this.style.setProperty(t, u, i);
  };
}
function cD(t, r, i) {
  return arguments.length > 1 ? this.each((r == null ? lD : typeof r == "function" ? sD : oD)(t, r, i ?? "")) : oi(this.node(), t);
}
function oi(t, r) {
  return t.style.getPropertyValue(r) || QT(t).getComputedStyle(t, null).getPropertyValue(r);
}
function fD(t) {
  return function() {
    delete this[t];
  };
}
function dD(t, r) {
  return function() {
    this[t] = r;
  };
}
function hD(t, r) {
  return function() {
    var i = r.apply(this, arguments);
    i == null ? delete this[t] : this[t] = i;
  };
}
function gD(t, r) {
  return arguments.length > 1 ? this.each((r == null ? fD : typeof r == "function" ? hD : dD)(t, r)) : this.node()[t];
}
function $T(t) {
  return t.trim().split(/^|\s+/);
}
function tm(t) {
  return t.classList || new FT(t);
}
function FT(t) {
  this._node = t, this._names = $T(t.getAttribute("class") || "");
}
FT.prototype = {
  add: function(t) {
    var r = this._names.indexOf(t);
    r < 0 && (this._names.push(t), this._node.setAttribute("class", this._names.join(" ")));
  },
  remove: function(t) {
    var r = this._names.indexOf(t);
    r >= 0 && (this._names.splice(r, 1), this._node.setAttribute("class", this._names.join(" ")));
  },
  contains: function(t) {
    return this._names.indexOf(t) >= 0;
  }
};
function JT(t, r) {
  for (var i = tm(t), u = -1, l = r.length; ++u < l; ) i.add(r[u]);
}
function PT(t, r) {
  for (var i = tm(t), u = -1, l = r.length; ++u < l; ) i.remove(r[u]);
}
function vD(t) {
  return function() {
    JT(this, t);
  };
}
function pD(t) {
  return function() {
    PT(this, t);
  };
}
function yD(t, r) {
  return function() {
    (r.apply(this, arguments) ? JT : PT)(this, t);
  };
}
function mD(t, r) {
  var i = $T(t + "");
  if (arguments.length < 2) {
    for (var u = tm(this.node()), l = -1, s = i.length; ++l < s; ) if (!u.contains(i[l])) return !1;
    return !0;
  }
  return this.each((typeof r == "function" ? yD : r ? vD : pD)(i, r));
}
function bD() {
  this.textContent = "";
}
function _D(t) {
  return function() {
    this.textContent = t;
  };
}
function xD(t) {
  return function() {
    var r = t.apply(this, arguments);
    this.textContent = r ?? "";
  };
}
function SD(t) {
  return arguments.length ? this.each(t == null ? bD : (typeof t == "function" ? xD : _D)(t)) : this.node().textContent;
}
function ED() {
  this.innerHTML = "";
}
function wD(t) {
  return function() {
    this.innerHTML = t;
  };
}
function AD(t) {
  return function() {
    var r = t.apply(this, arguments);
    this.innerHTML = r ?? "";
  };
}
function TD(t) {
  return arguments.length ? this.each(t == null ? ED : (typeof t == "function" ? AD : wD)(t)) : this.node().innerHTML;
}
function MD() {
  this.nextSibling && this.parentNode.appendChild(this);
}
function CD() {
  return this.each(MD);
}
function qD() {
  this.previousSibling && this.parentNode.insertBefore(this, this.parentNode.firstChild);
}
function RD() {
  return this.each(qD);
}
function ND(t) {
  var r = typeof t == "function" ? t : kT(t);
  return this.select(function() {
    return this.appendChild(r.apply(this, arguments));
  });
}
function zD() {
  return null;
}
function OD(t, r) {
  var i = typeof t == "function" ? t : kT(t), u = r == null ? zD : typeof r == "function" ? r : em(r);
  return this.select(function() {
    return this.insertBefore(i.apply(this, arguments), u.apply(this, arguments) || null);
  });
}
function DD() {
  var t = this.parentNode;
  t && t.removeChild(this);
}
function LD() {
  return this.each(DD);
}
function HD() {
  var t = this.cloneNode(!1), r = this.parentNode;
  return r ? r.insertBefore(t, this.nextSibling) : t;
}
function jD() {
  var t = this.cloneNode(!0), r = this.parentNode;
  return r ? r.insertBefore(t, this.nextSibling) : t;
}
function BD(t) {
  return this.select(t ? jD : HD);
}
function UD(t) {
  return arguments.length ? this.property("__data__", t) : this.node().__data__;
}
function GD(t) {
  return function(r) {
    t.call(this, r, this.__data__);
  };
}
function VD(t) {
  return t.trim().split(/^|\s+/).map(function(r) {
    var i = "", u = r.indexOf(".");
    return u >= 0 && (i = r.slice(u + 1), r = r.slice(0, u)), { type: r, name: i };
  });
}
function YD(t) {
  return function() {
    var r = this.__on;
    if (r) {
      for (var i = 0, u = -1, l = r.length, s; i < l; ++i)
        s = r[i], (!t.type || s.type === t.type) && s.name === t.name ? this.removeEventListener(s.type, s.listener, s.options) : r[++u] = s;
      ++u ? r.length = u : delete this.__on;
    }
  };
}
function kD(t, r, i) {
  return function() {
    var u = this.__on, l, s = GD(r);
    if (u) {
      for (var c = 0, f = u.length; c < f; ++c)
        if ((l = u[c]).type === t.type && l.name === t.name) {
          this.removeEventListener(l.type, l.listener, l.options), this.addEventListener(l.type, l.listener = s, l.options = i), l.value = r;
          return;
        }
    }
    this.addEventListener(t.type, s, i), l = { type: t.type, name: t.name, value: r, listener: s, options: i }, u ? u.push(l) : this.__on = [l];
  };
}
function XD(t, r, i) {
  var u = VD(t + ""), l, s = u.length, c;
  if (arguments.length < 2) {
    var f = this.node().__on;
    if (f) {
      for (var g = 0, h = f.length, v; g < h; ++g)
        for (l = 0, v = f[g]; l < s; ++l)
          if ((c = u[l]).type === v.type && c.name === v.name)
            return v.value;
    }
    return;
  }
  for (f = r ? kD : YD, l = 0; l < s; ++l) this.each(f(u[l], r, i));
  return this;
}
function WT(t, r, i) {
  var u = QT(t), l = u.CustomEvent;
  typeof l == "function" ? l = new l(r, i) : (l = u.document.createEvent("Event"), i ? (l.initEvent(r, i.bubbles, i.cancelable), l.detail = i.detail) : l.initEvent(r, !1, !1)), t.dispatchEvent(l);
}
function ID(t, r) {
  return function() {
    return WT(this, t, r);
  };
}
function ZD(t, r) {
  return function() {
    return WT(this, t, r.apply(this, arguments));
  };
}
function KD(t, r) {
  return this.each((typeof r == "function" ? ZD : ID)(t, r));
}
function* QD() {
  for (var t = this._groups, r = 0, i = t.length; r < i; ++r)
    for (var u = t[r], l = 0, s = u.length, c; l < s; ++l)
      (c = u[l]) && (yield c);
}
var eM = [null];
function Qt(t, r) {
  this._groups = t, this._parents = r;
}
function ku() {
  return new Qt([[document.documentElement]], eM);
}
function $D() {
  return this;
}
Qt.prototype = ku.prototype = {
  constructor: Qt,
  select: xO,
  selectAll: AO,
  selectChild: qO,
  selectChildren: OO,
  filter: DO,
  data: GO,
  enter: LO,
  exit: YO,
  join: kO,
  merge: XO,
  selection: $D,
  order: IO,
  sort: ZO,
  call: QO,
  nodes: $O,
  node: FO,
  size: JO,
  empty: PO,
  each: WO,
  attr: uD,
  style: cD,
  property: gD,
  classed: mD,
  text: SD,
  html: TD,
  raise: CD,
  lower: RD,
  append: ND,
  insert: OD,
  remove: LD,
  clone: BD,
  datum: UD,
  on: XD,
  dispatch: KD,
  [Symbol.iterator]: QD
};
function Kt(t) {
  return typeof t == "string" ? new Qt([[document.querySelector(t)]], [document.documentElement]) : new Qt([[t]], eM);
}
function FD(t) {
  let r;
  for (; r = t.sourceEvent; ) t = r;
  return t;
}
function cn(t, r) {
  if (t = FD(t), r === void 0 && (r = t.currentTarget), r) {
    var i = r.ownerSVGElement || r;
    if (i.createSVGPoint) {
      var u = i.createSVGPoint();
      return u.x = t.clientX, u.y = t.clientY, u = u.matrixTransform(r.getScreenCTM().inverse()), [u.x, u.y];
    }
    if (r.getBoundingClientRect) {
      var l = r.getBoundingClientRect();
      return [t.clientX - l.left - r.clientLeft, t.clientY - l.top - r.clientTop];
    }
  }
  return [t.pageX, t.pageY];
}
const JD = { passive: !1 }, Cu = { capture: !0, passive: !1 };
function uy(t) {
  t.stopImmediatePropagation();
}
function ui(t) {
  t.preventDefault(), t.stopImmediatePropagation();
}
function tM(t) {
  var r = t.document.documentElement, i = Kt(t).on("dragstart.drag", ui, Cu);
  "onselectstart" in r ? i.on("selectstart.drag", ui, Cu) : (r.__noselect = r.style.MozUserSelect, r.style.MozUserSelect = "none");
}
function nM(t, r) {
  var i = t.document.documentElement, u = Kt(t).on("dragstart.drag", null);
  r && (u.on("click.drag", ui, Cu), setTimeout(function() {
    u.on("click.drag", null);
  }, 0)), "onselectstart" in i ? u.on("selectstart.drag", null) : (i.style.MozUserSelect = i.__noselect, delete i.__noselect);
}
const _o = (t) => () => t;
function _y(t, {
  sourceEvent: r,
  subject: i,
  target: u,
  identifier: l,
  active: s,
  x: c,
  y: f,
  dx: g,
  dy: h,
  dispatch: v
}) {
  Object.defineProperties(this, {
    type: { value: t, enumerable: !0, configurable: !0 },
    sourceEvent: { value: r, enumerable: !0, configurable: !0 },
    subject: { value: i, enumerable: !0, configurable: !0 },
    target: { value: u, enumerable: !0, configurable: !0 },
    identifier: { value: l, enumerable: !0, configurable: !0 },
    active: { value: s, enumerable: !0, configurable: !0 },
    x: { value: c, enumerable: !0, configurable: !0 },
    y: { value: f, enumerable: !0, configurable: !0 },
    dx: { value: g, enumerable: !0, configurable: !0 },
    dy: { value: h, enumerable: !0, configurable: !0 },
    _: { value: v }
  });
}
_y.prototype.on = function() {
  var t = this._.on.apply(this._, arguments);
  return t === this._ ? this : t;
};
function PD(t) {
  return !t.ctrlKey && !t.button;
}
function WD() {
  return this.parentNode;
}
function e4(t, r) {
  return r ?? { x: t.x, y: t.y };
}
function t4() {
  return navigator.maxTouchPoints || "ontouchstart" in this;
}
function rM() {
  var t = PD, r = WD, i = e4, u = t4, l = {}, s = cs("start", "drag", "end"), c = 0, f, g, h, v, y = 0;
  function m(T) {
    T.on("mousedown.drag", b).filter(u).on("touchstart.drag", w).on("touchmove.drag", E, JD).on("touchend.drag touchcancel.drag", M).style("touch-action", "none").style("-webkit-tap-highlight-color", "rgba(0,0,0,0)");
  }
  function b(T, N) {
    if (!(v || !t.call(this, T, N))) {
      var R = S(this, r.call(this, T, N), T, N, "mouse");
      R && (Kt(T.view).on("mousemove.drag", _, Cu).on("mouseup.drag", A, Cu), tM(T.view), uy(T), h = !1, f = T.clientX, g = T.clientY, R("start", T));
    }
  }
  function _(T) {
    if (ui(T), !h) {
      var N = T.clientX - f, R = T.clientY - g;
      h = N * N + R * R > y;
    }
    l.mouse("drag", T);
  }
  function A(T) {
    Kt(T.view).on("mousemove.drag mouseup.drag", null), nM(T.view, h), ui(T), l.mouse("end", T);
  }
  function w(T, N) {
    if (t.call(this, T, N)) {
      var R = T.changedTouches, q = r.call(this, T, N), L = R.length, j, Z;
      for (j = 0; j < L; ++j)
        (Z = S(this, q, T, N, R[j].identifier, R[j])) && (uy(T), Z("start", T, R[j]));
    }
  }
  function E(T) {
    var N = T.changedTouches, R = N.length, q, L;
    for (q = 0; q < R; ++q)
      (L = l[N[q].identifier]) && (ui(T), L("drag", T, N[q]));
  }
  function M(T) {
    var N = T.changedTouches, R = N.length, q, L;
    for (v && clearTimeout(v), v = setTimeout(function() {
      v = null;
    }, 500), q = 0; q < R; ++q)
      (L = l[N[q].identifier]) && (uy(T), L("end", T, N[q]));
  }
  function S(T, N, R, q, L, j) {
    var Z = s.copy(), Y = cn(j || R, N), J, Q, D;
    if ((D = i.call(T, new _y("beforestart", {
      sourceEvent: R,
      target: m,
      identifier: L,
      active: c,
      x: Y[0],
      y: Y[1],
      dx: 0,
      dy: 0,
      dispatch: Z
    }), q)) != null)
      return J = D.x - Y[0] || 0, Q = D.y - Y[1] || 0, function X(z, B, $) {
        var U = Y, ae;
        switch (z) {
          case "start":
            l[L] = X, ae = c++;
            break;
          case "end":
            delete l[L], --c;
          // falls through
          case "drag":
            Y = cn($ || B, N), ae = c;
            break;
        }
        Z.call(
          z,
          T,
          new _y(z, {
            sourceEvent: B,
            subject: D,
            target: m,
            identifier: L,
            active: ae,
            x: Y[0] + J,
            y: Y[1] + Q,
            dx: Y[0] - U[0],
            dy: Y[1] - U[1],
            dispatch: Z
          }),
          q
        );
      };
  }
  return m.filter = function(T) {
    return arguments.length ? (t = typeof T == "function" ? T : _o(!!T), m) : t;
  }, m.container = function(T) {
    return arguments.length ? (r = typeof T == "function" ? T : _o(T), m) : r;
  }, m.subject = function(T) {
    return arguments.length ? (i = typeof T == "function" ? T : _o(T), m) : i;
  }, m.touchable = function(T) {
    return arguments.length ? (u = typeof T == "function" ? T : _o(!!T), m) : u;
  }, m.on = function() {
    var T = s.on.apply(s, arguments);
    return T === s ? m : T;
  }, m.clickDistance = function(T) {
    return arguments.length ? (y = (T = +T) * T, m) : Math.sqrt(y);
  }, m;
}
function nm(t, r, i) {
  t.prototype = r.prototype = i, i.constructor = t;
}
function aM(t, r) {
  var i = Object.create(t.prototype);
  for (var u in r) i[u] = r[u];
  return i;
}
function Xu() {
}
var qu = 0.7, jo = 1 / qu, li = "\\s*([+-]?\\d+)\\s*", Ru = "\\s*([+-]?(?:\\d*\\.)?\\d+(?:[eE][+-]?\\d+)?)\\s*", An = "\\s*([+-]?(?:\\d*\\.)?\\d+(?:[eE][+-]?\\d+)?)%\\s*", n4 = /^#([0-9a-f]{3,8})$/, r4 = new RegExp(`^rgb\\(${li},${li},${li}\\)$`), a4 = new RegExp(`^rgb\\(${An},${An},${An}\\)$`), i4 = new RegExp(`^rgba\\(${li},${li},${li},${Ru}\\)$`), u4 = new RegExp(`^rgba\\(${An},${An},${An},${Ru}\\)$`), l4 = new RegExp(`^hsl\\(${Ru},${An},${An}\\)$`), o4 = new RegExp(`^hsla\\(${Ru},${An},${An},${Ru}\\)$`), O2 = {
  aliceblue: 15792383,
  antiquewhite: 16444375,
  aqua: 65535,
  aquamarine: 8388564,
  azure: 15794175,
  beige: 16119260,
  bisque: 16770244,
  black: 0,
  blanchedalmond: 16772045,
  blue: 255,
  blueviolet: 9055202,
  brown: 10824234,
  burlywood: 14596231,
  cadetblue: 6266528,
  chartreuse: 8388352,
  chocolate: 13789470,
  coral: 16744272,
  cornflowerblue: 6591981,
  cornsilk: 16775388,
  crimson: 14423100,
  cyan: 65535,
  darkblue: 139,
  darkcyan: 35723,
  darkgoldenrod: 12092939,
  darkgray: 11119017,
  darkgreen: 25600,
  darkgrey: 11119017,
  darkkhaki: 12433259,
  darkmagenta: 9109643,
  darkolivegreen: 5597999,
  darkorange: 16747520,
  darkorchid: 10040012,
  darkred: 9109504,
  darksalmon: 15308410,
  darkseagreen: 9419919,
  darkslateblue: 4734347,
  darkslategray: 3100495,
  darkslategrey: 3100495,
  darkturquoise: 52945,
  darkviolet: 9699539,
  deeppink: 16716947,
  deepskyblue: 49151,
  dimgray: 6908265,
  dimgrey: 6908265,
  dodgerblue: 2003199,
  firebrick: 11674146,
  floralwhite: 16775920,
  forestgreen: 2263842,
  fuchsia: 16711935,
  gainsboro: 14474460,
  ghostwhite: 16316671,
  gold: 16766720,
  goldenrod: 14329120,
  gray: 8421504,
  green: 32768,
  greenyellow: 11403055,
  grey: 8421504,
  honeydew: 15794160,
  hotpink: 16738740,
  indianred: 13458524,
  indigo: 4915330,
  ivory: 16777200,
  khaki: 15787660,
  lavender: 15132410,
  lavenderblush: 16773365,
  lawngreen: 8190976,
  lemonchiffon: 16775885,
  lightblue: 11393254,
  lightcoral: 15761536,
  lightcyan: 14745599,
  lightgoldenrodyellow: 16448210,
  lightgray: 13882323,
  lightgreen: 9498256,
  lightgrey: 13882323,
  lightpink: 16758465,
  lightsalmon: 16752762,
  lightseagreen: 2142890,
  lightskyblue: 8900346,
  lightslategray: 7833753,
  lightslategrey: 7833753,
  lightsteelblue: 11584734,
  lightyellow: 16777184,
  lime: 65280,
  limegreen: 3329330,
  linen: 16445670,
  magenta: 16711935,
  maroon: 8388608,
  mediumaquamarine: 6737322,
  mediumblue: 205,
  mediumorchid: 12211667,
  mediumpurple: 9662683,
  mediumseagreen: 3978097,
  mediumslateblue: 8087790,
  mediumspringgreen: 64154,
  mediumturquoise: 4772300,
  mediumvioletred: 13047173,
  midnightblue: 1644912,
  mintcream: 16121850,
  mistyrose: 16770273,
  moccasin: 16770229,
  navajowhite: 16768685,
  navy: 128,
  oldlace: 16643558,
  olive: 8421376,
  olivedrab: 7048739,
  orange: 16753920,
  orangered: 16729344,
  orchid: 14315734,
  palegoldenrod: 15657130,
  palegreen: 10025880,
  paleturquoise: 11529966,
  palevioletred: 14381203,
  papayawhip: 16773077,
  peachpuff: 16767673,
  peru: 13468991,
  pink: 16761035,
  plum: 14524637,
  powderblue: 11591910,
  purple: 8388736,
  rebeccapurple: 6697881,
  red: 16711680,
  rosybrown: 12357519,
  royalblue: 4286945,
  saddlebrown: 9127187,
  salmon: 16416882,
  sandybrown: 16032864,
  seagreen: 3050327,
  seashell: 16774638,
  sienna: 10506797,
  silver: 12632256,
  skyblue: 8900331,
  slateblue: 6970061,
  slategray: 7372944,
  slategrey: 7372944,
  snow: 16775930,
  springgreen: 65407,
  steelblue: 4620980,
  tan: 13808780,
  teal: 32896,
  thistle: 14204888,
  tomato: 16737095,
  turquoise: 4251856,
  violet: 15631086,
  wheat: 16113331,
  white: 16777215,
  whitesmoke: 16119285,
  yellow: 16776960,
  yellowgreen: 10145074
};
nm(Xu, ua, {
  copy(t) {
    return Object.assign(new this.constructor(), this, t);
  },
  displayable() {
    return this.rgb().displayable();
  },
  hex: D2,
  // Deprecated! Use color.formatHex.
  formatHex: D2,
  formatHex8: s4,
  formatHsl: c4,
  formatRgb: L2,
  toString: L2
});
function D2() {
  return this.rgb().formatHex();
}
function s4() {
  return this.rgb().formatHex8();
}
function c4() {
  return iM(this).formatHsl();
}
function L2() {
  return this.rgb().formatRgb();
}
function ua(t) {
  var r, i;
  return t = (t + "").trim().toLowerCase(), (r = n4.exec(t)) ? (i = r[1].length, r = parseInt(r[1], 16), i === 6 ? H2(r) : i === 3 ? new Bt(r >> 8 & 15 | r >> 4 & 240, r >> 4 & 15 | r & 240, (r & 15) << 4 | r & 15, 1) : i === 8 ? xo(r >> 24 & 255, r >> 16 & 255, r >> 8 & 255, (r & 255) / 255) : i === 4 ? xo(r >> 12 & 15 | r >> 8 & 240, r >> 8 & 15 | r >> 4 & 240, r >> 4 & 15 | r & 240, ((r & 15) << 4 | r & 15) / 255) : null) : (r = r4.exec(t)) ? new Bt(r[1], r[2], r[3], 1) : (r = a4.exec(t)) ? new Bt(r[1] * 255 / 100, r[2] * 255 / 100, r[3] * 255 / 100, 1) : (r = i4.exec(t)) ? xo(r[1], r[2], r[3], r[4]) : (r = u4.exec(t)) ? xo(r[1] * 255 / 100, r[2] * 255 / 100, r[3] * 255 / 100, r[4]) : (r = l4.exec(t)) ? U2(r[1], r[2] / 100, r[3] / 100, 1) : (r = o4.exec(t)) ? U2(r[1], r[2] / 100, r[3] / 100, r[4]) : O2.hasOwnProperty(t) ? H2(O2[t]) : t === "transparent" ? new Bt(NaN, NaN, NaN, 0) : null;
}
function H2(t) {
  return new Bt(t >> 16 & 255, t >> 8 & 255, t & 255, 1);
}
function xo(t, r, i, u) {
  return u <= 0 && (t = r = i = NaN), new Bt(t, r, i, u);
}
function f4(t) {
  return t instanceof Xu || (t = ua(t)), t ? (t = t.rgb(), new Bt(t.r, t.g, t.b, t.opacity)) : new Bt();
}
function xy(t, r, i, u) {
  return arguments.length === 1 ? f4(t) : new Bt(t, r, i, u ?? 1);
}
function Bt(t, r, i, u) {
  this.r = +t, this.g = +r, this.b = +i, this.opacity = +u;
}
nm(Bt, xy, aM(Xu, {
  brighter(t) {
    return t = t == null ? jo : Math.pow(jo, t), new Bt(this.r * t, this.g * t, this.b * t, this.opacity);
  },
  darker(t) {
    return t = t == null ? qu : Math.pow(qu, t), new Bt(this.r * t, this.g * t, this.b * t, this.opacity);
  },
  rgb() {
    return this;
  },
  clamp() {
    return new Bt(aa(this.r), aa(this.g), aa(this.b), Bo(this.opacity));
  },
  displayable() {
    return -0.5 <= this.r && this.r < 255.5 && -0.5 <= this.g && this.g < 255.5 && -0.5 <= this.b && this.b < 255.5 && 0 <= this.opacity && this.opacity <= 1;
  },
  hex: j2,
  // Deprecated! Use color.formatHex.
  formatHex: j2,
  formatHex8: d4,
  formatRgb: B2,
  toString: B2
}));
function j2() {
  return `#${ra(this.r)}${ra(this.g)}${ra(this.b)}`;
}
function d4() {
  return `#${ra(this.r)}${ra(this.g)}${ra(this.b)}${ra((isNaN(this.opacity) ? 1 : this.opacity) * 255)}`;
}
function B2() {
  const t = Bo(this.opacity);
  return `${t === 1 ? "rgb(" : "rgba("}${aa(this.r)}, ${aa(this.g)}, ${aa(this.b)}${t === 1 ? ")" : `, ${t})`}`;
}
function Bo(t) {
  return isNaN(t) ? 1 : Math.max(0, Math.min(1, t));
}
function aa(t) {
  return Math.max(0, Math.min(255, Math.round(t) || 0));
}
function ra(t) {
  return t = aa(t), (t < 16 ? "0" : "") + t.toString(16);
}
function U2(t, r, i, u) {
  return u <= 0 ? t = r = i = NaN : i <= 0 || i >= 1 ? t = r = NaN : r <= 0 && (t = NaN), new fn(t, r, i, u);
}
function iM(t) {
  if (t instanceof fn) return new fn(t.h, t.s, t.l, t.opacity);
  if (t instanceof Xu || (t = ua(t)), !t) return new fn();
  if (t instanceof fn) return t;
  t = t.rgb();
  var r = t.r / 255, i = t.g / 255, u = t.b / 255, l = Math.min(r, i, u), s = Math.max(r, i, u), c = NaN, f = s - l, g = (s + l) / 2;
  return f ? (r === s ? c = (i - u) / f + (i < u) * 6 : i === s ? c = (u - r) / f + 2 : c = (r - i) / f + 4, f /= g < 0.5 ? s + l : 2 - s - l, c *= 60) : f = g > 0 && g < 1 ? 0 : c, new fn(c, f, g, t.opacity);
}
function h4(t, r, i, u) {
  return arguments.length === 1 ? iM(t) : new fn(t, r, i, u ?? 1);
}
function fn(t, r, i, u) {
  this.h = +t, this.s = +r, this.l = +i, this.opacity = +u;
}
nm(fn, h4, aM(Xu, {
  brighter(t) {
    return t = t == null ? jo : Math.pow(jo, t), new fn(this.h, this.s, this.l * t, this.opacity);
  },
  darker(t) {
    return t = t == null ? qu : Math.pow(qu, t), new fn(this.h, this.s, this.l * t, this.opacity);
  },
  rgb() {
    var t = this.h % 360 + (this.h < 0) * 360, r = isNaN(t) || isNaN(this.s) ? 0 : this.s, i = this.l, u = i + (i < 0.5 ? i : 1 - i) * r, l = 2 * i - u;
    return new Bt(
      ly(t >= 240 ? t - 240 : t + 120, l, u),
      ly(t, l, u),
      ly(t < 120 ? t + 240 : t - 120, l, u),
      this.opacity
    );
  },
  clamp() {
    return new fn(G2(this.h), So(this.s), So(this.l), Bo(this.opacity));
  },
  displayable() {
    return (0 <= this.s && this.s <= 1 || isNaN(this.s)) && 0 <= this.l && this.l <= 1 && 0 <= this.opacity && this.opacity <= 1;
  },
  formatHsl() {
    const t = Bo(this.opacity);
    return `${t === 1 ? "hsl(" : "hsla("}${G2(this.h)}, ${So(this.s) * 100}%, ${So(this.l) * 100}%${t === 1 ? ")" : `, ${t})`}`;
  }
}));
function G2(t) {
  return t = (t || 0) % 360, t < 0 ? t + 360 : t;
}
function So(t) {
  return Math.max(0, Math.min(1, t || 0));
}
function ly(t, r, i) {
  return (t < 60 ? r + (i - r) * t / 60 : t < 180 ? i : t < 240 ? r + (i - r) * (240 - t) / 60 : r) * 255;
}
const rm = (t) => () => t;
function g4(t, r) {
  return function(i) {
    return t + i * r;
  };
}
function v4(t, r, i) {
  return t = Math.pow(t, i), r = Math.pow(r, i) - t, i = 1 / i, function(u) {
    return Math.pow(t + u * r, i);
  };
}
function p4(t) {
  return (t = +t) == 1 ? uM : function(r, i) {
    return i - r ? v4(r, i, t) : rm(isNaN(r) ? i : r);
  };
}
function uM(t, r) {
  var i = r - t;
  return i ? g4(t, i) : rm(isNaN(t) ? r : t);
}
const Uo = (function t(r) {
  var i = p4(r);
  function u(l, s) {
    var c = i((l = xy(l)).r, (s = xy(s)).r), f = i(l.g, s.g), g = i(l.b, s.b), h = uM(l.opacity, s.opacity);
    return function(v) {
      return l.r = c(v), l.g = f(v), l.b = g(v), l.opacity = h(v), l + "";
    };
  }
  return u.gamma = t, u;
})(1);
function y4(t, r) {
  r || (r = []);
  var i = t ? Math.min(r.length, t.length) : 0, u = r.slice(), l;
  return function(s) {
    for (l = 0; l < i; ++l) u[l] = t[l] * (1 - s) + r[l] * s;
    return u;
  };
}
function m4(t) {
  return ArrayBuffer.isView(t) && !(t instanceof DataView);
}
function b4(t, r) {
  var i = r ? r.length : 0, u = t ? Math.min(i, t.length) : 0, l = new Array(u), s = new Array(i), c;
  for (c = 0; c < u; ++c) l[c] = Tu(t[c], r[c]);
  for (; c < i; ++c) s[c] = r[c];
  return function(f) {
    for (c = 0; c < u; ++c) s[c] = l[c](f);
    return s;
  };
}
function _4(t, r) {
  var i = /* @__PURE__ */ new Date();
  return t = +t, r = +r, function(u) {
    return i.setTime(t * (1 - u) + r * u), i;
  };
}
function wn(t, r) {
  return t = +t, r = +r, function(i) {
    return t * (1 - i) + r * i;
  };
}
function x4(t, r) {
  var i = {}, u = {}, l;
  (t === null || typeof t != "object") && (t = {}), (r === null || typeof r != "object") && (r = {});
  for (l in r)
    l in t ? i[l] = Tu(t[l], r[l]) : u[l] = r[l];
  return function(s) {
    for (l in i) u[l] = i[l](s);
    return u;
  };
}
var Sy = /[-+]?(?:\d+\.?\d*|\.?\d+)(?:[eE][-+]?\d+)?/g, oy = new RegExp(Sy.source, "g");
function S4(t) {
  return function() {
    return t;
  };
}
function E4(t) {
  return function(r) {
    return t(r) + "";
  };
}
function lM(t, r) {
  var i = Sy.lastIndex = oy.lastIndex = 0, u, l, s, c = -1, f = [], g = [];
  for (t = t + "", r = r + ""; (u = Sy.exec(t)) && (l = oy.exec(r)); )
    (s = l.index) > i && (s = r.slice(i, s), f[c] ? f[c] += s : f[++c] = s), (u = u[0]) === (l = l[0]) ? f[c] ? f[c] += l : f[++c] = l : (f[++c] = null, g.push({ i: c, x: wn(u, l) })), i = oy.lastIndex;
  return i < r.length && (s = r.slice(i), f[c] ? f[c] += s : f[++c] = s), f.length < 2 ? g[0] ? E4(g[0].x) : S4(r) : (r = g.length, function(h) {
    for (var v = 0, y; v < r; ++v) f[(y = g[v]).i] = y.x(h);
    return f.join("");
  });
}
function Tu(t, r) {
  var i = typeof r, u;
  return r == null || i === "boolean" ? rm(r) : (i === "number" ? wn : i === "string" ? (u = ua(r)) ? (r = u, Uo) : lM : r instanceof ua ? Uo : r instanceof Date ? _4 : m4(r) ? y4 : Array.isArray(r) ? b4 : typeof r.valueOf != "function" && typeof r.toString != "function" || isNaN(r) ? x4 : wn)(t, r);
}
var V2 = 180 / Math.PI, Ey = {
  translateX: 0,
  translateY: 0,
  rotate: 0,
  skewX: 0,
  scaleX: 1,
  scaleY: 1
};
function oM(t, r, i, u, l, s) {
  var c, f, g;
  return (c = Math.sqrt(t * t + r * r)) && (t /= c, r /= c), (g = t * i + r * u) && (i -= t * g, u -= r * g), (f = Math.sqrt(i * i + u * u)) && (i /= f, u /= f, g /= f), t * u < r * i && (t = -t, r = -r, g = -g, c = -c), {
    translateX: l,
    translateY: s,
    rotate: Math.atan2(r, t) * V2,
    skewX: Math.atan(g) * V2,
    scaleX: c,
    scaleY: f
  };
}
var Eo;
function w4(t) {
  const r = new (typeof DOMMatrix == "function" ? DOMMatrix : WebKitCSSMatrix)(t + "");
  return r.isIdentity ? Ey : oM(r.a, r.b, r.c, r.d, r.e, r.f);
}
function A4(t) {
  return t == null || (Eo || (Eo = document.createElementNS("http://www.w3.org/2000/svg", "g")), Eo.setAttribute("transform", t), !(t = Eo.transform.baseVal.consolidate())) ? Ey : (t = t.matrix, oM(t.a, t.b, t.c, t.d, t.e, t.f));
}
function sM(t, r, i, u) {
  function l(h) {
    return h.length ? h.pop() + " " : "";
  }
  function s(h, v, y, m, b, _) {
    if (h !== y || v !== m) {
      var A = b.push("translate(", null, r, null, i);
      _.push({ i: A - 4, x: wn(h, y) }, { i: A - 2, x: wn(v, m) });
    } else (y || m) && b.push("translate(" + y + r + m + i);
  }
  function c(h, v, y, m) {
    h !== v ? (h - v > 180 ? v += 360 : v - h > 180 && (h += 360), m.push({ i: y.push(l(y) + "rotate(", null, u) - 2, x: wn(h, v) })) : v && y.push(l(y) + "rotate(" + v + u);
  }
  function f(h, v, y, m) {
    h !== v ? m.push({ i: y.push(l(y) + "skewX(", null, u) - 2, x: wn(h, v) }) : v && y.push(l(y) + "skewX(" + v + u);
  }
  function g(h, v, y, m, b, _) {
    if (h !== y || v !== m) {
      var A = b.push(l(b) + "scale(", null, ",", null, ")");
      _.push({ i: A - 4, x: wn(h, y) }, { i: A - 2, x: wn(v, m) });
    } else (y !== 1 || m !== 1) && b.push(l(b) + "scale(" + y + "," + m + ")");
  }
  return function(h, v) {
    var y = [], m = [];
    return h = t(h), v = t(v), s(h.translateX, h.translateY, v.translateX, v.translateY, y, m), c(h.rotate, v.rotate, y, m), f(h.skewX, v.skewX, y, m), g(h.scaleX, h.scaleY, v.scaleX, v.scaleY, y, m), h = v = null, function(b) {
      for (var _ = -1, A = m.length, w; ++_ < A; ) y[(w = m[_]).i] = w.x(b);
      return y.join("");
    };
  };
}
var T4 = sM(w4, "px, ", "px)", "deg)"), M4 = sM(A4, ", ", ")", ")"), C4 = 1e-12;
function Y2(t) {
  return ((t = Math.exp(t)) + 1 / t) / 2;
}
function q4(t) {
  return ((t = Math.exp(t)) - 1 / t) / 2;
}
function R4(t) {
  return ((t = Math.exp(2 * t)) - 1) / (t + 1);
}
const No = (function t(r, i, u) {
  function l(s, c) {
    var f = s[0], g = s[1], h = s[2], v = c[0], y = c[1], m = c[2], b = v - f, _ = y - g, A = b * b + _ * _, w, E;
    if (A < C4)
      E = Math.log(m / h) / r, w = function(q) {
        return [
          f + q * b,
          g + q * _,
          h * Math.exp(r * q * E)
        ];
      };
    else {
      var M = Math.sqrt(A), S = (m * m - h * h + u * A) / (2 * h * i * M), T = (m * m - h * h - u * A) / (2 * m * i * M), N = Math.log(Math.sqrt(S * S + 1) - S), R = Math.log(Math.sqrt(T * T + 1) - T);
      E = (R - N) / r, w = function(q) {
        var L = q * E, j = Y2(N), Z = h / (i * M) * (j * R4(r * L + N) - q4(N));
        return [
          f + Z * b,
          g + Z * _,
          h * j / Y2(r * L + N)
        ];
      };
    }
    return w.duration = E * 1e3 * r / Math.SQRT2, w;
  }
  return l.rho = function(s) {
    var c = Math.max(1e-3, +s), f = c * c, g = f * f;
    return t(c, f, g);
  }, l;
})(Math.SQRT2, 2, 4);
var si = 0, wu = 0, bu = 0, cM = 1e3, Go, Au, Vo = 0, la = 0, ds = 0, Nu = typeof performance == "object" && performance.now ? performance : Date, fM = typeof window == "object" && window.requestAnimationFrame ? window.requestAnimationFrame.bind(window) : function(t) {
  setTimeout(t, 17);
};
function am() {
  return la || (fM(N4), la = Nu.now() + ds);
}
function N4() {
  la = 0;
}
function Yo() {
  this._call = this._time = this._next = null;
}
Yo.prototype = dM.prototype = {
  constructor: Yo,
  restart: function(t, r, i) {
    if (typeof t != "function") throw new TypeError("callback is not a function");
    i = (i == null ? am() : +i) + (r == null ? 0 : +r), !this._next && Au !== this && (Au ? Au._next = this : Go = this, Au = this), this._call = t, this._time = i, wy();
  },
  stop: function() {
    this._call && (this._call = null, this._time = 1 / 0, wy());
  }
};
function dM(t, r, i) {
  var u = new Yo();
  return u.restart(t, r, i), u;
}
function z4() {
  am(), ++si;
  for (var t = Go, r; t; )
    (r = la - t._time) >= 0 && t._call.call(void 0, r), t = t._next;
  --si;
}
function k2() {
  la = (Vo = Nu.now()) + ds, si = wu = 0;
  try {
    z4();
  } finally {
    si = 0, D4(), la = 0;
  }
}
function O4() {
  var t = Nu.now(), r = t - Vo;
  r > cM && (ds -= r, Vo = t);
}
function D4() {
  for (var t, r = Go, i, u = 1 / 0; r; )
    r._call ? (u > r._time && (u = r._time), t = r, r = r._next) : (i = r._next, r._next = null, r = t ? t._next = i : Go = i);
  Au = t, wy(u);
}
function wy(t) {
  if (!si) {
    wu && (wu = clearTimeout(wu));
    var r = t - la;
    r > 24 ? (t < 1 / 0 && (wu = setTimeout(k2, t - Nu.now() - ds)), bu && (bu = clearInterval(bu))) : (bu || (Vo = Nu.now(), bu = setInterval(O4, cM)), si = 1, fM(k2));
  }
}
function X2(t, r, i) {
  var u = new Yo();
  return r = r == null ? 0 : +r, u.restart((l) => {
    u.stop(), t(l + r);
  }, r, i), u;
}
var L4 = cs("start", "end", "cancel", "interrupt"), H4 = [], hM = 0, I2 = 1, Ay = 2, zo = 3, Z2 = 4, Ty = 5, Oo = 6;
function hs(t, r, i, u, l, s) {
  var c = t.__transition;
  if (!c) t.__transition = {};
  else if (i in c) return;
  j4(t, i, {
    name: r,
    index: u,
    // For context during callback.
    group: l,
    // For context during callback.
    on: L4,
    tween: H4,
    time: s.time,
    delay: s.delay,
    duration: s.duration,
    ease: s.ease,
    timer: null,
    state: hM
  });
}
function im(t, r) {
  var i = yn(t, r);
  if (i.state > hM) throw new Error("too late; already scheduled");
  return i;
}
function Mn(t, r) {
  var i = yn(t, r);
  if (i.state > zo) throw new Error("too late; already running");
  return i;
}
function yn(t, r) {
  var i = t.__transition;
  if (!i || !(i = i[r])) throw new Error("transition not found");
  return i;
}
function j4(t, r, i) {
  var u = t.__transition, l;
  u[r] = i, i.timer = dM(s, 0, i.time);
  function s(h) {
    i.state = I2, i.timer.restart(c, i.delay, i.time), i.delay <= h && c(h - i.delay);
  }
  function c(h) {
    var v, y, m, b;
    if (i.state !== I2) return g();
    for (v in u)
      if (b = u[v], b.name === i.name) {
        if (b.state === zo) return X2(c);
        b.state === Z2 ? (b.state = Oo, b.timer.stop(), b.on.call("interrupt", t, t.__data__, b.index, b.group), delete u[v]) : +v < r && (b.state = Oo, b.timer.stop(), b.on.call("cancel", t, t.__data__, b.index, b.group), delete u[v]);
      }
    if (X2(function() {
      i.state === zo && (i.state = Z2, i.timer.restart(f, i.delay, i.time), f(h));
    }), i.state = Ay, i.on.call("start", t, t.__data__, i.index, i.group), i.state === Ay) {
      for (i.state = zo, l = new Array(m = i.tween.length), v = 0, y = -1; v < m; ++v)
        (b = i.tween[v].value.call(t, t.__data__, i.index, i.group)) && (l[++y] = b);
      l.length = y + 1;
    }
  }
  function f(h) {
    for (var v = h < i.duration ? i.ease.call(null, h / i.duration) : (i.timer.restart(g), i.state = Ty, 1), y = -1, m = l.length; ++y < m; )
      l[y].call(t, v);
    i.state === Ty && (i.on.call("end", t, t.__data__, i.index, i.group), g());
  }
  function g() {
    i.state = Oo, i.timer.stop(), delete u[r];
    for (var h in u) return;
    delete t.__transition;
  }
}
function Do(t, r) {
  var i = t.__transition, u, l, s = !0, c;
  if (i) {
    r = r == null ? null : r + "";
    for (c in i) {
      if ((u = i[c]).name !== r) {
        s = !1;
        continue;
      }
      l = u.state > Ay && u.state < Ty, u.state = Oo, u.timer.stop(), u.on.call(l ? "interrupt" : "cancel", t, t.__data__, u.index, u.group), delete i[c];
    }
    s && delete t.__transition;
  }
}
function B4(t) {
  return this.each(function() {
    Do(this, t);
  });
}
function U4(t, r) {
  var i, u;
  return function() {
    var l = Mn(this, t), s = l.tween;
    if (s !== i) {
      u = i = s;
      for (var c = 0, f = u.length; c < f; ++c)
        if (u[c].name === r) {
          u = u.slice(), u.splice(c, 1);
          break;
        }
    }
    l.tween = u;
  };
}
function G4(t, r, i) {
  var u, l;
  if (typeof i != "function") throw new Error();
  return function() {
    var s = Mn(this, t), c = s.tween;
    if (c !== u) {
      l = (u = c).slice();
      for (var f = { name: r, value: i }, g = 0, h = l.length; g < h; ++g)
        if (l[g].name === r) {
          l[g] = f;
          break;
        }
      g === h && l.push(f);
    }
    s.tween = l;
  };
}
function V4(t, r) {
  var i = this._id;
  if (t += "", arguments.length < 2) {
    for (var u = yn(this.node(), i).tween, l = 0, s = u.length, c; l < s; ++l)
      if ((c = u[l]).name === t)
        return c.value;
    return null;
  }
  return this.each((r == null ? U4 : G4)(i, t, r));
}
function um(t, r, i) {
  var u = t._id;
  return t.each(function() {
    var l = Mn(this, u);
    (l.value || (l.value = {}))[r] = i.apply(this, arguments);
  }), function(l) {
    return yn(l, u).value[r];
  };
}
function gM(t, r) {
  var i;
  return (typeof r == "number" ? wn : r instanceof ua ? Uo : (i = ua(r)) ? (r = i, Uo) : lM)(t, r);
}
function Y4(t) {
  return function() {
    this.removeAttribute(t);
  };
}
function k4(t) {
  return function() {
    this.removeAttributeNS(t.space, t.local);
  };
}
function X4(t, r, i) {
  var u, l = i + "", s;
  return function() {
    var c = this.getAttribute(t);
    return c === l ? null : c === u ? s : s = r(u = c, i);
  };
}
function I4(t, r, i) {
  var u, l = i + "", s;
  return function() {
    var c = this.getAttributeNS(t.space, t.local);
    return c === l ? null : c === u ? s : s = r(u = c, i);
  };
}
function Z4(t, r, i) {
  var u, l, s;
  return function() {
    var c, f = i(this), g;
    return f == null ? void this.removeAttribute(t) : (c = this.getAttribute(t), g = f + "", c === g ? null : c === u && g === l ? s : (l = g, s = r(u = c, f)));
  };
}
function K4(t, r, i) {
  var u, l, s;
  return function() {
    var c, f = i(this), g;
    return f == null ? void this.removeAttributeNS(t.space, t.local) : (c = this.getAttributeNS(t.space, t.local), g = f + "", c === g ? null : c === u && g === l ? s : (l = g, s = r(u = c, f)));
  };
}
function Q4(t, r) {
  var i = fs(t), u = i === "transform" ? M4 : gM;
  return this.attrTween(t, typeof r == "function" ? (i.local ? K4 : Z4)(i, u, um(this, "attr." + t, r)) : r == null ? (i.local ? k4 : Y4)(i) : (i.local ? I4 : X4)(i, u, r));
}
function $4(t, r) {
  return function(i) {
    this.setAttribute(t, r.call(this, i));
  };
}
function F4(t, r) {
  return function(i) {
    this.setAttributeNS(t.space, t.local, r.call(this, i));
  };
}
function J4(t, r) {
  var i, u;
  function l() {
    var s = r.apply(this, arguments);
    return s !== u && (i = (u = s) && F4(t, s)), i;
  }
  return l._value = r, l;
}
function P4(t, r) {
  var i, u;
  function l() {
    var s = r.apply(this, arguments);
    return s !== u && (i = (u = s) && $4(t, s)), i;
  }
  return l._value = r, l;
}
function W4(t, r) {
  var i = "attr." + t;
  if (arguments.length < 2) return (i = this.tween(i)) && i._value;
  if (r == null) return this.tween(i, null);
  if (typeof r != "function") throw new Error();
  var u = fs(t);
  return this.tween(i, (u.local ? J4 : P4)(u, r));
}
function e5(t, r) {
  return function() {
    im(this, t).delay = +r.apply(this, arguments);
  };
}
function t5(t, r) {
  return r = +r, function() {
    im(this, t).delay = r;
  };
}
function n5(t) {
  var r = this._id;
  return arguments.length ? this.each((typeof t == "function" ? e5 : t5)(r, t)) : yn(this.node(), r).delay;
}
function r5(t, r) {
  return function() {
    Mn(this, t).duration = +r.apply(this, arguments);
  };
}
function a5(t, r) {
  return r = +r, function() {
    Mn(this, t).duration = r;
  };
}
function i5(t) {
  var r = this._id;
  return arguments.length ? this.each((typeof t == "function" ? r5 : a5)(r, t)) : yn(this.node(), r).duration;
}
function u5(t, r) {
  if (typeof r != "function") throw new Error();
  return function() {
    Mn(this, t).ease = r;
  };
}
function l5(t) {
  var r = this._id;
  return arguments.length ? this.each(u5(r, t)) : yn(this.node(), r).ease;
}
function o5(t, r) {
  return function() {
    var i = r.apply(this, arguments);
    if (typeof i != "function") throw new Error();
    Mn(this, t).ease = i;
  };
}
function s5(t) {
  if (typeof t != "function") throw new Error();
  return this.each(o5(this._id, t));
}
function c5(t) {
  typeof t != "function" && (t = IT(t));
  for (var r = this._groups, i = r.length, u = new Array(i), l = 0; l < i; ++l)
    for (var s = r[l], c = s.length, f = u[l] = [], g, h = 0; h < c; ++h)
      (g = s[h]) && t.call(g, g.__data__, h, s) && f.push(g);
  return new Pn(u, this._parents, this._name, this._id);
}
function f5(t) {
  if (t._id !== this._id) throw new Error();
  for (var r = this._groups, i = t._groups, u = r.length, l = i.length, s = Math.min(u, l), c = new Array(u), f = 0; f < s; ++f)
    for (var g = r[f], h = i[f], v = g.length, y = c[f] = new Array(v), m, b = 0; b < v; ++b)
      (m = g[b] || h[b]) && (y[b] = m);
  for (; f < u; ++f)
    c[f] = r[f];
  return new Pn(c, this._parents, this._name, this._id);
}
function d5(t) {
  return (t + "").trim().split(/^|\s+/).every(function(r) {
    var i = r.indexOf(".");
    return i >= 0 && (r = r.slice(0, i)), !r || r === "start";
  });
}
function h5(t, r, i) {
  var u, l, s = d5(r) ? im : Mn;
  return function() {
    var c = s(this, t), f = c.on;
    f !== u && (l = (u = f).copy()).on(r, i), c.on = l;
  };
}
function g5(t, r) {
  var i = this._id;
  return arguments.length < 2 ? yn(this.node(), i).on.on(t) : this.each(h5(i, t, r));
}
function v5(t) {
  return function() {
    var r = this.parentNode;
    for (var i in this.__transition) if (+i !== t) return;
    r && r.removeChild(this);
  };
}
function p5() {
  return this.on("end.remove", v5(this._id));
}
function y5(t) {
  var r = this._name, i = this._id;
  typeof t != "function" && (t = em(t));
  for (var u = this._groups, l = u.length, s = new Array(l), c = 0; c < l; ++c)
    for (var f = u[c], g = f.length, h = s[c] = new Array(g), v, y, m = 0; m < g; ++m)
      (v = f[m]) && (y = t.call(v, v.__data__, m, f)) && ("__data__" in v && (y.__data__ = v.__data__), h[m] = y, hs(h[m], r, i, m, h, yn(v, i)));
  return new Pn(s, this._parents, r, i);
}
function m5(t) {
  var r = this._name, i = this._id;
  typeof t != "function" && (t = XT(t));
  for (var u = this._groups, l = u.length, s = [], c = [], f = 0; f < l; ++f)
    for (var g = u[f], h = g.length, v, y = 0; y < h; ++y)
      if (v = g[y]) {
        for (var m = t.call(v, v.__data__, y, g), b, _ = yn(v, i), A = 0, w = m.length; A < w; ++A)
          (b = m[A]) && hs(b, r, i, A, m, _);
        s.push(m), c.push(v);
      }
  return new Pn(s, c, r, i);
}
var b5 = ku.prototype.constructor;
function _5() {
  return new b5(this._groups, this._parents);
}
function x5(t, r) {
  var i, u, l;
  return function() {
    var s = oi(this, t), c = (this.style.removeProperty(t), oi(this, t));
    return s === c ? null : s === i && c === u ? l : l = r(i = s, u = c);
  };
}
function vM(t) {
  return function() {
    this.style.removeProperty(t);
  };
}
function S5(t, r, i) {
  var u, l = i + "", s;
  return function() {
    var c = oi(this, t);
    return c === l ? null : c === u ? s : s = r(u = c, i);
  };
}
function E5(t, r, i) {
  var u, l, s;
  return function() {
    var c = oi(this, t), f = i(this), g = f + "";
    return f == null && (g = f = (this.style.removeProperty(t), oi(this, t))), c === g ? null : c === u && g === l ? s : (l = g, s = r(u = c, f));
  };
}
function w5(t, r) {
  var i, u, l, s = "style." + r, c = "end." + s, f;
  return function() {
    var g = Mn(this, t), h = g.on, v = g.value[s] == null ? f || (f = vM(r)) : void 0;
    (h !== i || l !== v) && (u = (i = h).copy()).on(c, l = v), g.on = u;
  };
}
function A5(t, r, i) {
  var u = (t += "") == "transform" ? T4 : gM;
  return r == null ? this.styleTween(t, x5(t, u)).on("end.style." + t, vM(t)) : typeof r == "function" ? this.styleTween(t, E5(t, u, um(this, "style." + t, r))).each(w5(this._id, t)) : this.styleTween(t, S5(t, u, r), i).on("end.style." + t, null);
}
function T5(t, r, i) {
  return function(u) {
    this.style.setProperty(t, r.call(this, u), i);
  };
}
function M5(t, r, i) {
  var u, l;
  function s() {
    var c = r.apply(this, arguments);
    return c !== l && (u = (l = c) && T5(t, c, i)), u;
  }
  return s._value = r, s;
}
function C5(t, r, i) {
  var u = "style." + (t += "");
  if (arguments.length < 2) return (u = this.tween(u)) && u._value;
  if (r == null) return this.tween(u, null);
  if (typeof r != "function") throw new Error();
  return this.tween(u, M5(t, r, i ?? ""));
}
function q5(t) {
  return function() {
    this.textContent = t;
  };
}
function R5(t) {
  return function() {
    var r = t(this);
    this.textContent = r ?? "";
  };
}
function N5(t) {
  return this.tween("text", typeof t == "function" ? R5(um(this, "text", t)) : q5(t == null ? "" : t + ""));
}
function z5(t) {
  return function(r) {
    this.textContent = t.call(this, r);
  };
}
function O5(t) {
  var r, i;
  function u() {
    var l = t.apply(this, arguments);
    return l !== i && (r = (i = l) && z5(l)), r;
  }
  return u._value = t, u;
}
function D5(t) {
  var r = "text";
  if (arguments.length < 1) return (r = this.tween(r)) && r._value;
  if (t == null) return this.tween(r, null);
  if (typeof t != "function") throw new Error();
  return this.tween(r, O5(t));
}
function L5() {
  for (var t = this._name, r = this._id, i = pM(), u = this._groups, l = u.length, s = 0; s < l; ++s)
    for (var c = u[s], f = c.length, g, h = 0; h < f; ++h)
      if (g = c[h]) {
        var v = yn(g, r);
        hs(g, t, i, h, c, {
          time: v.time + v.delay + v.duration,
          delay: 0,
          duration: v.duration,
          ease: v.ease
        });
      }
  return new Pn(u, this._parents, t, i);
}
function H5() {
  var t, r, i = this, u = i._id, l = i.size();
  return new Promise(function(s, c) {
    var f = { value: c }, g = { value: function() {
      --l === 0 && s();
    } };
    i.each(function() {
      var h = Mn(this, u), v = h.on;
      v !== t && (r = (t = v).copy(), r._.cancel.push(f), r._.interrupt.push(f), r._.end.push(g)), h.on = r;
    }), l === 0 && s();
  });
}
var j5 = 0;
function Pn(t, r, i, u) {
  this._groups = t, this._parents = r, this._name = i, this._id = u;
}
function pM() {
  return ++j5;
}
var Fn = ku.prototype;
Pn.prototype = {
  constructor: Pn,
  select: y5,
  selectAll: m5,
  selectChild: Fn.selectChild,
  selectChildren: Fn.selectChildren,
  filter: c5,
  merge: f5,
  selection: _5,
  transition: L5,
  call: Fn.call,
  nodes: Fn.nodes,
  node: Fn.node,
  size: Fn.size,
  empty: Fn.empty,
  each: Fn.each,
  on: g5,
  attr: Q4,
  attrTween: W4,
  style: A5,
  styleTween: C5,
  text: N5,
  textTween: D5,
  remove: p5,
  tween: V4,
  delay: n5,
  duration: i5,
  ease: l5,
  easeVarying: s5,
  end: H5,
  [Symbol.iterator]: Fn[Symbol.iterator]
};
function B5(t) {
  return ((t *= 2) <= 1 ? t * t * t : (t -= 2) * t * t + 2) / 2;
}
var U5 = {
  time: null,
  // Set on use.
  delay: 0,
  duration: 250,
  ease: B5
};
function G5(t, r) {
  for (var i; !(i = t.__transition) || !(i = i[r]); )
    if (!(t = t.parentNode))
      throw new Error(`transition ${r} not found`);
  return i;
}
function V5(t) {
  var r, i;
  t instanceof Pn ? (r = t._id, t = t._name) : (r = pM(), (i = U5).time = am(), t = t == null ? null : t + "");
  for (var u = this._groups, l = u.length, s = 0; s < l; ++s)
    for (var c = u[s], f = c.length, g, h = 0; h < f; ++h)
      (g = c[h]) && hs(g, t, r, h, c, i || G5(g, r));
  return new Pn(u, this._parents, t, r);
}
ku.prototype.interrupt = B4;
ku.prototype.transition = V5;
const wo = (t) => () => t;
function Y5(t, {
  sourceEvent: r,
  target: i,
  transform: u,
  dispatch: l
}) {
  Object.defineProperties(this, {
    type: { value: t, enumerable: !0, configurable: !0 },
    sourceEvent: { value: r, enumerable: !0, configurable: !0 },
    target: { value: i, enumerable: !0, configurable: !0 },
    transform: { value: u, enumerable: !0, configurable: !0 },
    _: { value: l }
  });
}
function Jn(t, r, i) {
  this.k = t, this.x = r, this.y = i;
}
Jn.prototype = {
  constructor: Jn,
  scale: function(t) {
    return t === 1 ? this : new Jn(this.k * t, this.x, this.y);
  },
  translate: function(t, r) {
    return t === 0 & r === 0 ? this : new Jn(this.k, this.x + this.k * t, this.y + this.k * r);
  },
  apply: function(t) {
    return [t[0] * this.k + this.x, t[1] * this.k + this.y];
  },
  applyX: function(t) {
    return t * this.k + this.x;
  },
  applyY: function(t) {
    return t * this.k + this.y;
  },
  invert: function(t) {
    return [(t[0] - this.x) / this.k, (t[1] - this.y) / this.k];
  },
  invertX: function(t) {
    return (t - this.x) / this.k;
  },
  invertY: function(t) {
    return (t - this.y) / this.k;
  },
  rescaleX: function(t) {
    return t.copy().domain(t.range().map(this.invertX, this).map(t.invert, t));
  },
  rescaleY: function(t) {
    return t.copy().domain(t.range().map(this.invertY, this).map(t.invert, t));
  },
  toString: function() {
    return "translate(" + this.x + "," + this.y + ") scale(" + this.k + ")";
  }
};
var gs = new Jn(1, 0, 0);
yM.prototype = Jn.prototype;
function yM(t) {
  for (; !t.__zoom; ) if (!(t = t.parentNode)) return gs;
  return t.__zoom;
}
function sy(t) {
  t.stopImmediatePropagation();
}
function _u(t) {
  t.preventDefault(), t.stopImmediatePropagation();
}
function k5(t) {
  return (!t.ctrlKey || t.type === "wheel") && !t.button;
}
function X5() {
  var t = this;
  return t instanceof SVGElement ? (t = t.ownerSVGElement || t, t.hasAttribute("viewBox") ? (t = t.viewBox.baseVal, [[t.x, t.y], [t.x + t.width, t.y + t.height]]) : [[0, 0], [t.width.baseVal.value, t.height.baseVal.value]]) : [[0, 0], [t.clientWidth, t.clientHeight]];
}
function K2() {
  return this.__zoom || gs;
}
function I5(t) {
  return -t.deltaY * (t.deltaMode === 1 ? 0.05 : t.deltaMode ? 1 : 2e-3) * (t.ctrlKey ? 10 : 1);
}
function Z5() {
  return navigator.maxTouchPoints || "ontouchstart" in this;
}
function K5(t, r, i) {
  var u = t.invertX(r[0][0]) - i[0][0], l = t.invertX(r[1][0]) - i[1][0], s = t.invertY(r[0][1]) - i[0][1], c = t.invertY(r[1][1]) - i[1][1];
  return t.translate(
    l > u ? (u + l) / 2 : Math.min(0, u) || Math.max(0, l),
    c > s ? (s + c) / 2 : Math.min(0, s) || Math.max(0, c)
  );
}
function mM() {
  var t = k5, r = X5, i = K5, u = I5, l = Z5, s = [0, 1 / 0], c = [[-1 / 0, -1 / 0], [1 / 0, 1 / 0]], f = 250, g = No, h = cs("start", "zoom", "end"), v, y, m, b = 500, _ = 150, A = 0, w = 10;
  function E(D) {
    D.property("__zoom", K2).on("wheel.zoom", L, { passive: !1 }).on("mousedown.zoom", j).on("dblclick.zoom", Z).filter(l).on("touchstart.zoom", Y).on("touchmove.zoom", J).on("touchend.zoom touchcancel.zoom", Q).style("-webkit-tap-highlight-color", "rgba(0,0,0,0)");
  }
  E.transform = function(D, X, z, B) {
    var $ = D.selection ? D.selection() : D;
    $.property("__zoom", K2), D !== $ ? N(D, X, z, B) : $.interrupt().each(function() {
      R(this, arguments).event(B).start().zoom(null, typeof X == "function" ? X.apply(this, arguments) : X).end();
    });
  }, E.scaleBy = function(D, X, z, B) {
    E.scaleTo(D, function() {
      var $ = this.__zoom.k, U = typeof X == "function" ? X.apply(this, arguments) : X;
      return $ * U;
    }, z, B);
  }, E.scaleTo = function(D, X, z, B) {
    E.transform(D, function() {
      var $ = r.apply(this, arguments), U = this.__zoom, ae = z == null ? T($) : typeof z == "function" ? z.apply(this, arguments) : z, O = U.invert(ae), G = typeof X == "function" ? X.apply(this, arguments) : X;
      return i(S(M(U, G), ae, O), $, c);
    }, z, B);
  }, E.translateBy = function(D, X, z, B) {
    E.transform(D, function() {
      return i(this.__zoom.translate(
        typeof X == "function" ? X.apply(this, arguments) : X,
        typeof z == "function" ? z.apply(this, arguments) : z
      ), r.apply(this, arguments), c);
    }, null, B);
  }, E.translateTo = function(D, X, z, B, $) {
    E.transform(D, function() {
      var U = r.apply(this, arguments), ae = this.__zoom, O = B == null ? T(U) : typeof B == "function" ? B.apply(this, arguments) : B;
      return i(gs.translate(O[0], O[1]).scale(ae.k).translate(
        typeof X == "function" ? -X.apply(this, arguments) : -X,
        typeof z == "function" ? -z.apply(this, arguments) : -z
      ), U, c);
    }, B, $);
  };
  function M(D, X) {
    return X = Math.max(s[0], Math.min(s[1], X)), X === D.k ? D : new Jn(X, D.x, D.y);
  }
  function S(D, X, z) {
    var B = X[0] - z[0] * D.k, $ = X[1] - z[1] * D.k;
    return B === D.x && $ === D.y ? D : new Jn(D.k, B, $);
  }
  function T(D) {
    return [(+D[0][0] + +D[1][0]) / 2, (+D[0][1] + +D[1][1]) / 2];
  }
  function N(D, X, z, B) {
    D.on("start.zoom", function() {
      R(this, arguments).event(B).start();
    }).on("interrupt.zoom end.zoom", function() {
      R(this, arguments).event(B).end();
    }).tween("zoom", function() {
      var $ = this, U = arguments, ae = R($, U).event(B), O = r.apply($, U), G = z == null ? T(O) : typeof z == "function" ? z.apply($, U) : z, ie = Math.max(O[1][0] - O[0][0], O[1][1] - O[0][1]), H = $.__zoom, K = typeof X == "function" ? X.apply($, U) : X, P = g(H.invert(G).concat(ie / H.k), K.invert(G).concat(ie / K.k));
      return function(ue) {
        if (ue === 1) ue = K;
        else {
          var W = P(ue), se = ie / W[2];
          ue = new Jn(se, G[0] - W[0] * se, G[1] - W[1] * se);
        }
        ae.zoom(null, ue);
      };
    });
  }
  function R(D, X, z) {
    return !z && D.__zooming || new q(D, X);
  }
  function q(D, X) {
    this.that = D, this.args = X, this.active = 0, this.sourceEvent = null, this.extent = r.apply(D, X), this.taps = 0;
  }
  q.prototype = {
    event: function(D) {
      return D && (this.sourceEvent = D), this;
    },
    start: function() {
      return ++this.active === 1 && (this.that.__zooming = this, this.emit("start")), this;
    },
    zoom: function(D, X) {
      return this.mouse && D !== "mouse" && (this.mouse[1] = X.invert(this.mouse[0])), this.touch0 && D !== "touch" && (this.touch0[1] = X.invert(this.touch0[0])), this.touch1 && D !== "touch" && (this.touch1[1] = X.invert(this.touch1[0])), this.that.__zoom = X, this.emit("zoom"), this;
    },
    end: function() {
      return --this.active === 0 && (delete this.that.__zooming, this.emit("end")), this;
    },
    emit: function(D) {
      var X = Kt(this.that).datum();
      h.call(
        D,
        this.that,
        new Y5(D, {
          sourceEvent: this.sourceEvent,
          target: E,
          transform: this.that.__zoom,
          dispatch: h
        }),
        X
      );
    }
  };
  function L(D, ...X) {
    if (!t.apply(this, arguments)) return;
    var z = R(this, X).event(D), B = this.__zoom, $ = Math.max(s[0], Math.min(s[1], B.k * Math.pow(2, u.apply(this, arguments)))), U = cn(D);
    if (z.wheel)
      (z.mouse[0][0] !== U[0] || z.mouse[0][1] !== U[1]) && (z.mouse[1] = B.invert(z.mouse[0] = U)), clearTimeout(z.wheel);
    else {
      if (B.k === $) return;
      z.mouse = [U, B.invert(U)], Do(this), z.start();
    }
    _u(D), z.wheel = setTimeout(ae, _), z.zoom("mouse", i(S(M(B, $), z.mouse[0], z.mouse[1]), z.extent, c));
    function ae() {
      z.wheel = null, z.end();
    }
  }
  function j(D, ...X) {
    if (m || !t.apply(this, arguments)) return;
    var z = D.currentTarget, B = R(this, X, !0).event(D), $ = Kt(D.view).on("mousemove.zoom", G, !0).on("mouseup.zoom", ie, !0), U = cn(D, z), ae = D.clientX, O = D.clientY;
    tM(D.view), sy(D), B.mouse = [U, this.__zoom.invert(U)], Do(this), B.start();
    function G(H) {
      if (_u(H), !B.moved) {
        var K = H.clientX - ae, P = H.clientY - O;
        B.moved = K * K + P * P > A;
      }
      B.event(H).zoom("mouse", i(S(B.that.__zoom, B.mouse[0] = cn(H, z), B.mouse[1]), B.extent, c));
    }
    function ie(H) {
      $.on("mousemove.zoom mouseup.zoom", null), nM(H.view, B.moved), _u(H), B.event(H).end();
    }
  }
  function Z(D, ...X) {
    if (t.apply(this, arguments)) {
      var z = this.__zoom, B = cn(D.changedTouches ? D.changedTouches[0] : D, this), $ = z.invert(B), U = z.k * (D.shiftKey ? 0.5 : 2), ae = i(S(M(z, U), B, $), r.apply(this, X), c);
      _u(D), f > 0 ? Kt(this).transition().duration(f).call(N, ae, B, D) : Kt(this).call(E.transform, ae, B, D);
    }
  }
  function Y(D, ...X) {
    if (t.apply(this, arguments)) {
      var z = D.touches, B = z.length, $ = R(this, X, D.changedTouches.length === B).event(D), U, ae, O, G;
      for (sy(D), ae = 0; ae < B; ++ae)
        O = z[ae], G = cn(O, this), G = [G, this.__zoom.invert(G), O.identifier], $.touch0 ? !$.touch1 && $.touch0[2] !== G[2] && ($.touch1 = G, $.taps = 0) : ($.touch0 = G, U = !0, $.taps = 1 + !!v);
      v && (v = clearTimeout(v)), U && ($.taps < 2 && (y = G[0], v = setTimeout(function() {
        v = null;
      }, b)), Do(this), $.start());
    }
  }
  function J(D, ...X) {
    if (this.__zooming) {
      var z = R(this, X).event(D), B = D.changedTouches, $ = B.length, U, ae, O, G;
      for (_u(D), U = 0; U < $; ++U)
        ae = B[U], O = cn(ae, this), z.touch0 && z.touch0[2] === ae.identifier ? z.touch0[0] = O : z.touch1 && z.touch1[2] === ae.identifier && (z.touch1[0] = O);
      if (ae = z.that.__zoom, z.touch1) {
        var ie = z.touch0[0], H = z.touch0[1], K = z.touch1[0], P = z.touch1[1], ue = (ue = K[0] - ie[0]) * ue + (ue = K[1] - ie[1]) * ue, W = (W = P[0] - H[0]) * W + (W = P[1] - H[1]) * W;
        ae = M(ae, Math.sqrt(ue / W)), O = [(ie[0] + K[0]) / 2, (ie[1] + K[1]) / 2], G = [(H[0] + P[0]) / 2, (H[1] + P[1]) / 2];
      } else if (z.touch0) O = z.touch0[0], G = z.touch0[1];
      else return;
      z.zoom("touch", i(S(ae, O, G), z.extent, c));
    }
  }
  function Q(D, ...X) {
    if (this.__zooming) {
      var z = R(this, X).event(D), B = D.changedTouches, $ = B.length, U, ae;
      for (sy(D), m && clearTimeout(m), m = setTimeout(function() {
        m = null;
      }, b), U = 0; U < $; ++U)
        ae = B[U], z.touch0 && z.touch0[2] === ae.identifier ? delete z.touch0 : z.touch1 && z.touch1[2] === ae.identifier && delete z.touch1;
      if (z.touch1 && !z.touch0 && (z.touch0 = z.touch1, delete z.touch1), z.touch0) z.touch0[1] = this.__zoom.invert(z.touch0[0]);
      else if (z.end(), z.taps === 2 && (ae = cn(ae, this), Math.hypot(y[0] - ae[0], y[1] - ae[1]) < w)) {
        var O = Kt(this).on("dblclick.zoom");
        O && O.apply(this, arguments);
      }
    }
  }
  return E.wheelDelta = function(D) {
    return arguments.length ? (u = typeof D == "function" ? D : wo(+D), E) : u;
  }, E.filter = function(D) {
    return arguments.length ? (t = typeof D == "function" ? D : wo(!!D), E) : t;
  }, E.touchable = function(D) {
    return arguments.length ? (l = typeof D == "function" ? D : wo(!!D), E) : l;
  }, E.extent = function(D) {
    return arguments.length ? (r = typeof D == "function" ? D : wo([[+D[0][0], +D[0][1]], [+D[1][0], +D[1][1]]]), E) : r;
  }, E.scaleExtent = function(D) {
    return arguments.length ? (s[0] = +D[0], s[1] = +D[1], E) : [s[0], s[1]];
  }, E.translateExtent = function(D) {
    return arguments.length ? (c[0][0] = +D[0][0], c[1][0] = +D[1][0], c[0][1] = +D[0][1], c[1][1] = +D[1][1], E) : [[c[0][0], c[0][1]], [c[1][0], c[1][1]]];
  }, E.constrain = function(D) {
    return arguments.length ? (i = D, E) : i;
  }, E.duration = function(D) {
    return arguments.length ? (f = +D, E) : f;
  }, E.interpolate = function(D) {
    return arguments.length ? (g = D, E) : g;
  }, E.on = function() {
    var D = h.on.apply(h, arguments);
    return D === h ? E : D;
  }, E.clickDistance = function(D) {
    return arguments.length ? (A = (D = +D) * D, E) : Math.sqrt(A);
  }, E.tapDistance = function(D) {
    return arguments.length ? (w = +D, E) : w;
  }, E;
}
const vn = {
  error001: (t = "react") => `Seems like you have not used ${t === "svelte" ? "SvelteFlowProvider" : "ReactFlowProvider"} as an ancestor. Help: https://${t}flow.dev/error#001`,
  error002: () => "It looks like you've created a new nodeTypes or edgeTypes object. If this wasn't on purpose please define the nodeTypes/edgeTypes outside of the component or memoize them.",
  error003: (t) => `Node type "${t}" not found. Using fallback type "default".`,
  error004: () => "The parent container needs a width and a height to render the graph.",
  error005: () => "Only child nodes can use a parent extent.",
  error006: () => "Can't create edge. An edge needs a source and a target.",
  error007: (t) => `The old edge with id=${t} does not exist.`,
  error009: (t) => `Marker type "${t}" doesn't exist.`,
  error008: (t, { id: r, sourceHandle: i, targetHandle: u }) => `Couldn't create edge for ${t} handle id: "${t === "source" ? i : u}", edge id: ${r}.`,
  error010: () => "Handle: No node id found. Make sure to only use a Handle inside a custom Node.",
  error011: (t) => `Edge type "${t}" not found. Using fallback type "default".`,
  error012: (t) => `Node with id "${t}" does not exist, it may have been removed. This can happen when a node is deleted before the "onNodeClick" handler is called.`,
  error013: (t = "react") => `It seems that you haven't loaded the styles. Please import '@xyflow/${t}/dist/style.css' or base.css to make sure everything is working properly.`,
  error014: () => "useNodeConnections: No node ID found. Call useNodeConnections inside a custom Node or provide a node ID.",
  error015: () => "It seems that you are trying to drag a node that is not initialized. Please use onNodesChange as explained in the docs.",
  error016: (t) => `Edge with id "${t}" does not exist, it may have been removed. This can happen when an edge is deleted before the "onEdgeClick" handler is called.`
}, zu = [
  [Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY],
  [Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY]
], bM = ["Enter", " ", "Escape"], _M = {
  "node.a11yDescription.default": "Press enter or space to select a node. Press delete to remove it and escape to cancel.",
  "node.a11yDescription.keyboardDisabled": "Press enter or space to select a node. You can then use the arrow keys to move the node around. Press delete to remove it and escape to cancel.",
  "node.a11yDescription.ariaLiveMessage": ({ direction: t, x: r, y: i }) => `Moved selected node ${t}. New position, x: ${r}, y: ${i}`,
  "edge.a11yDescription.default": "Press enter or space to select an edge. You can then press delete to remove it or escape to cancel.",
  // Control elements
  "controls.ariaLabel": "Control Panel",
  "controls.zoomIn.ariaLabel": "Zoom In",
  "controls.zoomOut.ariaLabel": "Zoom Out",
  "controls.fitView.ariaLabel": "Fit View",
  "controls.interactive.ariaLabel": "Toggle Interactivity",
  // Mini map
  "minimap.ariaLabel": "Mini Map",
  // Handle
  "handle.ariaLabel": "Handle"
};
var ci;
(function(t) {
  t.Strict = "strict", t.Loose = "loose";
})(ci || (ci = {}));
var ia;
(function(t) {
  t.Free = "free", t.Vertical = "vertical", t.Horizontal = "horizontal";
})(ia || (ia = {}));
var Ou;
(function(t) {
  t.Partial = "partial", t.Full = "full";
})(Ou || (Ou = {}));
const xM = {
  inProgress: !1,
  isValid: null,
  from: null,
  fromHandle: null,
  fromPosition: null,
  fromNode: null,
  to: null,
  toHandle: null,
  toPosition: null,
  toNode: null,
  pointer: null
};
var Or;
(function(t) {
  t.Bezier = "default", t.Straight = "straight", t.Step = "step", t.SmoothStep = "smoothstep", t.SimpleBezier = "simplebezier";
})(Or || (Or = {}));
var ko;
(function(t) {
  t.Arrow = "arrow", t.ArrowClosed = "arrowclosed";
})(ko || (ko = {}));
var _e;
(function(t) {
  t.Left = "left", t.Top = "top", t.Right = "right", t.Bottom = "bottom";
})(_e || (_e = {}));
const Q2 = {
  [_e.Left]: _e.Right,
  [_e.Right]: _e.Left,
  [_e.Top]: _e.Bottom,
  [_e.Bottom]: _e.Top
};
function SM(t) {
  return t === null ? null : t ? "valid" : "invalid";
}
const EM = (t) => !!t && typeof t == "object" && "id" in t && "source" in t && "target" in t, Q5 = (t) => !!t && typeof t == "object" && "id" in t && "position" in t && !("source" in t) && !("target" in t), lm = (t) => !!t && typeof t == "object" && "id" in t && "internals" in t && !("source" in t) && !("target" in t), Iu = (t, r = [0, 0]) => {
  const { width: i, height: u } = Cn(t), l = t.origin ?? r, s = i * l[0], c = u * l[1];
  return {
    x: t.position.x - s,
    y: t.position.y - c
  };
}, $5 = (t, r = { nodeOrigin: [0, 0] }) => {
  if (t.length === 0)
    return { x: 0, y: 0, width: 0, height: 0 };
  const i = t.reduce((u, l) => {
    const s = typeof l == "string";
    let c = !r.nodeLookup && !s ? l : void 0;
    r.nodeLookup && (c = s ? r.nodeLookup.get(l) : lm(l) ? l : r.nodeLookup.get(l.id));
    const f = c ? Xo(c, r.nodeOrigin) : { x: 0, y: 0, x2: 0, y2: 0 };
    return vs(u, f);
  }, { x: 1 / 0, y: 1 / 0, x2: -1 / 0, y2: -1 / 0 });
  return ps(i);
}, Zu = (t, r = {}) => {
  let i = { x: 1 / 0, y: 1 / 0, x2: -1 / 0, y2: -1 / 0 }, u = !1;
  return t.forEach((l) => {
    (r.filter === void 0 || r.filter(l)) && (i = vs(i, Xo(l)), u = !0);
  }), u ? ps(i) : { x: 0, y: 0, width: 0, height: 0 };
}, om = (t, r, [i, u, l] = [0, 0, 1], s = !1, c = !1) => {
  const f = (r.x - i) / l, g = (r.y - u) / l, h = r.width / l, v = r.height / l, y = [];
  for (const m of t.values()) {
    const { measured: b, selectable: _ = !0, hidden: A = !1 } = m;
    if (c && !_ || A)
      continue;
    const w = b.width ?? m.width ?? m.initialWidth ?? 0, E = b.height ?? m.height ?? m.initialHeight ?? 0, { x: M, y: S } = m.internals.positionAbsolute, T = MM(f, g, h, v, M, S, w, E), N = w * E, R = s && T > 0;
    (!m.internals.handleBounds || R || T >= N || m.dragging) && y.push(m);
  }
  return y;
}, F5 = (t, r) => {
  const i = /* @__PURE__ */ new Set();
  return t.forEach((u) => {
    i.add(u.id);
  }), r.filter((u) => i.has(u.source) || i.has(u.target));
};
function J5(t, r) {
  const i = /* @__PURE__ */ new Map(), u = r != null && r.nodes ? new Set(r.nodes.map((l) => l.id)) : null;
  return t.forEach((l) => {
    let s;
    if (r != null && r.includeHiddenNodes) {
      const { width: c, height: f } = Cn(l);
      s = c > 0 && f > 0;
    } else
      s = !!(l.measured.width && l.measured.height && !l.hidden);
    s && (!u || u.has(l.id)) && i.set(l.id, l);
  }), i;
}
async function P5({ nodes: t, width: r, height: i, panZoom: u, minZoom: l, maxZoom: s }, c) {
  if (t.size === 0)
    return !0;
  const f = J5(t, c), g = Zu(f), h = cm(g, r, i, (c == null ? void 0 : c.minZoom) ?? l, (c == null ? void 0 : c.maxZoom) ?? s, (c == null ? void 0 : c.padding) ?? 0.1);
  return await u.setViewport(h, {
    duration: c == null ? void 0 : c.duration,
    ease: c == null ? void 0 : c.ease,
    interpolate: c == null ? void 0 : c.interpolate
  }), !0;
}
function wM({ nodeId: t, nextPosition: r, nodeLookup: i, nodeOrigin: u = [0, 0], nodeExtent: l, onError: s }) {
  const c = i.get(t), f = c.parentId ? i.get(c.parentId) : void 0, { x: g, y: h } = f ? f.internals.positionAbsolute : { x: 0, y: 0 }, v = c.origin ?? u;
  let y = c.extent || l;
  if (c.extent === "parent" && !c.expandParent)
    if (!f)
      s == null || s("005", vn.error005());
    else {
      const b = f.measured.width, _ = f.measured.height;
      b && _ && (y = [
        [g, h],
        [g + b, h + _]
      ]);
    }
  else f && sa(c.extent) && (y = [
    [c.extent[0][0] + g, c.extent[0][1] + h],
    [c.extent[1][0] + g, c.extent[1][1] + h]
  ]);
  const m = sa(y) ? oa(r, y, c.measured) : r;
  return (c.measured.width === void 0 || c.measured.height === void 0) && (s == null || s("015", vn.error015())), {
    position: {
      x: m.x - g + (c.measured.width ?? 0) * v[0],
      y: m.y - h + (c.measured.height ?? 0) * v[1]
    },
    positionAbsolute: m
  };
}
async function W5({ nodesToRemove: t = [], edgesToRemove: r = [], nodes: i, edges: u, onBeforeDelete: l }) {
  const s = new Set(t.map((m) => m.id)), c = [];
  for (const m of i) {
    if (m.deletable === !1)
      continue;
    const b = s.has(m.id), _ = !b && m.parentId && c.find((A) => A.id === m.parentId);
    (b || _) && c.push(m);
  }
  const f = new Set(r.map((m) => m.id)), g = u.filter((m) => m.deletable !== !1), v = F5(c, g);
  for (const m of g)
    f.has(m.id) && !v.find((_) => _.id === m.id) && v.push(m);
  if (!l)
    return {
      edges: v,
      nodes: c
    };
  const y = await l({
    nodes: c,
    edges: v
  });
  return typeof y == "boolean" ? y ? { edges: v, nodes: c } : { edges: [], nodes: [] } : y;
}
const fi = (t, r = 0, i = 1) => Math.min(Math.max(t, r), i), oa = (t = { x: 0, y: 0 }, r, i) => ({
  x: fi(t.x, r[0][0], r[1][0] - ((i == null ? void 0 : i.width) ?? 0)),
  y: fi(t.y, r[0][1], r[1][1] - ((i == null ? void 0 : i.height) ?? 0))
});
function AM(t, r, i) {
  const { width: u, height: l } = Cn(i), { x: s, y: c } = i.internals.positionAbsolute;
  return oa(t, [
    [s, c],
    [s + u, c + l]
  ], r);
}
const $2 = (t, r, i) => t < r ? fi(Math.abs(t - r), 1, r) / r : t > i ? -fi(Math.abs(t - i), 1, r) / r : 0, sm = (t, r, i = 15, u = 40) => {
  const l = $2(t.x, u, r.width - u) * i, s = $2(t.y, u, r.height - u) * i;
  return [l, s];
}, vs = (t, r) => ({
  x: Math.min(t.x, r.x),
  y: Math.min(t.y, r.y),
  x2: Math.max(t.x2, r.x2),
  y2: Math.max(t.y2, r.y2)
}), My = ({ x: t, y: r, width: i, height: u }) => ({
  x: t,
  y: r,
  x2: t + i,
  y2: r + u
}), ps = ({ x: t, y: r, x2: i, y2: u }) => ({
  x: t,
  y: r,
  width: i - t,
  height: u - r
}), Du = (t, r = [0, 0]) => {
  var l, s;
  const { x: i, y: u } = lm(t) ? t.internals.positionAbsolute : Iu(t, r);
  return {
    x: i,
    y: u,
    width: ((l = t.measured) == null ? void 0 : l.width) ?? t.width ?? t.initialWidth ?? 0,
    height: ((s = t.measured) == null ? void 0 : s.height) ?? t.height ?? t.initialHeight ?? 0
  };
}, Xo = (t, r = [0, 0]) => {
  var l, s;
  const { x: i, y: u } = lm(t) ? t.internals.positionAbsolute : Iu(t, r);
  return {
    x: i,
    y: u,
    x2: i + (((l = t.measured) == null ? void 0 : l.width) ?? t.width ?? t.initialWidth ?? 0),
    y2: u + (((s = t.measured) == null ? void 0 : s.height) ?? t.height ?? t.initialHeight ?? 0)
  };
}, TM = (t, r) => ps(vs(My(t), My(r))), MM = (t, r, i, u, l, s, c, f) => {
  const g = Math.max(0, Math.min(t + i, l + c) - Math.max(t, l)), h = Math.max(0, Math.min(r + u, s + f) - Math.max(r, s));
  return Math.ceil(g * h);
}, Io = (t, r) => MM(t.x, t.y, t.width, t.height, r.x, r.y, r.width, r.height), F2 = (t) => dn(t.width) && dn(t.height) && dn(t.x) && dn(t.y), dn = (t) => !isNaN(t) && isFinite(t), CM = (t, r) => (i, u) => {
}, Ku = (t, r = [1, 1]) => ({
  x: r[0] * Math.round(t.x / r[0]),
  y: r[1] * Math.round(t.y / r[1])
}), Qu = ({ x: t, y: r }, [i, u, l], s = !1, c = [1, 1]) => {
  const f = {
    x: (t - i) / l,
    y: (r - u) / l
  };
  return s ? Ku(f, c) : f;
}, di = ({ x: t, y: r }, [i, u, l]) => ({
  x: t * l + i,
  y: r * l + u
});
function ri(t, r) {
  if (typeof t == "number")
    return Math.floor((r - r / (1 + t)) * 0.5);
  if (typeof t == "string" && t.endsWith("px")) {
    const i = parseFloat(t);
    if (!Number.isNaN(i))
      return Math.floor(i);
  }
  if (typeof t == "string" && t.endsWith("%")) {
    const i = parseFloat(t);
    if (!Number.isNaN(i))
      return Math.floor(r * i * 0.01);
  }
  return console.error(`The padding value "${t}" is invalid. Please provide a number or a string with a valid unit (px or %).`), 0;
}
function e6(t, r, i) {
  if (typeof t == "string" || typeof t == "number") {
    const u = ri(t, i), l = ri(t, r);
    return {
      top: u,
      right: l,
      bottom: u,
      left: l,
      x: l * 2,
      y: u * 2
    };
  }
  if (typeof t == "object") {
    const u = ri(t.top ?? t.y ?? 0, i), l = ri(t.bottom ?? t.y ?? 0, i), s = ri(t.left ?? t.x ?? 0, r), c = ri(t.right ?? t.x ?? 0, r);
    return { top: u, right: c, bottom: l, left: s, x: s + c, y: u + l };
  }
  return { top: 0, right: 0, bottom: 0, left: 0, x: 0, y: 0 };
}
function t6(t, r, i, u, l, s) {
  const { x: c, y: f } = di(t, [r, i, u]), { x: g, y: h } = di({ x: t.x + t.width, y: t.y + t.height }, [r, i, u]), v = l - g, y = s - h;
  return {
    left: Math.floor(c),
    top: Math.floor(f),
    right: Math.floor(v),
    bottom: Math.floor(y)
  };
}
const cm = (t, r, i, u, l, s) => {
  const c = e6(s, r, i), f = (r - c.x) / t.width, g = (i - c.y) / t.height, h = Math.min(f, g), v = fi(h, u, l), y = t.x + t.width / 2, m = t.y + t.height / 2, b = r / 2 - y * v, _ = i / 2 - m * v, A = t6(t, b, _, v, r, i), w = {
    left: Math.min(A.left - c.left, 0),
    top: Math.min(A.top - c.top, 0),
    right: Math.min(A.right - c.right, 0),
    bottom: Math.min(A.bottom - c.bottom, 0)
  };
  return {
    x: b - w.left + w.right,
    y: _ - w.top + w.bottom,
    zoom: v
  };
}, Lu = () => {
  var t;
  return typeof navigator < "u" && ((t = navigator == null ? void 0 : navigator.userAgent) == null ? void 0 : t.indexOf("Mac")) >= 0;
};
function sa(t) {
  return t != null && t !== "parent";
}
function Cn(t) {
  var r, i;
  return {
    width: ((r = t.measured) == null ? void 0 : r.width) ?? t.width ?? t.initialWidth ?? 0,
    height: ((i = t.measured) == null ? void 0 : i.height) ?? t.height ?? t.initialHeight ?? 0
  };
}
function qM(t) {
  var r, i;
  return (((r = t.measured) == null ? void 0 : r.width) ?? t.width ?? t.initialWidth) !== void 0 && (((i = t.measured) == null ? void 0 : i.height) ?? t.height ?? t.initialHeight) !== void 0;
}
function RM(t, r = { width: 0, height: 0 }, i, u, l) {
  const s = { ...t }, c = u.get(i);
  if (c) {
    const f = c.origin || l;
    s.x += c.internals.positionAbsolute.x - (r.width ?? 0) * f[0], s.y += c.internals.positionAbsolute.y - (r.height ?? 0) * f[1];
  }
  return s;
}
function J2(t, r) {
  if (t.size !== r.size)
    return !1;
  for (const i of t)
    if (!r.has(i))
      return !1;
  return !0;
}
function n6() {
  let t, r;
  return { promise: new Promise((u, l) => {
    t = u, r = l;
  }), resolve: t, reject: r };
}
function r6(t) {
  return { ..._M, ...t || {} };
}
function Mu(t, { snapGrid: r = [0, 0], snapToGrid: i = !1, transform: u, containerBounds: l }) {
  const { x: s, y: c } = hn(t), f = Qu({ x: s - ((l == null ? void 0 : l.left) ?? 0), y: c - ((l == null ? void 0 : l.top) ?? 0) }, u), { x: g, y: h } = i ? Ku(f, r) : f;
  return {
    xSnapped: g,
    ySnapped: h,
    ...f
  };
}
const fm = (t) => ({
  width: t.offsetWidth,
  height: t.offsetHeight
}), NM = (t) => {
  var r;
  return ((r = t == null ? void 0 : t.getRootNode) == null ? void 0 : r.call(t)) || (window == null ? void 0 : window.document);
}, a6 = ["INPUT", "SELECT", "TEXTAREA"];
function zM(t) {
  var u, l;
  const r = ((l = (u = t.composedPath) == null ? void 0 : u.call(t)) == null ? void 0 : l[0]) || t.target;
  return (r == null ? void 0 : r.nodeType) !== 1 ? !1 : a6.includes(r.nodeName) || r.hasAttribute("contenteditable") || !!r.closest(".nokey");
}
const OM = (t) => "clientX" in t, hn = (t, r) => {
  var s, c;
  const i = OM(t), u = i ? t.clientX : (s = t.touches) == null ? void 0 : s[0].clientX, l = i ? t.clientY : (c = t.touches) == null ? void 0 : c[0].clientY;
  return {
    x: u - ((r == null ? void 0 : r.left) ?? 0),
    y: l - ((r == null ? void 0 : r.top) ?? 0)
  };
}, P2 = (t, r, i, u, l) => {
  const s = r.querySelectorAll(`.${t}`);
  return !s || !s.length ? null : Array.from(s).map((c) => {
    const f = c.getBoundingClientRect();
    return {
      id: c.getAttribute("data-handleid"),
      type: t,
      nodeId: l,
      position: c.getAttribute("data-handlepos"),
      x: (f.left - i.left) / u,
      y: (f.top - i.top) / u,
      ...fm(c)
    };
  });
};
function DM({ sourceX: t, sourceY: r, targetX: i, targetY: u, sourceControlX: l, sourceControlY: s, targetControlX: c, targetControlY: f }) {
  const g = t * 0.125 + l * 0.375 + c * 0.375 + i * 0.125, h = r * 0.125 + s * 0.375 + f * 0.375 + u * 0.125, v = Math.abs(g - t), y = Math.abs(h - r);
  return [g, h, v, y];
}
function Ao(t, r) {
  return t >= 0 ? 0.5 * t : r * 25 * Math.sqrt(-t);
}
function W2({ pos: t, x1: r, y1: i, x2: u, y2: l, c: s }) {
  switch (t) {
    case _e.Left:
      return [r - Ao(r - u, s), i];
    case _e.Right:
      return [r + Ao(u - r, s), i];
    case _e.Top:
      return [r, i - Ao(i - l, s)];
    case _e.Bottom:
      return [r, i + Ao(l - i, s)];
  }
}
function LM({ sourceX: t, sourceY: r, sourcePosition: i = _e.Bottom, targetX: u, targetY: l, targetPosition: s = _e.Top, curvature: c = 0.25 }) {
  const [f, g] = W2({
    pos: i,
    x1: t,
    y1: r,
    x2: u,
    y2: l,
    c
  }), [h, v] = W2({
    pos: s,
    x1: u,
    y1: l,
    x2: t,
    y2: r,
    c
  }), [y, m, b, _] = DM({
    sourceX: t,
    sourceY: r,
    targetX: u,
    targetY: l,
    sourceControlX: f,
    sourceControlY: g,
    targetControlX: h,
    targetControlY: v
  });
  return [
    `M${t},${r} C${f},${g} ${h},${v} ${u},${l}`,
    y,
    m,
    b,
    _
  ];
}
function HM({ sourceX: t, sourceY: r, targetX: i, targetY: u }) {
  const l = Math.abs(i - t) / 2, s = i < t ? i + l : i - l, c = Math.abs(u - r) / 2, f = u < r ? u + c : u - c;
  return [s, f, l, c];
}
function i6({ sourceNode: t, targetNode: r, selected: i = !1, zIndex: u = 0, elevateOnSelect: l = !1, zIndexMode: s = "basic" }) {
  if (s === "manual")
    return u;
  const c = l && i ? u + 1e3 : u, f = Math.max(t.parentId || l && t.selected ? t.internals.z : 0, r.parentId || l && r.selected ? r.internals.z : 0);
  return c + f;
}
function u6({ sourceNode: t, targetNode: r, width: i, height: u, transform: l }) {
  const s = vs(Xo(t), Xo(r));
  s.x === s.x2 && (s.x2 += 1), s.y === s.y2 && (s.y2 += 1);
  const c = {
    x: -l[0] / l[2],
    y: -l[1] / l[2],
    width: i / l[2],
    height: u / l[2]
  };
  return Io(c, ps(s)) > 0;
}
const l6 = ({ source: t, sourceHandle: r, target: i, targetHandle: u }) => `xy-edge__${t}${r || ""}-${i}${u || ""}`, o6 = (t, r) => r.some((i) => i.source === t.source && i.target === t.target && (i.sourceHandle === t.sourceHandle || !i.sourceHandle && !t.sourceHandle) && (i.targetHandle === t.targetHandle || !i.targetHandle && !t.targetHandle)), s6 = (t, r, i = {}) => {
  var s;
  if (!t.source || !t.target)
    return (s = i.onError) == null || s.call(i, "006", vn.error006()), r;
  const u = i.getEdgeId || l6;
  let l;
  return EM(t) ? l = { ...t } : l = {
    ...t,
    id: u(t)
  }, o6(l, r) ? r : (l.sourceHandle === null && delete l.sourceHandle, l.targetHandle === null && delete l.targetHandle, r.concat(l));
};
function jM({ sourceX: t, sourceY: r, targetX: i, targetY: u }) {
  const [l, s, c, f] = HM({
    sourceX: t,
    sourceY: r,
    targetX: i,
    targetY: u
  });
  return [`M ${t},${r}L ${i},${u}`, l, s, c, f];
}
const eA = {
  [_e.Left]: { x: -1, y: 0 },
  [_e.Right]: { x: 1, y: 0 },
  [_e.Top]: { x: 0, y: -1 },
  [_e.Bottom]: { x: 0, y: 1 }
}, c6 = ({ source: t, sourcePosition: r = _e.Bottom, target: i }) => r === _e.Left || r === _e.Right ? t.x < i.x ? { x: 1, y: 0 } : { x: -1, y: 0 } : t.y < i.y ? { x: 0, y: 1 } : { x: 0, y: -1 }, tA = (t, r) => Math.sqrt(Math.pow(r.x - t.x, 2) + Math.pow(r.y - t.y, 2));
function f6({ source: t, sourcePosition: r = _e.Bottom, target: i, targetPosition: u = _e.Top, center: l, offset: s, stepPosition: c }) {
  const f = eA[r], g = eA[u], h = { x: t.x + f.x * s, y: t.y + f.y * s }, v = { x: i.x + g.x * s, y: i.y + g.y * s }, y = c6({
    source: h,
    sourcePosition: r,
    target: v
  }), m = y.x !== 0 ? "x" : "y", b = y[m];
  let _ = [], A, w;
  const E = { x: 0, y: 0 }, M = { x: 0, y: 0 }, [, , S, T] = HM({
    sourceX: t.x,
    sourceY: t.y,
    targetX: i.x,
    targetY: i.y
  });
  if (f[m] * g[m] === -1) {
    m === "x" ? (A = l.x ?? h.x + (v.x - h.x) * c, w = l.y ?? (h.y + v.y) / 2) : (A = l.x ?? (h.x + v.x) / 2, w = l.y ?? h.y + (v.y - h.y) * c);
    const L = [
      { x: A, y: h.y },
      { x: A, y: v.y }
    ], j = [
      { x: h.x, y: w },
      { x: v.x, y: w }
    ];
    f[m] === b ? _ = m === "x" ? L : j : _ = m === "x" ? j : L;
  } else {
    const L = [{ x: h.x, y: v.y }], j = [{ x: v.x, y: h.y }];
    if (m === "x" ? _ = f.x === b ? j : L : _ = f.y === b ? L : j, r === u) {
      const D = Math.abs(t[m] - i[m]);
      if (D <= s) {
        const X = Math.min(s - 1, s - D);
        f[m] === b ? E[m] = (h[m] > t[m] ? -1 : 1) * X : M[m] = (v[m] > i[m] ? -1 : 1) * X;
      }
    }
    if (r !== u) {
      const D = m === "x" ? "y" : "x", X = f[m] === g[D], z = h[D] > v[D], B = h[D] < v[D];
      (f[m] === 1 && (!X && z || X && B) || f[m] !== 1 && (!X && B || X && z)) && (_ = m === "x" ? L : j);
    }
    const Z = { x: h.x + E.x, y: h.y + E.y }, Y = { x: v.x + M.x, y: v.y + M.y }, J = Math.max(Math.abs(Z.x - _[0].x), Math.abs(Y.x - _[0].x)), Q = Math.max(Math.abs(Z.y - _[0].y), Math.abs(Y.y - _[0].y));
    J >= Q ? (A = (Z.x + Y.x) / 2, w = _[0].y) : (A = _[0].x, w = (Z.y + Y.y) / 2);
  }
  const N = { x: h.x + E.x, y: h.y + E.y }, R = { x: v.x + M.x, y: v.y + M.y };
  return [[
    t,
    // we only want to add the gapped source/target if they are different from the first/last point to avoid duplicates which can cause issues with the bends
    ...N.x !== _[0].x || N.y !== _[0].y ? [N] : [],
    ..._,
    ...R.x !== _[_.length - 1].x || R.y !== _[_.length - 1].y ? [R] : [],
    i
  ], A, w, S, T];
}
function d6(t, r, i, u) {
  const l = Math.min(tA(t, r) / 2, tA(r, i) / 2, u), { x: s, y: c } = r;
  if (t.x === s && s === i.x || t.y === c && c === i.y)
    return `L${s} ${c}`;
  if (t.y === c) {
    const h = t.x < i.x ? -1 : 1, v = t.y < i.y ? 1 : -1;
    return `L ${s + l * h},${c}Q ${s},${c} ${s},${c + l * v}`;
  }
  const f = t.x < i.x ? 1 : -1, g = t.y < i.y ? -1 : 1;
  return `L ${s},${c + l * g}Q ${s},${c} ${s + l * f},${c}`;
}
function Cy({ sourceX: t, sourceY: r, sourcePosition: i = _e.Bottom, targetX: u, targetY: l, targetPosition: s = _e.Top, borderRadius: c = 5, centerX: f, centerY: g, offset: h = 20, stepPosition: v = 0.5 }) {
  const [y, m, b, _, A] = f6({
    source: { x: t, y: r },
    sourcePosition: i,
    target: { x: u, y: l },
    targetPosition: s,
    center: { x: f, y: g },
    offset: h,
    stepPosition: v
  });
  let w = `M${y[0].x} ${y[0].y}`;
  for (let E = 1; E < y.length - 1; E++)
    w += d6(y[E - 1], y[E], y[E + 1], c);
  return w += `L${y[y.length - 1].x} ${y[y.length - 1].y}`, [w, m, b, _, A];
}
function nA(t) {
  var r;
  return t && !!(t.internals.handleBounds || (r = t.handles) != null && r.length) && !!(t.measured.width || t.width || t.initialWidth);
}
function h6(t) {
  var y;
  const { sourceNode: r, targetNode: i } = t;
  if (!nA(r) || !nA(i))
    return null;
  const u = r.internals.handleBounds || rA(r.handles), l = i.internals.handleBounds || rA(i.handles), s = aA((u == null ? void 0 : u.source) ?? [], t.sourceHandle), c = aA(
    // when connection type is loose we can define all handles as sources and connect source -> source
    t.connectionMode === ci.Strict ? (l == null ? void 0 : l.target) ?? [] : ((l == null ? void 0 : l.target) ?? []).concat((l == null ? void 0 : l.source) ?? []),
    t.targetHandle
  );
  if (!s || !c)
    return (y = t.onError) == null || y.call(t, "008", vn.error008(s ? "target" : "source", {
      id: t.id,
      sourceHandle: t.sourceHandle,
      targetHandle: t.targetHandle
    })), null;
  const f = (s == null ? void 0 : s.position) || _e.Bottom, g = (c == null ? void 0 : c.position) || _e.Top, h = ca(r, s, f), v = ca(i, c, g);
  return {
    sourceX: h.x,
    sourceY: h.y,
    targetX: v.x,
    targetY: v.y,
    sourcePosition: f,
    targetPosition: g
  };
}
function rA(t) {
  if (!t)
    return null;
  const r = [], i = [];
  for (const u of t)
    u.width = u.width ?? 1, u.height = u.height ?? 1, u.type === "source" ? r.push(u) : u.type === "target" && i.push(u);
  return {
    source: r,
    target: i
  };
}
function ca(t, r, i = _e.Left, u = !1) {
  const l = ((r == null ? void 0 : r.x) ?? 0) + t.internals.positionAbsolute.x, s = ((r == null ? void 0 : r.y) ?? 0) + t.internals.positionAbsolute.y, { width: c, height: f } = r ?? Cn(t);
  if (u)
    return { x: l + c / 2, y: s + f / 2 };
  switch ((r == null ? void 0 : r.position) ?? i) {
    case _e.Top:
      return { x: l + c / 2, y: s };
    case _e.Right:
      return { x: l + c, y: s + f / 2 };
    case _e.Bottom:
      return { x: l + c / 2, y: s + f };
    case _e.Left:
      return { x: l, y: s + f / 2 };
  }
}
function aA(t, r) {
  return t && (r ? t.find((i) => i.id === r) : t[0]) || null;
}
function qy(t, r) {
  return t ? typeof t == "string" ? t : `${r ? `${r}__` : ""}${Object.keys(t).sort().map((u) => `${u}=${t[u]}`).join("&")}` : "";
}
function g6(t, { id: r, defaultColor: i, defaultMarkerStart: u, defaultMarkerEnd: l }) {
  const s = /* @__PURE__ */ new Set();
  return t.reduce((c, f) => ([f.markerStart || u, f.markerEnd || l].forEach((g) => {
    if (g && typeof g == "object") {
      const h = qy(g, r);
      s.has(h) || (c.push({ id: h, color: g.color || i, ...g }), s.add(h));
    }
  }), c), []).sort((c, f) => c.id.localeCompare(f.id));
}
const BM = 1e3, v6 = 10, dm = {
  nodeOrigin: [0, 0],
  nodeExtent: zu,
  elevateNodesOnSelect: !0,
  zIndexMode: "basic",
  defaults: {}
}, p6 = {
  ...dm,
  checkEquality: !0
};
function hm(t, r) {
  const i = { ...t };
  for (const u in r)
    r[u] !== void 0 && (i[u] = r[u]);
  return i;
}
function y6(t, r, i) {
  const u = hm(dm, i);
  for (const l of t.values())
    if (l.parentId)
      vm(l, t, r, u);
    else {
      const s = Iu(l, u.nodeOrigin), c = sa(l.extent) ? l.extent : u.nodeExtent, f = oa(s, c, Cn(l));
      l.internals.positionAbsolute = f;
    }
}
function m6(t, r) {
  if (!t.handles)
    return t.measured ? r == null ? void 0 : r.internals.handleBounds : void 0;
  const i = [], u = [];
  for (const l of t.handles) {
    const s = {
      id: l.id,
      width: l.width ?? 1,
      height: l.height ?? 1,
      nodeId: t.id,
      x: l.x,
      y: l.y,
      position: l.position,
      type: l.type
    };
    l.type === "source" ? i.push(s) : l.type === "target" && u.push(s);
  }
  return {
    source: i,
    target: u
  };
}
function gm(t) {
  return t === "manual";
}
function Ry(t, r, i, u = {}) {
  var v, y;
  const l = hm(p6, u), s = { i: 0 }, c = new Map(r), f = l != null && l.elevateNodesOnSelect && !gm(l.zIndexMode) ? BM : 0;
  let g = t.length > 0, h = !1;
  r.clear(), i.clear();
  for (const m of t) {
    let b = c.get(m.id);
    if (l.checkEquality && m === (b == null ? void 0 : b.internals.userNode))
      r.set(m.id, b);
    else {
      const _ = Iu(m, l.nodeOrigin), A = sa(m.extent) ? m.extent : l.nodeExtent, w = oa(_, A, Cn(m));
      b = {
        ...l.defaults,
        ...m,
        measured: {
          width: (v = m.measured) == null ? void 0 : v.width,
          height: (y = m.measured) == null ? void 0 : y.height
        },
        internals: {
          positionAbsolute: w,
          // if user re-initializes the node or removes `measured` for whatever reason, we reset the handleBounds so that the node gets re-measured
          handleBounds: m6(m, b),
          z: UM(m, f, l.zIndexMode),
          userNode: m
        }
      }, r.set(m.id, b);
    }
    (b.measured === void 0 || b.measured.width === void 0 || b.measured.height === void 0) && !b.hidden && (g = !1), m.parentId && vm(b, r, i, u, s), h || (h = m.selected ?? !1);
  }
  return { nodesInitialized: g, hasSelectedNodes: h };
}
function b6(t, r) {
  if (!t.parentId)
    return;
  const i = r.get(t.parentId);
  i ? i.set(t.id, t) : r.set(t.parentId, /* @__PURE__ */ new Map([[t.id, t]]));
}
function vm(t, r, i, u, l) {
  const { elevateNodesOnSelect: s, nodeOrigin: c, nodeExtent: f, zIndexMode: g } = hm(dm, u), h = t.parentId, v = r.get(h);
  if (!v) {
    console.warn(`Parent node ${h} not found. Please make sure that parent nodes are in front of their child nodes in the nodes array.`);
    return;
  }
  b6(t, i), l && !v.parentId && v.internals.rootParentIndex === void 0 && g === "auto" && (v.internals.rootParentIndex = ++l.i, v.internals.z = v.internals.z + l.i * v6), l && v.internals.rootParentIndex !== void 0 && (l.i = v.internals.rootParentIndex);
  const y = s && !gm(g) ? BM : 0, { x: m, y: b, z: _ } = _6(t, v, c, f, y, g), { positionAbsolute: A } = t.internals, w = m !== A.x || b !== A.y;
  (w || _ !== t.internals.z) && r.set(t.id, {
    ...t,
    internals: {
      ...t.internals,
      positionAbsolute: w ? { x: m, y: b } : A,
      z: _
    }
  });
}
function UM(t, r, i) {
  const u = dn(t.zIndex) ? t.zIndex : 0;
  return gm(i) ? u : u + (t.selected ? r : 0);
}
function _6(t, r, i, u, l, s) {
  const { x: c, y: f } = r.internals.positionAbsolute, g = Cn(t), h = Iu(t, i), v = sa(t.extent) ? oa(h, t.extent, g) : h;
  let y = oa({ x: c + v.x, y: f + v.y }, u, g);
  t.extent === "parent" && (y = AM(y, g, r));
  const m = UM(t, l, s), b = r.internals.z ?? 0;
  return {
    x: y.x,
    y: y.y,
    z: b >= m ? b + 1 : m
  };
}
function pm(t, r, i, u = [0, 0]) {
  var c;
  const l = [], s = /* @__PURE__ */ new Map();
  for (const f of t) {
    const g = r.get(f.parentId);
    if (!g)
      continue;
    const h = ((c = s.get(f.parentId)) == null ? void 0 : c.expandedRect) ?? Du(g), v = TM(h, f.rect);
    s.set(f.parentId, { expandedRect: v, parent: g });
  }
  return s.size > 0 && s.forEach(({ expandedRect: f, parent: g }, h) => {
    var S;
    const v = g.internals.positionAbsolute, y = Cn(g), m = g.origin ?? u, b = f.x < v.x ? Math.round(Math.abs(v.x - f.x)) : 0, _ = f.y < v.y ? Math.round(Math.abs(v.y - f.y)) : 0, A = Math.max(y.width, Math.round(f.width)), w = Math.max(y.height, Math.round(f.height)), E = (A - y.width) * m[0], M = (w - y.height) * m[1];
    (b > 0 || _ > 0 || E || M) && (l.push({
      id: h,
      type: "position",
      position: {
        x: g.position.x - b + E,
        y: g.position.y - _ + M
      }
    }), (S = i.get(h)) == null || S.forEach((T) => {
      t.some((N) => N.id === T.id) || l.push({
        id: T.id,
        type: "position",
        position: {
          x: T.position.x + b,
          y: T.position.y + _
        }
      });
    })), (y.width < f.width || y.height < f.height || b || _) && l.push({
      id: h,
      type: "dimensions",
      setAttributes: !0,
      dimensions: {
        width: A + (b ? m[0] * b - E : 0),
        height: w + (_ ? m[1] * _ - M : 0)
      }
    });
  }), l;
}
function x6(t, r, i, u, l, s, c) {
  const f = u == null ? void 0 : u.querySelector(".xyflow__viewport");
  let g = !1;
  if (!f)
    return { changes: [], updatedInternals: g };
  const h = [], v = window.getComputedStyle(f), { m22: y } = new window.DOMMatrixReadOnly(v.transform), m = [];
  for (const b of t.values()) {
    const _ = r.get(b.id);
    if (!_)
      continue;
    if (_.hidden) {
      r.set(_.id, {
        ..._,
        internals: {
          ..._.internals,
          handleBounds: void 0
        }
      }), g = !0;
      continue;
    }
    const A = fm(b.nodeElement), w = _.measured.width !== A.width || _.measured.height !== A.height;
    if (!!(A.width && A.height && (w || !_.internals.handleBounds || b.force))) {
      const M = b.nodeElement.getBoundingClientRect(), S = sa(_.extent) ? _.extent : s;
      let { positionAbsolute: T } = _.internals;
      if (_.parentId && _.extent === "parent") {
        const R = r.get(_.parentId);
        R && (T = AM(T, A, R));
      } else S && (T = oa(T, S, A));
      const N = {
        ..._,
        measured: A,
        internals: {
          ..._.internals,
          positionAbsolute: T,
          handleBounds: {
            source: P2("source", b.nodeElement, M, y, _.id),
            target: P2("target", b.nodeElement, M, y, _.id)
          }
        }
      };
      r.set(_.id, N), _.parentId && vm(N, r, i, { nodeOrigin: l, zIndexMode: c }), g = !0, w && (h.push({
        id: _.id,
        type: "dimensions",
        dimensions: A
      }), _.expandParent && _.parentId && m.push({
        id: _.id,
        parentId: _.parentId,
        rect: Du(N, l)
      }));
    }
  }
  if (m.length > 0) {
    const b = pm(m, r, i, l);
    h.push(...b);
  }
  return { changes: h, updatedInternals: g };
}
async function S6({ delta: t, panZoom: r, transform: i, translateExtent: u, width: l, height: s }) {
  if (!r || !t.x && !t.y)
    return !1;
  const c = await r.setViewportConstrained({
    x: i[0] + t.x,
    y: i[1] + t.y,
    zoom: i[2]
  }, [
    [0, 0],
    [l, s]
  ], u);
  return !!c && (c.x !== i[0] || c.y !== i[1] || c.k !== i[2]);
}
function iA(t, r, i, u, l, s) {
  let c = l;
  const f = u.get(c) || /* @__PURE__ */ new Map();
  u.set(c, f.set(i, r)), c = `${l}-${t}`;
  const g = u.get(c) || /* @__PURE__ */ new Map();
  if (u.set(c, g.set(i, r)), s) {
    c = `${l}-${t}-${s}`;
    const h = u.get(c) || /* @__PURE__ */ new Map();
    u.set(c, h.set(i, r));
  }
}
function GM(t, r, i) {
  t.clear(), r.clear();
  for (const u of i) {
    const { source: l, target: s, sourceHandle: c = null, targetHandle: f = null } = u, g = { edgeId: u.id, source: l, target: s, sourceHandle: c, targetHandle: f }, h = `${l}-${c}--${s}-${f}`, v = `${s}-${f}--${l}-${c}`;
    iA("source", g, v, t, l, c), iA("target", g, h, t, s, f), r.set(u.id, u);
  }
}
function VM(t, r) {
  if (!t.parentId)
    return !1;
  const i = r.get(t.parentId);
  return i ? i.selected ? !0 : VM(i, r) : !1;
}
function uA(t, r, i) {
  var l;
  let u = t;
  do {
    if ((l = u == null ? void 0 : u.matches) != null && l.call(u, r))
      return !0;
    if (u === i)
      return !1;
    u = u == null ? void 0 : u.parentElement;
  } while (u);
  return !1;
}
function E6(t, r, i, u) {
  const l = /* @__PURE__ */ new Map();
  for (const [s, c] of t)
    if ((c.selected || c.id === u) && (!c.parentId || !VM(c, t)) && (c.draggable || r && typeof c.draggable > "u")) {
      const f = t.get(s);
      f && l.set(s, {
        id: s,
        position: f.position || { x: 0, y: 0 },
        distance: {
          x: i.x - f.internals.positionAbsolute.x,
          y: i.y - f.internals.positionAbsolute.y
        },
        extent: f.extent,
        parentId: f.parentId,
        origin: f.origin,
        expandParent: f.expandParent,
        internals: {
          positionAbsolute: f.internals.positionAbsolute || { x: 0, y: 0 }
        },
        measured: {
          width: f.measured.width ?? 0,
          height: f.measured.height ?? 0
        }
      });
    }
  return l;
}
function cy({ nodeId: t, dragItems: r, nodeLookup: i, dragging: u = !0 }) {
  var c, f, g;
  const l = [];
  for (const [h, v] of r) {
    const y = (c = i.get(h)) == null ? void 0 : c.internals.userNode;
    y && l.push({
      ...y,
      position: v.position,
      dragging: u
    });
  }
  if (!t)
    return [l[0], l];
  const s = (f = i.get(t)) == null ? void 0 : f.internals.userNode;
  return [
    s ? {
      ...s,
      position: ((g = r.get(t)) == null ? void 0 : g.position) || s.position,
      dragging: u
    } : l[0],
    l
  ];
}
function w6({ dragItems: t, snapGrid: r, x: i, y: u }) {
  const l = t.values().next().value;
  if (!l)
    return null;
  const s = {
    x: i - l.distance.x,
    y: u - l.distance.y
  }, c = Ku(s, r);
  return {
    x: c.x - s.x,
    y: c.y - s.y
  };
}
function A6({ onNodeMouseDown: t, getStoreItems: r, onDragStart: i, onDrag: u, onDragStop: l }) {
  let s = { x: null, y: null }, c = 0, f = /* @__PURE__ */ new Map(), g = !1, h = { x: 0, y: 0 }, v = null, y = !1, m = null, b = !1, _ = !1, A = null;
  function w({ noDragClassName: M, handleSelector: S, domNode: T, isSelectable: N, nodeId: R, nodeClickDistance: q = 0 }) {
    m = Kt(T);
    function L({ x: J, y: Q }) {
      const { nodeLookup: D, nodeExtent: X, snapGrid: z, snapToGrid: B, nodeOrigin: $, onNodeDrag: U, onSelectionDrag: ae, onError: O, updateNodePositions: G } = r();
      s = { x: J, y: Q };
      let ie = !1;
      const H = f.size > 1, K = H && X ? My(Zu(f)) : null, P = H && B ? w6({
        dragItems: f,
        snapGrid: z,
        x: J,
        y: Q
      }) : null;
      for (const [ue, W] of f) {
        if (!D.has(ue))
          continue;
        let se = { x: J - W.distance.x, y: Q - W.distance.y };
        B && (se = P ? {
          x: Math.round(se.x + P.x),
          y: Math.round(se.y + P.y)
        } : Ku(se, z));
        let de = null;
        if (H && X && !W.extent && K) {
          const { positionAbsolute: he } = W.internals, me = he.x - K.x + X[0][0], ge = he.x + W.measured.width - K.x2 + X[1][0], Ae = he.y - K.y + X[0][1], xe = he.y + W.measured.height - K.y2 + X[1][1];
          de = [
            [me, Ae],
            [ge, xe]
          ];
        }
        const { position: ve, positionAbsolute: ye } = wM({
          nodeId: ue,
          nextPosition: se,
          nodeLookup: D,
          nodeExtent: de || X,
          nodeOrigin: $,
          onError: O
        });
        ie = ie || W.position.x !== ve.x || W.position.y !== ve.y, W.position = ve, W.internals.positionAbsolute = ye;
      }
      if (_ = _ || ie, !!ie && (G(f, !0), A && (u || U || !R && ae))) {
        const [ue, W] = cy({
          nodeId: R,
          dragItems: f,
          nodeLookup: D
        });
        u == null || u(A, f, ue, W), U == null || U(A, ue, W), R || ae == null || ae(A, W);
      }
    }
    async function j() {
      if (!v)
        return;
      const { transform: J, panBy: Q, autoPanSpeed: D, autoPanOnNodeDrag: X } = r();
      if (!X) {
        g = !1, cancelAnimationFrame(c);
        return;
      }
      const [z, B] = sm(h, v, D);
      (z !== 0 || B !== 0) && (s.x = (s.x ?? 0) - z / J[2], s.y = (s.y ?? 0) - B / J[2], await Q({ x: z, y: B }) && L(s)), c = requestAnimationFrame(j);
    }
    function Z(J) {
      var H;
      const { nodeLookup: Q, multiSelectionActive: D, nodesDraggable: X, transform: z, snapGrid: B, snapToGrid: $, selectNodesOnDrag: U, onNodeDragStart: ae, onSelectionDragStart: O, unselectNodesAndEdges: G } = r();
      y = !0, (!U || !N) && !D && R && ((H = Q.get(R)) != null && H.selected || G()), N && U && R && (t == null || t(R));
      const ie = Mu(J.sourceEvent, { transform: z, snapGrid: B, snapToGrid: $, containerBounds: v });
      if (s = ie, f = E6(Q, X, ie, R), f.size > 0 && (i || ae || !R && O)) {
        const [K, P] = cy({
          nodeId: R,
          dragItems: f,
          nodeLookup: Q
        });
        i == null || i(J.sourceEvent, f, K, P), ae == null || ae(J.sourceEvent, K, P), R || O == null || O(J.sourceEvent, P);
      }
    }
    const Y = rM().clickDistance(q).on("start", (J) => {
      const { domNode: Q, nodeDragThreshold: D, transform: X, snapGrid: z, snapToGrid: B } = r();
      v = (Q == null ? void 0 : Q.getBoundingClientRect()) || null, b = !1, _ = !1, A = J.sourceEvent, D === 0 && Z(J), s = Mu(J.sourceEvent, { transform: X, snapGrid: z, snapToGrid: B, containerBounds: v }), h = hn(J.sourceEvent, v);
    }).on("drag", (J) => {
      const { autoPanOnNodeDrag: Q, transform: D, snapGrid: X, snapToGrid: z, nodeDragThreshold: B, nodeLookup: $ } = r(), U = Mu(J.sourceEvent, { transform: D, snapGrid: X, snapToGrid: z, containerBounds: v });
      if (A = J.sourceEvent, (J.sourceEvent.type === "touchmove" && J.sourceEvent.touches.length > 1 || // if user deletes a node while dragging, we need to abort the drag to prevent errors
      R && !$.has(R)) && (b = !0), !b) {
        if (!g && Q && y && (g = !0, j()), !y) {
          const ae = hn(J.sourceEvent, v), O = ae.x - h.x, G = ae.y - h.y;
          Math.sqrt(O * O + G * G) > B && Z(J);
        }
        (s.x !== U.xSnapped || s.y !== U.ySnapped) && f && y && (h = hn(J.sourceEvent, v), L(U));
      }
    }).on("end", (J) => {
      if (!y || b) {
        b && f.size > 0 && r().updateNodePositions(f, !1);
        return;
      }
      if (g = !1, y = !1, cancelAnimationFrame(c), f.size > 0) {
        const { nodeLookup: Q, updateNodePositions: D, onNodeDragStop: X, onSelectionDragStop: z } = r();
        if (_ && (D(f, !1), _ = !1), l || X || !R && z) {
          const [B, $] = cy({
            nodeId: R,
            dragItems: f,
            nodeLookup: Q,
            dragging: !1
          });
          l == null || l(J.sourceEvent, f, B, $), X == null || X(J.sourceEvent, B, $), R || z == null || z(J.sourceEvent, $);
        }
      }
    }).filter((J) => {
      const Q = J.target;
      return !J.button && (!M || !uA(Q, `.${M}`, T)) && (!S || uA(Q, S, T));
    });
    m.call(Y);
  }
  function E() {
    m == null || m.on(".drag", null);
  }
  return {
    update: w,
    destroy: E
  };
}
function T6(t, r, i) {
  const u = [], l = {
    x: t.x - i,
    y: t.y - i,
    width: i * 2,
    height: i * 2
  };
  for (const s of r.values())
    Io(l, Du(s)) > 0 && u.push(s);
  return u;
}
const M6 = 250;
function C6(t, r, i, u) {
  var f, g;
  let l = [], s = 1 / 0;
  const c = T6(t, i, r + M6);
  for (const h of c) {
    const v = [...((f = h.internals.handleBounds) == null ? void 0 : f.source) ?? [], ...((g = h.internals.handleBounds) == null ? void 0 : g.target) ?? []];
    for (const y of v) {
      if (u.nodeId === y.nodeId && u.type === y.type && u.id === y.id)
        continue;
      const { x: m, y: b } = ca(h, y, y.position, !0), _ = Math.sqrt(Math.pow(m - t.x, 2) + Math.pow(b - t.y, 2));
      _ > r || (_ < s ? (l = [{ ...y, x: m, y: b }], s = _) : _ === s && l.push({ ...y, x: m, y: b }));
    }
  }
  if (!l.length)
    return null;
  if (l.length > 1) {
    const h = u.type === "source" ? "target" : "source";
    return l.find((v) => v.type === h) ?? l[0];
  }
  return l[0];
}
function YM(t, r, i, u, l, s = !1) {
  var h, v, y;
  const c = u.get(t);
  if (!c)
    return null;
  const f = l === "strict" ? (h = c.internals.handleBounds) == null ? void 0 : h[r] : [...((v = c.internals.handleBounds) == null ? void 0 : v.source) ?? [], ...((y = c.internals.handleBounds) == null ? void 0 : y.target) ?? []], g = (i ? f == null ? void 0 : f.find((m) => m.id === i) : f == null ? void 0 : f[0]) ?? null;
  return g && s ? { ...g, ...ca(c, g, g.position, !0) } : g;
}
function kM(t, r) {
  return t || (r != null && r.classList.contains("target") ? "target" : r != null && r.classList.contains("source") ? "source" : null);
}
function q6(t, r) {
  let i = null;
  return r ? i = !0 : t && !r && (i = !1), i;
}
const XM = () => !0;
function R6(t, { connectionMode: r, connectionRadius: i, handleId: u, nodeId: l, edgeUpdaterType: s, isTarget: c, domNode: f, nodeLookup: g, lib: h, autoPanOnConnect: v, flowId: y, panBy: m, cancelConnection: b, onConnectStart: _, onConnect: A, onConnectEnd: w, isValidConnection: E = XM, onReconnectEnd: M, updateConnection: S, getTransform: T, getFromHandle: N, autoPanSpeed: R, dragThreshold: q = 1, handleDomNode: L }) {
  const j = NM(t.target);
  let Z = 0, Y;
  const { x: J, y: Q } = hn(t), D = kM(s, L), X = f == null ? void 0 : f.getBoundingClientRect();
  let z = !1;
  if (!X || !D)
    return;
  const B = YM(l, D, u, g, r);
  if (!B)
    return;
  let $ = hn(t, X), U = !1, ae = null, O = !1, G = null;
  function ie() {
    if (!v || !X)
      return;
    const [ve, ye] = sm($, X, R);
    m({ x: ve, y: ye }), Z = requestAnimationFrame(ie);
  }
  const H = {
    ...B,
    nodeId: l,
    type: D,
    position: B.position
  }, K = g.get(l);
  let ue = {
    inProgress: !0,
    isValid: null,
    from: ca(K, H, _e.Left, !0),
    fromHandle: H,
    fromPosition: H.position,
    fromNode: K,
    to: $,
    toHandle: null,
    toPosition: Q2[H.position],
    toNode: null,
    pointer: $
  };
  function W() {
    z = !0, S(ue), _ == null || _(t, { nodeId: l, handleId: u, handleType: D });
  }
  q === 0 && W();
  function se(ve) {
    if (!z) {
      const { x: xe, y: Pe } = hn(ve), tt = xe - J, xt = Pe - Q;
      if (!(tt * tt + xt * xt > q * q))
        return;
      W();
    }
    if (!N() || !H) {
      de(ve);
      return;
    }
    const ye = T();
    $ = hn(ve, X), Y = C6(Qu($, ye, !1, [1, 1]), i, g, H), U || (ie(), U = !0);
    const he = IM(ve, {
      handle: Y,
      connectionMode: r,
      fromNodeId: l,
      fromHandleId: u,
      fromType: c ? "target" : "source",
      isValidConnection: E,
      doc: j,
      lib: h,
      flowId: y,
      nodeLookup: g
    });
    G = he.handleDomNode, ae = he.connection, O = q6(!!Y, he.isValid);
    const me = g.get(l), ge = me ? ca(me, H, _e.Left, !0) : ue.from, Ae = {
      ...ue,
      from: ge,
      isValid: O,
      to: he.toHandle && O ? di({ x: he.toHandle.x, y: he.toHandle.y }, ye) : $,
      toHandle: he.toHandle,
      toPosition: O && he.toHandle ? he.toHandle.position : Q2[H.position],
      toNode: he.toHandle ? g.get(he.toHandle.nodeId) : null,
      pointer: $
    };
    S(Ae), ue = Ae;
  }
  function de(ve) {
    if (!("touches" in ve && ve.touches.length > 0)) {
      if (z) {
        (Y || G) && ae && O && (A == null || A(ae));
        const { inProgress: ye, ...he } = ue, me = {
          ...he,
          toPosition: ue.toHandle ? ue.toPosition : null
        };
        w == null || w(ve, me), s && (M == null || M(ve, me));
      }
      b(), cancelAnimationFrame(Z), U = !1, O = !1, ae = null, G = null, j.removeEventListener("mousemove", se), j.removeEventListener("mouseup", de), j.removeEventListener("touchmove", se), j.removeEventListener("touchend", de);
    }
  }
  j.addEventListener("mousemove", se), j.addEventListener("mouseup", de), j.addEventListener("touchmove", se), j.addEventListener("touchend", de);
}
function IM(t, { handle: r, connectionMode: i, fromNodeId: u, fromHandleId: l, fromType: s, doc: c, lib: f, flowId: g, isValidConnection: h = XM, nodeLookup: v }) {
  const y = s === "target", m = r ? c.querySelector(`.${f}-flow__handle[data-id="${g}-${r == null ? void 0 : r.nodeId}-${r == null ? void 0 : r.id}-${r == null ? void 0 : r.type}"]`) : null, { x: b, y: _ } = hn(t), A = c.elementFromPoint(b, _), w = A != null && A.classList.contains(`${f}-flow__handle`) ? A : m, E = {
    handleDomNode: w,
    isValid: !1,
    connection: null,
    toHandle: null
  };
  if (w) {
    const M = kM(void 0, w), S = w.getAttribute("data-nodeid"), T = w.getAttribute("data-handleid"), N = w.classList.contains("connectable"), R = w.classList.contains("connectableend");
    if (!S || !M)
      return E;
    const q = {
      source: y ? S : u,
      sourceHandle: y ? T : l,
      target: y ? u : S,
      targetHandle: y ? l : T
    };
    E.connection = q;
    const j = N && R && (i === ci.Strict ? y && M === "source" || !y && M === "target" : S !== u || T !== l);
    E.isValid = j && h(q), E.toHandle = YM(S, M, T, v, i, !0);
  }
  return E;
}
const Ny = {
  onPointerDown: R6,
  isValid: IM
};
function N6({ domNode: t, panZoom: r, getTransform: i, getViewScale: u }) {
  const l = Kt(t);
  function s({ translateExtent: f, width: g, height: h, zoomStep: v = 1, pannable: y = !0, zoomable: m = !0, inversePan: b = !1 }) {
    const _ = (S) => {
      if (S.sourceEvent.type !== "wheel" || !r)
        return;
      const T = i(), N = S.sourceEvent.ctrlKey && Lu() ? 10 : 1, R = -S.sourceEvent.deltaY * (S.sourceEvent.deltaMode === 1 ? 0.05 : S.sourceEvent.deltaMode ? 1 : 2e-3) * v, q = T[2] * Math.pow(2, R * N);
      r.scaleTo(q);
    };
    let A = [0, 0];
    const w = (S) => {
      (S.sourceEvent.type === "mousedown" || S.sourceEvent.type === "touchstart") && (A = [
        S.sourceEvent.clientX ?? S.sourceEvent.touches[0].clientX,
        S.sourceEvent.clientY ?? S.sourceEvent.touches[0].clientY
      ]);
    }, E = (S) => {
      const T = i();
      if (S.sourceEvent.type !== "mousemove" && S.sourceEvent.type !== "touchmove" || !r)
        return;
      const N = [
        S.sourceEvent.clientX ?? S.sourceEvent.touches[0].clientX,
        S.sourceEvent.clientY ?? S.sourceEvent.touches[0].clientY
      ], R = [N[0] - A[0], N[1] - A[1]];
      A = N;
      const q = u() * Math.max(T[2], Math.log(T[2])) * (b ? -1 : 1), L = {
        x: T[0] - R[0] * q,
        y: T[1] - R[1] * q
      }, j = [
        [0, 0],
        [g, h]
      ];
      r.setViewportConstrained({
        x: L.x,
        y: L.y,
        zoom: T[2]
      }, j, f);
    }, M = mM().on("start", w).on("zoom", y ? E : null).on("zoom.wheel", m ? _ : null);
    l.call(M, {});
  }
  function c() {
    l.on("zoom", null);
  }
  return {
    update: s,
    destroy: c,
    pointer: cn
  };
}
const ys = (t) => ({
  x: t.x,
  y: t.y,
  zoom: t.k
}), fy = ({ x: t, y: r, zoom: i }) => gs.translate(t, r).scale(i), ai = (t, r) => t.target.closest(`.${r}`), ZM = (t, r) => r === 2 && Array.isArray(t) && t.includes(2), z6 = (t) => ((t *= 2) <= 1 ? t * t * t : (t -= 2) * t * t + 2) / 2, dy = (t, r = 0, i = z6, u = () => {
}) => {
  const l = typeof r == "number" && r > 0;
  return l || u(), l ? t.transition().duration(r).ease(i).on("end", u) : t;
}, KM = (t) => {
  const r = t.ctrlKey && Lu() ? 10 : 1;
  return -t.deltaY * (t.deltaMode === 1 ? 0.05 : t.deltaMode ? 1 : 2e-3) * r;
};
function O6({ zoomPanValues: t, noWheelClassName: r, d3Selection: i, d3Zoom: u, panOnScrollMode: l, panOnScrollSpeed: s, zoomOnPinch: c, onPanZoomStart: f, onPanZoom: g, onPanZoomEnd: h }) {
  return (v) => {
    if (ai(v, r))
      return v.ctrlKey && v.preventDefault(), !1;
    v.preventDefault(), v.stopImmediatePropagation();
    const y = i.property("__zoom").k || 1;
    if (v.ctrlKey && c) {
      const w = cn(v), E = KM(v), M = y * Math.pow(2, E);
      u.scaleTo(i, M, w, v);
      return;
    }
    const m = v.deltaMode === 1 ? 20 : 1;
    let b = l === ia.Vertical ? 0 : v.deltaX * m, _ = l === ia.Horizontal ? 0 : v.deltaY * m;
    !Lu() && v.shiftKey && l !== ia.Vertical && (b = v.deltaY * m, _ = 0), u.translateBy(
      i,
      -(b / y) * s,
      -(_ / y) * s,
      // @ts-ignore
      { internal: !0 }
    );
    const A = ys(i.property("__zoom"));
    clearTimeout(t.panScrollTimeout), t.isPanScrolling ? g == null || g(v, A) : (t.isPanScrolling = !0, f == null || f(v, A)), t.panScrollTimeout = setTimeout(() => {
      h == null || h(v, A), t.isPanScrolling = !1;
    }, 150);
  };
}
function D6({ noWheelClassName: t, preventScrolling: r, d3ZoomHandler: i }) {
  return function(u, l) {
    const s = u.type === "wheel", c = !r && s && !u.ctrlKey, f = ai(u, t);
    if (u.ctrlKey && s && f && u.preventDefault(), c || f)
      return null;
    u.preventDefault(), i.call(this, u, l);
  };
}
function L6({ zoomPanValues: t, onDraggingChange: r, onPanZoomStart: i }) {
  return (u) => {
    var s, c, f;
    if ((s = u.sourceEvent) != null && s.internal)
      return;
    const l = ys(u.transform);
    t.mouseButton = ((c = u.sourceEvent) == null ? void 0 : c.button) || 0, t.isZoomingOrPanning = !0, t.prevViewport = l, ((f = u.sourceEvent) == null ? void 0 : f.type) === "mousedown" && r(!0), i && (i == null || i(u.sourceEvent, l));
  };
}
function H6({ zoomPanValues: t, panOnDrag: r, onPaneContextMenu: i, onTransformChange: u, onPanZoom: l }) {
  return (s) => {
    var c, f;
    t.usedRightMouseButton = !!(i && ZM(r, t.mouseButton ?? 0)), (c = s.sourceEvent) != null && c.sync || u([s.transform.x, s.transform.y, s.transform.k]), l && !((f = s.sourceEvent) != null && f.internal) && (l == null || l(s.sourceEvent, ys(s.transform)));
  };
}
function j6({ zoomPanValues: t, panOnDrag: r, panOnScroll: i, onDraggingChange: u, onPanZoomEnd: l, onPaneContextMenu: s }) {
  return (c) => {
    var f;
    if (!((f = c.sourceEvent) != null && f.internal) && (t.isZoomingOrPanning = !1, s && ZM(r, t.mouseButton ?? 0) && !t.usedRightMouseButton && c.sourceEvent && s(c.sourceEvent), t.usedRightMouseButton = !1, u(!1), l)) {
      const g = ys(c.transform);
      t.prevViewport = g, clearTimeout(t.timerId), t.timerId = setTimeout(
        () => {
          l == null || l(c.sourceEvent, g);
        },
        // we need a setTimeout for panOnScroll to suppress multiple end events fired during scroll
        i ? 150 : 0
      );
    }
  };
}
function B6({ zoomActivationKeyPressed: t, zoomOnScroll: r, zoomOnPinch: i, panOnDrag: u, panOnScroll: l, zoomOnDoubleClick: s, userSelectionActive: c, noWheelClassName: f, noPanClassName: g, lib: h, connectionInProgress: v }) {
  return (y) => {
    var w;
    const m = t || r, b = i && y.ctrlKey, _ = y.type === "wheel";
    if (y.button === 1 && y.type === "mousedown" && (ai(y, `${h}-flow__node`) || ai(y, `${h}-flow__edge`)))
      return !0;
    if (!u && !m && !l && !s && !i || c || v && !_ || ai(y, f) && _ || ai(y, g) && (!_ || l && _ && !t) || !i && y.ctrlKey && _)
      return !1;
    if (!i && y.type === "touchstart" && ((w = y.touches) == null ? void 0 : w.length) > 1)
      return y.preventDefault(), !1;
    if (!m && !l && !b && _ || !u && (y.type === "mousedown" || y.type === "touchstart") || Array.isArray(u) && !u.includes(y.button) && y.type === "mousedown")
      return !1;
    const A = Array.isArray(u) && u.includes(y.button) || !y.button || y.button <= 1;
    return (!y.ctrlKey || _) && A;
  };
}
function U6({ domNode: t, minZoom: r, maxZoom: i, translateExtent: u, viewport: l, onPanZoom: s, onPanZoomStart: c, onPanZoomEnd: f, onDraggingChange: g }) {
  const h = {
    isZoomingOrPanning: !1,
    usedRightMouseButton: !1,
    prevViewport: {},
    mouseButton: 0,
    timerId: void 0,
    panScrollTimeout: void 0,
    isPanScrolling: !1
  }, v = t.getBoundingClientRect();
  let y = [
    [0, 0],
    [v.width, v.height]
  ];
  const m = typeof ResizeObserver < "u" ? new ResizeObserver((Q) => {
    const D = Q[0];
    D && (y = [
      [0, 0],
      [D.contentRect.width, D.contentRect.height]
    ]);
  }) : null;
  m == null || m.observe(t);
  const b = mM().extent(() => y).scaleExtent([r, i]).translateExtent(u), _ = Kt(t).call(b);
  T({
    x: l.x,
    y: l.y,
    zoom: fi(l.zoom, r, i)
  }, [
    [0, 0],
    [v.width, v.height]
  ], u);
  const A = _.on("wheel.zoom"), w = _.on("dblclick.zoom");
  b.wheelDelta(KM);
  async function E(Q, D) {
    return _ ? new Promise((X) => {
      b == null || b.interpolate((D == null ? void 0 : D.interpolate) === "linear" ? Tu : No).transform(dy(_, D == null ? void 0 : D.duration, D == null ? void 0 : D.ease, () => X(!0)), Q);
    }) : !1;
  }
  function M({ noWheelClassName: Q, noPanClassName: D, onPaneContextMenu: X, userSelectionActive: z, panOnScroll: B, panOnDrag: $, panOnScrollMode: U, panOnScrollSpeed: ae, preventScrolling: O, zoomOnPinch: G, zoomOnScroll: ie, zoomOnDoubleClick: H, zoomActivationKeyPressed: K, lib: P, onTransformChange: ue, connectionInProgress: W, paneClickDistance: se, selectionOnDrag: de }) {
    z && !h.isZoomingOrPanning && S();
    const ve = B && !K && !z;
    b.clickDistance(de ? 1 / 0 : !dn(se) || se < 0 ? 0 : se);
    const ye = ve ? O6({
      zoomPanValues: h,
      noWheelClassName: Q,
      d3Selection: _,
      d3Zoom: b,
      panOnScrollMode: U,
      panOnScrollSpeed: ae,
      zoomOnPinch: G,
      onPanZoomStart: c,
      onPanZoom: s,
      onPanZoomEnd: f
    }) : D6({
      noWheelClassName: Q,
      preventScrolling: O,
      d3ZoomHandler: A
    });
    _.on("wheel.zoom", ye, { passive: !1 });
    const he = L6({
      zoomPanValues: h,
      onDraggingChange: g,
      onPanZoomStart: c
    });
    b.on("start", he);
    const me = H6({
      zoomPanValues: h,
      panOnDrag: $,
      onPaneContextMenu: !!X,
      onPanZoom: s,
      onTransformChange: ue
    });
    b.on("zoom", me);
    const ge = j6({
      zoomPanValues: h,
      panOnDrag: $,
      panOnScroll: B,
      onPaneContextMenu: X,
      onPanZoomEnd: f,
      onDraggingChange: g
    });
    b.on("end", ge);
    const Ae = B6({
      zoomActivationKeyPressed: K,
      panOnDrag: $,
      zoomOnScroll: ie,
      panOnScroll: B,
      zoomOnDoubleClick: H,
      zoomOnPinch: G,
      userSelectionActive: z,
      noPanClassName: D,
      noWheelClassName: Q,
      lib: P,
      connectionInProgress: W
    });
    b.filter(Ae), H ? _.on("dblclick.zoom", w) : _.on("dblclick.zoom", null);
  }
  function S() {
    b.on("zoom", null);
  }
  async function T(Q, D, X) {
    const z = fy(Q), B = b == null ? void 0 : b.constrain()(z, D, X);
    return B && await E(B), B;
  }
  async function N(Q, D) {
    const X = fy(Q);
    return await E(X, D), X;
  }
  function R(Q) {
    if (_) {
      const D = fy(Q), X = _.property("__zoom");
      (X.k !== Q.zoom || X.x !== Q.x || X.y !== Q.y) && (b == null || b.transform(_, D, null, { sync: !0 }));
    }
  }
  function q() {
    const Q = _ ? yM(_.node()) : { x: 0, y: 0, k: 1 };
    return { x: Q.x, y: Q.y, zoom: Q.k };
  }
  async function L(Q, D) {
    return _ ? new Promise((X) => {
      b == null || b.interpolate((D == null ? void 0 : D.interpolate) === "linear" ? Tu : No).scaleTo(dy(_, D == null ? void 0 : D.duration, D == null ? void 0 : D.ease, () => X(!0)), Q);
    }) : !1;
  }
  async function j(Q, D) {
    return _ ? new Promise((X) => {
      b == null || b.interpolate((D == null ? void 0 : D.interpolate) === "linear" ? Tu : No).scaleBy(dy(_, D == null ? void 0 : D.duration, D == null ? void 0 : D.ease, () => X(!0)), Q);
    }) : !1;
  }
  function Z(Q) {
    b == null || b.scaleExtent(Q);
  }
  function Y(Q) {
    b == null || b.translateExtent(Q);
  }
  function J(Q) {
    const D = !dn(Q) || Q < 0 ? 0 : Q;
    b == null || b.clickDistance(D);
  }
  return {
    update: M,
    destroy: S,
    setViewport: N,
    setViewportConstrained: T,
    getViewport: q,
    scaleTo: L,
    scaleBy: j,
    setScaleExtent: Z,
    setTranslateExtent: Y,
    syncViewport: R,
    setClickDistance: J
  };
}
var hi;
(function(t) {
  t.Line = "line", t.Handle = "handle";
})(hi || (hi = {}));
function G6({ width: t, prevWidth: r, height: i, prevHeight: u, affectsX: l, affectsY: s }) {
  const c = t - r, f = i - u, g = [c > 0 ? 1 : c < 0 ? -1 : 0, f > 0 ? 1 : f < 0 ? -1 : 0];
  return c && l && (g[0] = g[0] * -1), f && s && (g[1] = g[1] * -1), g;
}
function lA(t) {
  const r = t.includes("right") || t.includes("left"), i = t.includes("bottom") || t.includes("top"), u = t.includes("left"), l = t.includes("top");
  return {
    isHorizontal: r,
    isVertical: i,
    affectsX: u,
    affectsY: l
  };
}
function Nr(t, r) {
  return Math.max(0, r - t);
}
function zr(t, r) {
  return Math.max(0, t - r);
}
function To(t, r, i) {
  return Math.max(0, r - t, t - i);
}
function oA(t, r) {
  return t ? !r : r;
}
function V6(t, r, i, u, l, s, c, f) {
  let { affectsX: g, affectsY: h } = r;
  const { isHorizontal: v, isVertical: y } = r, m = v && y, { xSnapped: b, ySnapped: _ } = i, { minWidth: A, maxWidth: w, minHeight: E, maxHeight: M } = u, { x: S, y: T, width: N, height: R, aspectRatio: q } = t;
  let L = Math.floor(v ? b - t.pointerX : 0), j = Math.floor(y ? _ - t.pointerY : 0);
  const Z = N + (g ? -L : L), Y = R + (h ? -j : j), J = -s[0] * N, Q = -s[1] * R;
  let D = To(Z, A, w), X = To(Y, E, M);
  if (c) {
    let $ = 0, U = 0;
    g && L < 0 ? $ = Nr(S + L + J, c[0][0]) : !g && L > 0 && ($ = zr(S + Z + J, c[1][0])), h && j < 0 ? U = Nr(T + j + Q, c[0][1]) : !h && j > 0 && (U = zr(T + Y + Q, c[1][1])), D = Math.max(D, $), X = Math.max(X, U);
  }
  if (f) {
    let $ = 0, U = 0;
    g && L > 0 ? $ = zr(S + L, f[0][0]) : !g && L < 0 && ($ = Nr(S + Z, f[1][0])), h && j > 0 ? U = zr(T + j, f[0][1]) : !h && j < 0 && (U = Nr(T + Y, f[1][1])), D = Math.max(D, $), X = Math.max(X, U);
  }
  if (l) {
    if (v) {
      const $ = To(Z / q, E, M) * q;
      if (D = Math.max(D, $), c) {
        let U = 0;
        !g && !h || g && !h && m ? U = zr(T + Q + Z / q, c[1][1]) * q : U = Nr(T + Q + (g ? L : -L) / q, c[0][1]) * q, D = Math.max(D, U);
      }
      if (f) {
        let U = 0;
        !g && !h || g && !h && m ? U = Nr(T + Z / q, f[1][1]) * q : U = zr(T + (g ? L : -L) / q, f[0][1]) * q, D = Math.max(D, U);
      }
    }
    if (y) {
      const $ = To(Y * q, A, w) / q;
      if (X = Math.max(X, $), c) {
        let U = 0;
        !g && !h || h && !g && m ? U = zr(S + Y * q + J, c[1][0]) / q : U = Nr(S + (h ? j : -j) * q + J, c[0][0]) / q, X = Math.max(X, U);
      }
      if (f) {
        let U = 0;
        !g && !h || h && !g && m ? U = Nr(S + Y * q, f[1][0]) / q : U = zr(S + (h ? j : -j) * q, f[0][0]) / q, X = Math.max(X, U);
      }
    }
  }
  j = j + (j < 0 ? X : -X), L = L + (L < 0 ? D : -D), l && (m ? Z > Y * q ? j = (oA(g, h) ? -L : L) / q : L = (oA(g, h) ? -j : j) * q : v ? (j = L / q, h = g) : (L = j * q, g = h));
  const z = g ? S + L : S, B = h ? T + j : T;
  return {
    width: N + (g ? -L : L),
    height: R + (h ? -j : j),
    x: s[0] * L * (g ? -1 : 1) + z,
    y: s[1] * j * (h ? -1 : 1) + B
  };
}
const QM = { width: 0, height: 0, x: 0, y: 0 }, Y6 = {
  ...QM,
  pointerX: 0,
  pointerY: 0,
  aspectRatio: 1
};
function k6(t, r, i) {
  const u = r.position.x + t.position.x, l = r.position.y + t.position.y, s = t.measured.width ?? 0, c = t.measured.height ?? 0, f = i[0] * s, g = i[1] * c;
  return [
    [u - f, l - g],
    [u + s - f, l + c - g]
  ];
}
function X6({ domNode: t, nodeId: r, getStoreItems: i, onChange: u, onEnd: l }) {
  const s = Kt(t);
  let c = {
    controlDirection: lA("bottom-right"),
    boundaries: {
      minWidth: 0,
      minHeight: 0,
      maxWidth: Number.MAX_VALUE,
      maxHeight: Number.MAX_VALUE
    },
    resizeDirection: void 0,
    keepAspectRatio: !1
  };
  function f({ controlPosition: h, boundaries: v, keepAspectRatio: y, resizeDirection: m, onResizeStart: b, onResize: _, onResizeEnd: A, shouldResize: w }) {
    let E = { ...QM }, M = { ...Y6 };
    c = {
      boundaries: v,
      resizeDirection: m,
      keepAspectRatio: y,
      controlDirection: lA(h)
    };
    let S, T = null, N = [], R, q, L, j = !1;
    const Z = rM().on("start", (Y) => {
      const { nodeLookup: J, transform: Q, snapGrid: D, snapToGrid: X, nodeOrigin: z, paneDomNode: B } = i();
      if (S = J.get(r), !S)
        return;
      T = (B == null ? void 0 : B.getBoundingClientRect()) ?? null;
      const { xSnapped: $, ySnapped: U } = Mu(Y.sourceEvent, {
        transform: Q,
        snapGrid: D,
        snapToGrid: X,
        containerBounds: T
      });
      E = {
        width: S.measured.width ?? 0,
        height: S.measured.height ?? 0,
        x: S.position.x ?? 0,
        y: S.position.y ?? 0
      }, M = {
        ...E,
        pointerX: $,
        pointerY: U,
        aspectRatio: E.width / E.height
      }, R = void 0, q = sa(S.extent) ? S.extent : void 0, S.parentId && (S.extent === "parent" || S.expandParent) && (R = J.get(S.parentId)), R && S.extent === "parent" && (q = [
        [0, 0],
        [R.measured.width, R.measured.height]
      ]), N = [], L = void 0;
      for (const [ae, O] of J)
        if (O.parentId === r && (N.push({
          id: ae,
          position: { ...O.position },
          extent: O.extent
        }), O.extent === "parent" || O.expandParent)) {
          const G = k6(O, S, O.origin ?? z);
          L ? L = [
            [Math.min(G[0][0], L[0][0]), Math.min(G[0][1], L[0][1])],
            [Math.max(G[1][0], L[1][0]), Math.max(G[1][1], L[1][1])]
          ] : L = G;
        }
      b == null || b(Y, { ...E });
    }).on("drag", (Y) => {
      const { transform: J, snapGrid: Q, snapToGrid: D, nodeOrigin: X } = i(), z = Mu(Y.sourceEvent, {
        transform: J,
        snapGrid: Q,
        snapToGrid: D,
        containerBounds: T
      }), B = [];
      if (!S)
        return;
      const { x: $, y: U, width: ae, height: O } = E, G = {}, ie = S.origin ?? X, { width: H, height: K, x: P, y: ue } = V6(M, c.controlDirection, z, c.boundaries, c.keepAspectRatio, ie, q, L), W = H !== ae, se = K !== O, de = P !== $ && W, ve = ue !== U && se;
      if (!de && !ve && !W && !se)
        return;
      if ((de || ve || ie[0] === 1 || ie[1] === 1) && (G.x = de ? P : E.x, G.y = ve ? ue : E.y, E.x = G.x, E.y = G.y, N.length > 0)) {
        const ge = P - $, Ae = ue - U;
        for (const xe of N)
          xe.position = {
            x: xe.position.x - ge + ie[0] * (H - ae),
            y: xe.position.y - Ae + ie[1] * (K - O)
          }, B.push(xe);
      }
      if ((W || se) && (G.width = W && (!c.resizeDirection || c.resizeDirection === "horizontal") ? H : E.width, G.height = se && (!c.resizeDirection || c.resizeDirection === "vertical") ? K : E.height, E.width = G.width, E.height = G.height), R && S.expandParent) {
        const ge = ie[0] * (G.width ?? 0);
        G.x && G.x < ge && (E.x = ge, M.x = M.x - (G.x - ge));
        const Ae = ie[1] * (G.height ?? 0);
        G.y && G.y < Ae && (E.y = Ae, M.y = M.y - (G.y - Ae));
      }
      const ye = G6({
        width: E.width,
        prevWidth: ae,
        height: E.height,
        prevHeight: O,
        affectsX: c.controlDirection.affectsX,
        affectsY: c.controlDirection.affectsY
      }), he = { ...E, direction: ye };
      (w == null ? void 0 : w(Y, he)) !== !1 && (j = !0, _ == null || _(Y, he), u(G, B));
    }).on("end", (Y) => {
      j && (A == null || A(Y, { ...E }), l == null || l({ ...E }), j = !1);
    });
    s.call(Z);
  }
  function g() {
    s.on(".drag", null);
  }
  return {
    update: f,
    destroy: g
  };
}
var hy = { exports: {} }, gy = {}, vy = { exports: {} }, py = {};
/**
 * @license React
 * use-sync-external-store-shim.production.js
 *
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
var sA;
function I6() {
  if (sA) return py;
  sA = 1;
  var t = ju();
  function r(y, m) {
    return y === m && (y !== 0 || 1 / y === 1 / m) || y !== y && m !== m;
  }
  var i = typeof Object.is == "function" ? Object.is : r, u = t.useState, l = t.useEffect, s = t.useLayoutEffect, c = t.useDebugValue;
  function f(y, m) {
    var b = m(), _ = u({ inst: { value: b, getSnapshot: m } }), A = _[0].inst, w = _[1];
    return s(
      function() {
        A.value = b, A.getSnapshot = m, g(A) && w({ inst: A });
      },
      [y, b, m]
    ), l(
      function() {
        return g(A) && w({ inst: A }), y(function() {
          g(A) && w({ inst: A });
        });
      },
      [y]
    ), c(b), b;
  }
  function g(y) {
    var m = y.getSnapshot;
    y = y.value;
    try {
      var b = m();
      return !i(y, b);
    } catch {
      return !0;
    }
  }
  function h(y, m) {
    return m();
  }
  var v = typeof window > "u" || typeof window.document > "u" || typeof window.document.createElement > "u" ? h : f;
  return py.useSyncExternalStore = t.useSyncExternalStore !== void 0 ? t.useSyncExternalStore : v, py;
}
var cA;
function Z6() {
  return cA || (cA = 1, vy.exports = I6()), vy.exports;
}
/**
 * @license React
 * use-sync-external-store-shim/with-selector.production.js
 *
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
var fA;
function K6() {
  if (fA) return gy;
  fA = 1;
  var t = ju(), r = Z6();
  function i(h, v) {
    return h === v && (h !== 0 || 1 / h === 1 / v) || h !== h && v !== v;
  }
  var u = typeof Object.is == "function" ? Object.is : i, l = r.useSyncExternalStore, s = t.useRef, c = t.useEffect, f = t.useMemo, g = t.useDebugValue;
  return gy.useSyncExternalStoreWithSelector = function(h, v, y, m, b) {
    var _ = s(null);
    if (_.current === null) {
      var A = { hasValue: !1, value: null };
      _.current = A;
    } else A = _.current;
    _ = f(
      function() {
        function E(R) {
          if (!M) {
            if (M = !0, S = R, R = m(R), b !== void 0 && A.hasValue) {
              var q = A.value;
              if (b(q, R))
                return T = q;
            }
            return T = R;
          }
          if (q = T, u(S, R)) return q;
          var L = m(R);
          return b !== void 0 && b(q, L) ? (S = R, q) : (S = R, T = L);
        }
        var M = !1, S, T, N = y === void 0 ? null : y;
        return [
          function() {
            return E(v());
          },
          N === null ? void 0 : function() {
            return E(N());
          }
        ];
      },
      [v, y, m, b]
    );
    var w = l(h, _[0], _[1]);
    return c(
      function() {
        A.hasValue = !0, A.value = w;
      },
      [w]
    ), g(w), w;
  }, gy;
}
var dA;
function Q6() {
  return dA || (dA = 1, hy.exports = K6()), hy.exports;
}
var $6 = Q6();
const F6 = /* @__PURE__ */ Ly($6), J6 = {}, hA = (t) => {
  let r;
  const i = /* @__PURE__ */ new Set(), u = (v, y) => {
    const m = typeof v == "function" ? v(r) : v;
    if (!Object.is(m, r)) {
      const b = r;
      r = y ?? (typeof m != "object" || m === null) ? m : Object.assign({}, r, m), i.forEach((_) => _(r, b));
    }
  }, l = () => r, g = { setState: u, getState: l, getInitialState: () => h, subscribe: (v) => (i.add(v), () => i.delete(v)), destroy: () => {
    (J6 ? "production" : void 0) !== "production" && console.warn(
      "[DEPRECATED] The `destroy` method will be unsupported in a future version. Instead use unsubscribe function returned by subscribe. Everything will be garbage-collected if store is garbage-collected."
    ), i.clear();
  } }, h = r = t(u, l, g);
  return g;
}, P6 = (t) => t ? hA(t) : hA, { useDebugValue: W6 } = ZR, { useSyncExternalStoreWithSelector: e8 } = F6, t8 = (t) => t;
function $M(t, r = t8, i) {
  const u = e8(
    t.subscribe,
    t.getState,
    t.getServerState || t.getInitialState,
    r,
    i
  );
  return W6(u), u;
}
const gA = (t, r) => {
  const i = P6(t), u = (l, s = r) => $M(i, l, s);
  return Object.assign(u, i), u;
}, n8 = (t, r) => t ? gA(t, r) : gA;
function $e(t, r) {
  if (Object.is(t, r))
    return !0;
  if (typeof t != "object" || t === null || typeof r != "object" || r === null)
    return !1;
  if (t instanceof Map && r instanceof Map) {
    if (t.size !== r.size) return !1;
    for (const [u, l] of t)
      if (!Object.is(l, r.get(u)))
        return !1;
    return !0;
  }
  if (t instanceof Set && r instanceof Set) {
    if (t.size !== r.size) return !1;
    for (const u of t)
      if (!r.has(u))
        return !1;
    return !0;
  }
  const i = Object.keys(t);
  if (i.length !== Object.keys(r).length)
    return !1;
  for (const u of i)
    if (!Object.prototype.hasOwnProperty.call(r, u) || !Object.is(t[u], r[u]))
      return !1;
  return !0;
}
kA();
const ms = re.createContext(null), r8 = ms.Provider, FM = vn.error001("react");
function Oe(t, r) {
  const i = re.useContext(ms);
  if (i === null)
    throw new Error(FM);
  return $M(i, t, r);
}
function Ze() {
  const t = re.useContext(ms);
  if (t === null)
    throw new Error(FM);
  return re.useMemo(() => ({
    getState: t.getState,
    setState: t.setState,
    subscribe: t.subscribe
  }), [t]);
}
const vA = { display: "none" }, a8 = {
  position: "absolute",
  width: 1,
  height: 1,
  margin: -1,
  border: 0,
  padding: 0,
  overflow: "hidden",
  clip: "rect(0px, 0px, 0px, 0px)",
  clipPath: "inset(100%)"
}, JM = "react-flow__node-desc", PM = "react-flow__edge-desc", i8 = "react-flow__aria-live", u8 = (t) => t.ariaLiveMessage, l8 = (t) => t.ariaLabelConfig;
function o8({ rfId: t }) {
  const r = Oe(u8);
  return k.jsx("div", { id: `${i8}-${t}`, "aria-live": "assertive", "aria-atomic": "true", style: a8, children: r });
}
function s8({ rfId: t, disableKeyboardA11y: r }) {
  const i = Oe(l8);
  return k.jsxs(k.Fragment, { children: [k.jsx("div", { id: `${JM}-${t}`, style: vA, children: r ? i["node.a11yDescription.default"] : i["node.a11yDescription.keyboardDisabled"] }), k.jsx("div", { id: `${PM}-${t}`, style: vA, children: i["edge.a11yDescription.default"] }), !r && k.jsx(o8, { rfId: t })] });
}
const $u = re.forwardRef(({ position: t = "top-left", children: r, className: i, style: u, ...l }, s) => {
  const c = `${t}`.split("-");
  return k.jsx("div", { className: ct(["react-flow__panel", i, ...c]), style: u, ref: s, ...l, children: r });
});
$u.displayName = "Panel";
const pA = "https://reactflow.dev?utm_source=attribution";
function c8({ proOptions: t, position: r = "bottom-right" }) {
  return t != null && t.hideAttribution ? null : k.jsx($u, { position: r, className: "react-flow__attribution", "data-message": `Please only hide this attribution when you are subscribed to React Flow Pro: ${pA}`, children: k.jsx("a", { href: pA, target: "_blank", rel: "noopener noreferrer", "aria-label": "React Flow attribution", children: "React Flow" }) });
}
const f8 = (t) => {
  const r = [], i = [];
  for (const [, u] of t.nodeLookup)
    u.selected && r.push(u.internals.userNode);
  for (const [, u] of t.edgeLookup)
    u.selected && i.push(u);
  return { selectedNodes: r, selectedEdges: i };
}, Mo = (t) => t.id;
function d8(t, r) {
  return $e(t.selectedNodes.map(Mo), r.selectedNodes.map(Mo)) && $e(t.selectedEdges.map(Mo), r.selectedEdges.map(Mo));
}
function h8({ onSelectionChange: t }) {
  const r = Ze(), { selectedNodes: i, selectedEdges: u } = Oe(f8, d8);
  return re.useEffect(() => {
    const l = { nodes: i, edges: u };
    t == null || t(l), r.getState().onSelectionChangeHandlers.forEach((s) => s(l));
  }, [i, u, t]), null;
}
const g8 = (t) => !!t.onSelectionChangeHandlers;
function v8({ onSelectionChange: t }) {
  const r = Oe(g8);
  return t || r ? k.jsx(h8, { onSelectionChange: t }) : null;
}
const WM = [0, 0], p8 = { x: 0, y: 0, zoom: 1 }, y8 = [
  "nodes",
  "edges",
  "defaultNodes",
  "defaultEdges",
  "onConnect",
  "onConnectStart",
  "onConnectEnd",
  "onClickConnectStart",
  "onClickConnectEnd",
  "nodesDraggable",
  "autoPanOnNodeFocus",
  "nodesConnectable",
  "nodesFocusable",
  "edgesFocusable",
  "edgesReconnectable",
  "elevateNodesOnSelect",
  "elevateEdgesOnSelect",
  "minZoom",
  "maxZoom",
  "nodeExtent",
  "onNodesChange",
  "onEdgesChange",
  "elementsSelectable",
  "connectionMode",
  "snapGrid",
  "snapToGrid",
  "translateExtent",
  "connectOnClick",
  "defaultEdgeOptions",
  "fitView",
  "fitViewOptions",
  "onNodesDelete",
  "onEdgesDelete",
  "onDelete",
  "onNodeDrag",
  "onNodeDragStart",
  "onNodeDragStop",
  "onSelectionDrag",
  "onSelectionDragStart",
  "onSelectionDragStop",
  "onMoveStart",
  "onMove",
  "onMoveEnd",
  "noPanClassName",
  "nodeOrigin",
  "autoPanOnConnect",
  "autoPanOnNodeDrag",
  "onError",
  "connectionRadius",
  "isValidConnection",
  "selectNodesOnDrag",
  "nodeDragThreshold",
  "connectionDragThreshold",
  "onBeforeDelete",
  "debug",
  "autoPanSpeed",
  "ariaLabelConfig",
  "zIndexMode"
], yA = [...y8, "rfId"], m8 = (t) => ({
  setNodes: t.setNodes,
  setEdges: t.setEdges,
  setMinZoom: t.setMinZoom,
  setMaxZoom: t.setMaxZoom,
  setTranslateExtent: t.setTranslateExtent,
  setNodeExtent: t.setNodeExtent,
  reset: t.reset,
  setDefaultNodesAndEdges: t.setDefaultNodesAndEdges
}), mA = {
  /*
   * these are values that are also passed directly to other components
   * than the StoreUpdater. We can reduce the number of setStore calls
   * by setting the same values here as prev fields.
   */
  translateExtent: zu,
  nodeOrigin: WM,
  minZoom: 0.5,
  maxZoom: 2,
  elementsSelectable: !0,
  noPanClassName: "nopan",
  rfId: "1"
};
function b8(t) {
  const { setNodes: r, setEdges: i, setMinZoom: u, setMaxZoom: l, setTranslateExtent: s, setNodeExtent: c, reset: f, setDefaultNodesAndEdges: g } = Oe(m8, $e), h = Ze();
  re.useEffect(() => (g(t.defaultNodes, t.defaultEdges), () => {
    v.current = mA, f();
  }), []);
  const v = re.useRef(mA);
  return re.useEffect(
    () => {
      for (const y of yA) {
        const m = t[y], b = v.current[y];
        m !== b && (typeof t[y] > "u" || (y === "nodes" ? r(m) : y === "edges" ? i(m) : y === "minZoom" ? u(m) : y === "maxZoom" ? l(m) : y === "translateExtent" ? s(m) : y === "nodeExtent" ? c(m) : y === "ariaLabelConfig" ? h.setState({ ariaLabelConfig: r6(m) }) : y === "fitView" ? h.setState({ fitViewQueued: m }) : y === "fitViewOptions" ? h.setState({ fitViewOptions: m }) : h.setState({ [y]: m })));
      }
      v.current = t;
    },
    // Only re-run the effect if one of the fields we track changes
    yA.map((y) => t[y])
  ), null;
}
function bA() {
  return typeof window > "u" || !window.matchMedia ? null : window.matchMedia("(prefers-color-scheme: dark)");
}
function _8(t) {
  var u;
  const [r, i] = re.useState(t === "system" ? null : t);
  return re.useEffect(() => {
    if (t !== "system") {
      i(t);
      return;
    }
    const l = bA(), s = () => i(l != null && l.matches ? "dark" : "light");
    return s(), l == null || l.addEventListener("change", s), () => {
      l == null || l.removeEventListener("change", s);
    };
  }, [t]), r !== null ? r : (u = bA()) != null && u.matches ? "dark" : "light";
}
const _A = typeof document < "u" ? document : null;
function Hu(t = null, r = { target: _A, actInsideInputWithModifier: !0 }) {
  const [i, u] = re.useState(!1), l = re.useRef(!1), s = re.useRef(/* @__PURE__ */ new Set([])), [c, f] = re.useMemo(() => {
    if (t !== null) {
      const h = (Array.isArray(t) ? t : [t]).filter((y) => typeof y == "string").map((y) => y.replace("+", `
`).replace(`

`, `
+`).split(`
`)), v = h.reduce((y, m) => y.concat(...m), []);
      return [h, v];
    }
    return [[], []];
  }, [t]);
  return re.useEffect(() => {
    const g = (r == null ? void 0 : r.target) ?? _A, h = (r == null ? void 0 : r.actInsideInputWithModifier) ?? !0;
    if (t !== null) {
      const v = (b) => {
        var w, E;
        if (l.current = b.ctrlKey || b.metaKey || b.shiftKey || b.altKey, (!l.current || l.current && !h) && zM(b))
          return !1;
        const A = SA(b.code, f);
        if (s.current.add(b[A]), xA(c, s.current, !1)) {
          const M = ((E = (w = b.composedPath) == null ? void 0 : w.call(b)) == null ? void 0 : E[0]) || b.target, S = (M == null ? void 0 : M.nodeName) === "BUTTON" || (M == null ? void 0 : M.nodeName) === "A";
          r.preventDefault !== !1 && (l.current || !S) && b.preventDefault(), u(!0);
        }
      }, y = (b) => {
        const _ = SA(b.code, f);
        xA(c, s.current, !0) ? (u(!1), s.current.clear()) : s.current.delete(b[_]), b.key === "Meta" && s.current.clear(), l.current = !1;
      }, m = () => {
        s.current.clear(), u(!1);
      };
      return g == null || g.addEventListener("keydown", v), g == null || g.addEventListener("keyup", y), window.addEventListener("blur", m), window.addEventListener("contextmenu", m), () => {
        g == null || g.removeEventListener("keydown", v), g == null || g.removeEventListener("keyup", y), window.removeEventListener("blur", m), window.removeEventListener("contextmenu", m);
      };
    }
  }, [t, u]), i;
}
function xA(t, r, i) {
  return t.filter((u) => i || u.length === r.size).some((u) => u.every((l) => r.has(l)));
}
function SA(t, r) {
  return r.includes(t) ? "code" : "key";
}
const x8 = () => {
  const t = Ze();
  return re.useMemo(() => ({
    zoomIn: async (r) => {
      const { panZoom: i } = t.getState();
      return i ? i.scaleBy(1.2, r) : !1;
    },
    zoomOut: async (r) => {
      const { panZoom: i } = t.getState();
      return i ? i.scaleBy(1 / 1.2, r) : !1;
    },
    zoomTo: async (r, i) => {
      const { panZoom: u } = t.getState();
      return u ? u.scaleTo(r, i) : !1;
    },
    getZoom: () => t.getState().transform[2],
    setViewport: async (r, i) => {
      const { transform: [u, l, s], panZoom: c } = t.getState();
      return c ? (await c.setViewport({
        x: r.x ?? u,
        y: r.y ?? l,
        zoom: r.zoom ?? s
      }, i), !0) : !1;
    },
    getViewport: () => {
      const [r, i, u] = t.getState().transform;
      return { x: r, y: i, zoom: u };
    },
    setCenter: async (r, i, u) => t.getState().setCenter(r, i, u),
    fitBounds: async (r, i) => {
      const { width: u, height: l, minZoom: s, maxZoom: c, panZoom: f } = t.getState(), g = cm(r, u, l, s, c, (i == null ? void 0 : i.padding) ?? 0.1);
      return f ? (await f.setViewport(g, {
        duration: i == null ? void 0 : i.duration,
        ease: i == null ? void 0 : i.ease,
        interpolate: i == null ? void 0 : i.interpolate
      }), !0) : !1;
    },
    screenToFlowPosition: (r, i = {}) => {
      const { transform: u, snapGrid: l, snapToGrid: s, domNode: c } = t.getState();
      if (!c)
        return r;
      const { x: f, y: g } = c.getBoundingClientRect(), h = {
        x: r.x - f,
        y: r.y - g
      }, v = i.snapGrid ?? l, y = i.snapToGrid ?? s;
      return Qu(h, u, y, v);
    },
    flowToScreenPosition: (r) => {
      const { transform: i, domNode: u } = t.getState();
      if (!u)
        return r;
      const { x: l, y: s } = u.getBoundingClientRect(), c = di(r, i);
      return {
        x: c.x + l,
        y: c.y + s
      };
    }
  }), []);
};
function eC(t, r) {
  const i = [], u = /* @__PURE__ */ new Map(), l = [];
  for (const s of t)
    if (s.type === "add") {
      l.push(s);
      continue;
    } else if (s.type === "remove" || s.type === "replace")
      u.set(s.id, [s]);
    else {
      const c = u.get(s.id);
      c ? c.push(s) : u.set(s.id, [s]);
    }
  for (const s of r) {
    const c = u.get(s.id);
    if (!c) {
      i.push(s);
      continue;
    }
    if (c[0].type === "remove")
      continue;
    if (c[0].type === "replace") {
      i.push({ ...c[0].item });
      continue;
    }
    const f = { ...s };
    for (const g of c)
      S8(g, f);
    i.push(f);
  }
  return l.length && l.forEach((s) => {
    s.index !== void 0 ? i.splice(s.index, 0, { ...s.item }) : i.push({ ...s.item });
  }), i;
}
function S8(t, r) {
  switch (t.type) {
    case "select": {
      r.selected = t.selected;
      break;
    }
    case "position": {
      typeof t.position < "u" && (r.position = t.position), typeof t.dragging < "u" && (r.dragging = t.dragging);
      break;
    }
    case "dimensions": {
      typeof t.dimensions < "u" && (r.measured = {
        ...t.dimensions
      }, t.setAttributes && ((t.setAttributes === !0 || t.setAttributes === "width") && (r.width = t.dimensions.width), (t.setAttributes === !0 || t.setAttributes === "height") && (r.height = t.dimensions.height))), typeof t.resizing == "boolean" && (r.resizing = t.resizing);
      break;
    }
  }
}
function E8(t, r) {
  return eC(t, r);
}
function w8(t, r) {
  return eC(t, r);
}
function na(t, r) {
  return {
    id: t,
    type: "select",
    selected: r
  };
}
function ii(t, r = /* @__PURE__ */ new Set(), i = !1) {
  const u = [];
  for (const [l, s] of t) {
    const c = r.has(l);
    !(s.selected === void 0 && !c) && s.selected !== c && (i && (s.selected = c), u.push(na(s.id, c)));
  }
  return u;
}
function EA({ items: t = [], lookup: r }) {
  var l;
  const i = [], u = new Map(t.map((s) => [s.id, s]));
  for (const [s, c] of t.entries()) {
    const f = r.get(c.id), g = ((l = f == null ? void 0 : f.internals) == null ? void 0 : l.userNode) ?? f;
    g !== void 0 && g !== c && i.push({ id: c.id, item: c, type: "replace" }), g === void 0 && i.push({ item: c, type: "add", index: s });
  }
  for (const [s] of r)
    u.get(s) === void 0 && i.push({ id: s, type: "remove" });
  return i;
}
function wA(t) {
  return {
    id: t.id,
    type: "remove"
  };
}
const A8 = CM();
function T8(t, r, i = {}) {
  return s6(t, r, {
    ...i,
    onError: i.onError ?? A8
  });
}
const AA = (t) => Q5(t), M8 = (t) => EM(t);
function tC(t) {
  return re.forwardRef(t);
}
const nC = typeof window < "u" ? re.useLayoutEffect : re.useEffect;
function TA(t) {
  const [r, i] = re.useState(BigInt(0)), [u] = re.useState(() => C8(() => i((l) => l + BigInt(1))));
  return nC(() => {
    const l = u.get();
    l.length && (t(l), u.reset());
  }, [r]), u;
}
function C8(t) {
  let r = [];
  return {
    get: () => r,
    reset: () => {
      r = [];
    },
    push: (i) => {
      r.push(i), t();
    }
  };
}
const rC = re.createContext(null);
function q8({ children: t }) {
  const r = Ze(), i = re.useCallback((f) => {
    const { nodes: g = [], setNodes: h, hasDefaultNodes: v, onNodesChange: y, nodeLookup: m, fitViewQueued: b, onNodesChangeMiddlewareMap: _ } = r.getState();
    let A = g;
    for (const E of f)
      A = typeof E == "function" ? E(A) : E;
    let w = EA({
      items: A,
      lookup: m
    });
    for (const E of _.values())
      w = E(w);
    v && h(A), w.length > 0 ? y == null || y(w) : b && window.requestAnimationFrame(() => {
      const { fitViewQueued: E, nodes: M, setNodes: S } = r.getState();
      E && S(M);
    });
  }, []), u = TA(i), l = re.useCallback((f) => {
    const { edges: g = [], setEdges: h, hasDefaultEdges: v, onEdgesChange: y, edgeLookup: m } = r.getState();
    let b = g;
    for (const _ of f)
      b = typeof _ == "function" ? _(b) : _;
    v ? h(b) : y && y(EA({
      items: b,
      lookup: m
    }));
  }, []), s = TA(l), c = re.useMemo(() => ({ nodeQueue: u, edgeQueue: s }), []);
  return k.jsx(rC.Provider, { value: c, children: t });
}
function R8() {
  const t = re.useContext(rC);
  if (!t)
    throw new Error("useBatchContext must be used within a BatchProvider");
  return t;
}
const N8 = (t) => !!t.panZoom;
function bs() {
  const t = x8(), r = Ze(), i = R8(), u = Oe(N8), l = re.useMemo(() => {
    const s = (y) => r.getState().nodeLookup.get(y), c = (y) => {
      i.nodeQueue.push(y);
    }, f = (y) => {
      i.edgeQueue.push(y);
    }, g = (y) => {
      var E, M;
      const { nodeLookup: m, nodeOrigin: b } = r.getState(), _ = AA(y) ? y : m.get(y.id), A = _.parentId ? RM(_.position, _.measured, _.parentId, m, b) : _.position, w = {
        ..._,
        position: A,
        width: ((E = _.measured) == null ? void 0 : E.width) ?? _.width,
        height: ((M = _.measured) == null ? void 0 : M.height) ?? _.height
      };
      return Du(w);
    }, h = (y, m, b = { replace: !1 }) => {
      c((_) => _.map((A) => {
        if (A.id === y) {
          const w = typeof m == "function" ? m(A) : m;
          return b.replace && AA(w) ? w : { ...A, ...w };
        }
        return A;
      }));
    }, v = (y, m, b = { replace: !1 }) => {
      f((_) => _.map((A) => {
        if (A.id === y) {
          const w = typeof m == "function" ? m(A) : m;
          return b.replace && M8(w) ? w : { ...A, ...w };
        }
        return A;
      }));
    };
    return {
      getNodes: () => r.getState().nodes.map((y) => ({ ...y })),
      getNode: (y) => {
        var m;
        return (m = s(y)) == null ? void 0 : m.internals.userNode;
      },
      getInternalNode: s,
      getEdges: () => {
        const { edges: y = [] } = r.getState();
        return y.map((m) => ({ ...m }));
      },
      getEdge: (y) => r.getState().edgeLookup.get(y),
      setNodes: c,
      setEdges: f,
      addNodes: (y) => {
        const m = Array.isArray(y) ? y : [y];
        i.nodeQueue.push((b) => [...b, ...m]);
      },
      addEdges: (y) => {
        const m = Array.isArray(y) ? y : [y];
        i.edgeQueue.push((b) => [...b, ...m]);
      },
      toObject: () => {
        const { nodes: y = [], edges: m = [], transform: b } = r.getState(), [_, A, w] = b;
        return {
          nodes: y.map((E) => ({ ...E })),
          edges: m.map((E) => ({ ...E })),
          viewport: {
            x: _,
            y: A,
            zoom: w
          }
        };
      },
      deleteElements: async ({ nodes: y = [], edges: m = [] }) => {
        const { nodes: b, edges: _, onNodesDelete: A, onEdgesDelete: w, triggerNodeChanges: E, triggerEdgeChanges: M, onDelete: S, onBeforeDelete: T } = r.getState(), { nodes: N, edges: R } = await W5({
          nodesToRemove: y,
          edgesToRemove: m,
          nodes: b,
          edges: _,
          onBeforeDelete: T
        }), q = R.length > 0, L = N.length > 0;
        if (q) {
          const j = R.map(wA);
          w == null || w(R), M(j);
        }
        if (L) {
          const j = N.map(wA);
          A == null || A(N), E(j);
        }
        return (L || q) && (S == null || S({ nodes: N, edges: R })), { deletedNodes: N, deletedEdges: R };
      },
      /**
       * Partial is defined as "the 2 nodes/areas are intersecting partially".
       * If a is contained in b or b is contained in a, they are both
       * considered fully intersecting.
       */
      getIntersectingNodes: (y, m = !0, b) => {
        const _ = F2(y), A = _ ? y : g(y), w = b !== void 0;
        return A ? (b || r.getState().nodes).filter((E) => {
          const M = r.getState().nodeLookup.get(E.id);
          if (M && !_ && (E.id === y.id || !M.internals.positionAbsolute))
            return !1;
          const S = Du(w ? E : M), T = Io(S, A);
          return m && T > 0 || T >= S.width * S.height || T >= A.width * A.height;
        }) : [];
      },
      isNodeIntersecting: (y, m, b = !0) => {
        const A = F2(y) ? y : g(y);
        if (!A)
          return !1;
        const w = Io(A, m);
        return b && w > 0 || w >= m.width * m.height || w >= A.width * A.height;
      },
      updateNode: h,
      updateNodeData: (y, m, b = { replace: !1 }) => {
        h(y, (_) => {
          const A = typeof m == "function" ? m(_) : m;
          return b.replace ? { ..._, data: A } : { ..._, data: { ..._.data, ...A } };
        }, b);
      },
      updateEdge: v,
      updateEdgeData: (y, m, b = { replace: !1 }) => {
        v(y, (_) => {
          const A = typeof m == "function" ? m(_) : m;
          return b.replace ? { ..._, data: A } : { ..._, data: { ..._.data, ...A } };
        }, b);
      },
      getNodesBounds: (y) => {
        const { nodeLookup: m, nodeOrigin: b } = r.getState();
        return $5(y, { nodeLookup: m, nodeOrigin: b });
      },
      getHandleConnections: ({ type: y, id: m, nodeId: b }) => {
        var _;
        return Array.from(((_ = r.getState().connectionLookup.get(`${b}-${y}${m ? `-${m}` : ""}`)) == null ? void 0 : _.values()) ?? []);
      },
      getNodeConnections: ({ type: y, handleId: m, nodeId: b }) => {
        var _;
        return Array.from(((_ = r.getState().connectionLookup.get(`${b}${y ? m ? `-${y}-${m}` : `-${y}` : ""}`)) == null ? void 0 : _.values()) ?? []);
      },
      fitView: async (y) => {
        const m = r.getState().fitViewResolver ?? n6();
        return r.setState({ fitViewQueued: !0, fitViewOptions: y, fitViewResolver: m }), i.nodeQueue.push((b) => [...b]), m.promise;
      }
    };
  }, []);
  return re.useMemo(() => ({
    ...l,
    ...t,
    viewportInitialized: u
  }), [u]);
}
const MA = (t) => t.selected, z8 = typeof window < "u" ? window : void 0;
function O8({ deleteKeyCode: t, multiSelectionKeyCode: r }) {
  const i = Ze(), { deleteElements: u } = bs(), l = Hu(t, { actInsideInputWithModifier: !1 }), s = Hu(r, { target: z8 });
  re.useEffect(() => {
    if (l) {
      const { edges: c, nodes: f } = i.getState();
      u({ nodes: f.filter(MA), edges: c.filter(MA) }), i.setState({ nodesSelectionActive: !1 });
    }
  }, [l]), re.useEffect(() => {
    i.setState({ multiSelectionActive: s });
  }, [s]);
}
function D8(t) {
  const r = Ze();
  re.useEffect(() => {
    const i = () => {
      var l, s, c, f;
      if (!t.current || !(((s = (l = t.current).checkVisibility) == null ? void 0 : s.call(l)) ?? !0))
        return !1;
      const u = fm(t.current);
      (u.height === 0 || u.width === 0) && ((f = (c = r.getState()).onError) == null || f.call(c, "004", vn.error004())), r.setState({ width: u.width || 500, height: u.height || 500 });
    };
    if (t.current) {
      i(), window.addEventListener("resize", i);
      const u = new ResizeObserver(() => i());
      return u.observe(t.current), () => {
        window.removeEventListener("resize", i), u && t.current && u.unobserve(t.current);
      };
    }
  }, []);
}
const _s = {
  position: "absolute",
  width: "100%",
  height: "100%",
  top: 0,
  left: 0
}, L8 = (t) => ({
  userSelectionActive: t.userSelectionActive,
  lib: t.lib,
  connectionInProgress: t.connection.inProgress
});
function H8({ onPaneContextMenu: t, zoomOnScroll: r = !0, zoomOnPinch: i = !0, panOnScroll: u = !1, panOnScrollSpeed: l = 0.5, panOnScrollMode: s = ia.Free, zoomOnDoubleClick: c = !0, panOnDrag: f = !0, defaultViewport: g, translateExtent: h, minZoom: v, maxZoom: y, zoomActivationKeyCode: m, preventScrolling: b = !0, children: _, noWheelClassName: A, noPanClassName: w, onViewportChange: E, isControlledViewport: M, paneClickDistance: S, selectionOnDrag: T }) {
  const N = Ze(), R = re.useRef(null), { userSelectionActive: q, lib: L, connectionInProgress: j } = Oe(L8, $e), Z = Hu(m), Y = re.useRef();
  D8(R);
  const J = re.useCallback((Q) => {
    E == null || E({ x: Q[0], y: Q[1], zoom: Q[2] }), M || N.setState({ transform: Q });
  }, [E, M]);
  return re.useEffect(() => {
    if (R.current) {
      Y.current = U6({
        domNode: R.current,
        minZoom: v,
        maxZoom: y,
        translateExtent: h,
        viewport: g,
        onDraggingChange: (z) => N.setState((B) => B.paneDragging === z ? B : { paneDragging: z }),
        onPanZoomStart: (z, B) => {
          const { onViewportChangeStart: $, onMoveStart: U } = N.getState();
          U == null || U(z, B), $ == null || $(B);
        },
        onPanZoom: (z, B) => {
          const { onViewportChange: $, onMove: U } = N.getState();
          U == null || U(z, B), $ == null || $(B);
        },
        onPanZoomEnd: (z, B) => {
          const { onViewportChangeEnd: $, onMoveEnd: U } = N.getState();
          U == null || U(z, B), $ == null || $(B);
        }
      });
      const { x: Q, y: D, zoom: X } = Y.current.getViewport();
      return N.setState({
        panZoom: Y.current,
        transform: [Q, D, X],
        domNode: R.current.closest(".react-flow")
      }), () => {
        var z;
        (z = Y.current) == null || z.destroy();
      };
    }
  }, []), re.useEffect(() => {
    var Q;
    (Q = Y.current) == null || Q.update({
      onPaneContextMenu: t,
      zoomOnScroll: r,
      zoomOnPinch: i,
      panOnScroll: u,
      panOnScrollSpeed: l,
      panOnScrollMode: s,
      zoomOnDoubleClick: c,
      panOnDrag: f,
      zoomActivationKeyPressed: Z,
      preventScrolling: b,
      noPanClassName: w,
      userSelectionActive: q,
      noWheelClassName: A,
      lib: L,
      onTransformChange: J,
      connectionInProgress: j,
      selectionOnDrag: T,
      paneClickDistance: S
    });
  }, [
    t,
    r,
    i,
    u,
    l,
    s,
    c,
    f,
    Z,
    b,
    w,
    q,
    A,
    L,
    J,
    j,
    T,
    S
  ]), k.jsx("div", { className: "react-flow__renderer", ref: R, style: _s, children: _ });
}
const j8 = (t) => ({
  userSelectionActive: t.userSelectionActive,
  userSelectionRect: t.userSelectionRect
});
function B8() {
  const { userSelectionActive: t, userSelectionRect: r } = Oe(j8, $e);
  return t && r ? k.jsx("div", { className: "react-flow__selection react-flow__container", style: {
    width: r.width,
    height: r.height,
    transform: `translate(${r.x}px, ${r.y}px)`
  } }) : null;
}
const yy = (t, r) => (i) => {
  i.target === r.current && (t == null || t(i));
}, U8 = (t) => ({
  userSelectionActive: t.userSelectionActive,
  elementsSelectable: t.elementsSelectable,
  dragging: t.paneDragging,
  panBy: t.panBy,
  autoPanSpeed: t.autoPanSpeed
});
function G8({ isSelecting: t, selectionKeyPressed: r, selectionMode: i = Ou.Full, panOnDrag: u, autoPanOnSelection: l, paneClickDistance: s, selectionOnDrag: c, onSelectionStart: f, onSelectionEnd: g, onPaneClick: h, onPaneContextMenu: v, onPaneScroll: y, onPaneMouseEnter: m, onPaneMouseMove: b, onPaneMouseLeave: _, children: A }) {
  const w = re.useRef(0), E = Ze(), { userSelectionActive: M, elementsSelectable: S, dragging: T, panBy: N, autoPanSpeed: R } = Oe(U8, $e), q = S && (t || M), L = re.useRef(null), j = re.useRef(), Z = re.useRef(/* @__PURE__ */ new Set()), Y = re.useRef(/* @__PURE__ */ new Set()), J = re.useRef(!1), Q = re.useRef(!1), D = re.useRef({ x: 0, y: 0 }), X = re.useRef(!1), z = (W) => {
    if (Q.current || J.current || E.getState().connection.inProgress) {
      Q.current = !1, J.current = !1;
      return;
    }
    h == null || h(W), E.getState().resetSelectedElements(), E.setState({ nodesSelectionActive: !1 });
  }, B = (W) => {
    if (Array.isArray(u) && (u != null && u.includes(2))) {
      W.preventDefault();
      return;
    }
    v == null || v(W);
  }, $ = y ? (W) => y(W) : void 0, U = (W) => {
    Q.current && (W.stopPropagation(), Q.current = !1);
  }, ae = (W) => {
    var xe, Pe;
    const { domNode: se, transform: de } = E.getState();
    if (j.current = se == null ? void 0 : se.getBoundingClientRect(), !j.current)
      return;
    const ve = W.target === L.current;
    if (!ve && !!W.target.closest(".nokey") || !t || !(c && ve || r) || W.button !== 0 || !W.isPrimary)
      return;
    (Pe = (xe = W.target) == null ? void 0 : xe.setPointerCapture) == null || Pe.call(xe, W.pointerId), Q.current = !1;
    const { x: me, y: ge } = hn(W.nativeEvent, j.current), Ae = Qu({ x: me, y: ge }, de);
    E.setState({
      userSelectionRect: {
        width: 0,
        height: 0,
        startX: Ae.x,
        startY: Ae.y,
        x: me,
        y: ge
      }
    }), ve || (W.stopPropagation(), W.preventDefault());
  };
  function O(W, se) {
    const { userSelectionRect: de } = E.getState();
    if (!de)
      return;
    const { transform: ve, nodeLookup: ye, edgeLookup: he, connectionLookup: me, triggerNodeChanges: ge, triggerEdgeChanges: Ae, defaultEdgeOptions: xe } = E.getState(), Pe = { x: de.startX, y: de.startY }, { x: tt, y: xt } = di(Pe, ve), gt = {
      startX: Pe.x,
      startY: Pe.y,
      x: W < tt ? W : tt,
      y: se < xt ? se : xt,
      width: Math.abs(W - tt),
      height: Math.abs(se - xt)
    }, St = Z.current, Ke = Y.current;
    Z.current = new Set(om(ye, gt, ve, i === Ou.Partial, !0).map((vt) => vt.id)), Y.current = /* @__PURE__ */ new Set();
    const ke = (xe == null ? void 0 : xe.selectable) ?? !0;
    for (const vt of Z.current) {
      const qt = me.get(vt);
      if (qt)
        for (const { edgeId: Tt } of qt.values()) {
          const ft = he.get(Tt);
          ft && (ft.selectable ?? ke) && Y.current.add(Tt);
        }
    }
    if (!J2(St, Z.current)) {
      const vt = ii(ye, Z.current, !0);
      ge(vt);
    }
    if (!J2(Ke, Y.current)) {
      const vt = ii(he, Y.current);
      Ae(vt);
    }
    E.setState({
      userSelectionRect: gt,
      userSelectionActive: !0,
      nodesSelectionActive: !1
    });
  }
  function G() {
    if (!l || !j.current)
      return;
    const [W, se] = sm(D.current, j.current, R);
    N({ x: W, y: se }).then((de) => {
      if (!Q.current || !de) {
        w.current = requestAnimationFrame(G);
        return;
      }
      const { x: ve, y: ye } = D.current;
      O(ve, ye), w.current = requestAnimationFrame(G);
    });
  }
  const ie = () => {
    cancelAnimationFrame(w.current), w.current = 0, X.current = !1;
  };
  re.useEffect(() => () => ie(), []);
  const H = (W) => {
    const { userSelectionRect: se, transform: de, resetSelectedElements: ve } = E.getState();
    if (!j.current || !se)
      return;
    const { x: ye, y: he } = hn(W.nativeEvent, j.current);
    D.current = { x: ye, y: he };
    const me = di({ x: se.startX, y: se.startY }, de);
    if (!Q.current) {
      const ge = r ? 0 : s;
      if (Math.hypot(ye - me.x, he - me.y) <= ge)
        return;
      ve(), f == null || f(W);
    }
    Q.current = !0, X.current || (G(), X.current = !0), O(ye, he);
  }, K = (W) => {
    var se, de;
    if (!q) {
      W.target === L.current && E.getState().connection.inProgress && (J.current = !0);
      return;
    }
    W.button === 0 && ((de = (se = W.target) == null ? void 0 : se.releasePointerCapture) == null || de.call(se, W.pointerId), !M && W.target === L.current && E.getState().userSelectionRect && (z == null || z(W)), E.setState({
      userSelectionActive: !1,
      userSelectionRect: null
    }), Q.current && (g == null || g(W), E.setState({
      nodesSelectionActive: Z.current.size > 0
    })), ie());
  }, P = (W) => {
    var se, de;
    (de = (se = W.target) == null ? void 0 : se.releasePointerCapture) == null || de.call(se, W.pointerId), ie();
  }, ue = u === !0 || Array.isArray(u) && u.includes(0);
  return k.jsxs("div", { className: ct(["react-flow__pane", { draggable: ue, dragging: T, selection: t }]), onClick: q ? void 0 : yy(z, L), onContextMenu: yy(B, L), onWheel: yy($, L), onPointerEnter: q ? void 0 : m, onPointerMove: q ? H : b, onPointerUp: K, onPointerCancel: q ? P : void 0, onPointerDownCapture: q ? ae : void 0, onClickCapture: q ? U : void 0, onPointerLeave: _, ref: L, style: _s, children: [A, k.jsx(B8, {})] });
}
function zy({ id: t, store: r, unselect: i = !1, nodeRef: u }) {
  const { addSelectedNodes: l, unselectNodesAndEdges: s, multiSelectionActive: c, nodeLookup: f, onError: g } = r.getState(), h = f.get(t);
  if (!h) {
    g == null || g("012", vn.error012(t));
    return;
  }
  r.setState({ nodesSelectionActive: !1 }), h.selected ? (i || h.selected && c) && (s({ nodes: [h], edges: [] }), requestAnimationFrame(() => {
    var v;
    return (v = u == null ? void 0 : u.current) == null ? void 0 : v.blur();
  })) : l([t]);
}
function aC({ nodeRef: t, disabled: r = !1, noDragClassName: i, handleSelector: u, nodeId: l, isSelectable: s, nodeClickDistance: c }) {
  const f = Ze(), [g, h] = re.useState(!1), v = re.useRef();
  return re.useEffect(() => {
    if (!r)
      return v.current = A6({
        getStoreItems: () => f.getState(),
        onNodeMouseDown: (y) => {
          zy({
            id: y,
            store: f,
            nodeRef: t
          });
        },
        onDragStart: () => {
          h(!0);
        },
        onDragStop: () => {
          h(!1);
        }
      }), () => {
        var y;
        (y = v.current) == null || y.destroy(), v.current = void 0;
      };
  }, [r, f, t]), re.useEffect(() => {
    r || !t.current || !v.current || v.current.update({
      noDragClassName: i,
      handleSelector: u,
      domNode: t.current,
      isSelectable: s,
      nodeId: l,
      nodeClickDistance: c
    });
  }, [i, u, r, s, t, l, c]), g;
}
const V8 = (t) => (r) => r.selected && (r.draggable || t && typeof r.draggable > "u");
function iC() {
  const t = Ze();
  return re.useCallback((i) => {
    const { nodeExtent: u, snapToGrid: l, snapGrid: s, nodesDraggable: c, onError: f, updateNodePositions: g, nodeLookup: h, nodeOrigin: v } = t.getState(), y = /* @__PURE__ */ new Map(), m = V8(c), b = l ? s[0] : 5, _ = l ? s[1] : 5, A = i.direction.x * b * i.factor, w = i.direction.y * _ * i.factor;
    for (const [, E] of h) {
      if (!m(E))
        continue;
      let M = {
        x: E.internals.positionAbsolute.x + A,
        y: E.internals.positionAbsolute.y + w
      };
      l && (M = Ku(M, s));
      const { position: S, positionAbsolute: T } = wM({
        nodeId: E.id,
        nextPosition: M,
        nodeLookup: h,
        nodeExtent: u,
        nodeOrigin: v,
        onError: f
      });
      E.position = S, E.internals.positionAbsolute = T, y.set(E.id, E);
    }
    g(y);
  }, []);
}
const ym = re.createContext(null), Y8 = ym.Provider;
ym.Consumer;
const uC = () => re.useContext(ym), k8 = (t) => ({
  connectOnClick: t.connectOnClick,
  noPanClassName: t.noPanClassName,
  rfId: t.rfId
}), lC = re.createContext(null);
function X8({ children: t }) {
  const r = Oe(k8, $e);
  return k.jsx(lC.Provider, { value: r, children: t });
}
function I8() {
  const t = re.useContext(lC);
  if (!t)
    throw new Error("useHandleConfig must be used within a HandleConfigProvider");
  return t;
}
const Z8 = {
  connectingFrom: !1,
  connectingTo: !1,
  clickConnecting: !1,
  isPossibleEndHandle: !0,
  connectionInProcess: !1,
  clickConnectionInProcess: !1,
  valid: !1
}, K8 = (t, r, i) => (u) => {
  const { connectionClickStartHandle: l, connectionMode: s, connection: c } = u, { fromHandle: f, toHandle: g, isValid: h } = c;
  if (!f && !l)
    return Z8;
  const v = (g == null ? void 0 : g.nodeId) === t && (g == null ? void 0 : g.id) === r && (g == null ? void 0 : g.type) === i;
  return {
    connectingFrom: (f == null ? void 0 : f.nodeId) === t && (f == null ? void 0 : f.id) === r && (f == null ? void 0 : f.type) === i,
    connectingTo: v,
    clickConnecting: (l == null ? void 0 : l.nodeId) === t && (l == null ? void 0 : l.id) === r && (l == null ? void 0 : l.type) === i,
    isPossibleEndHandle: s === ci.Strict ? (f == null ? void 0 : f.type) !== i : t !== (f == null ? void 0 : f.nodeId) || r !== (f == null ? void 0 : f.id),
    connectionInProcess: !!f,
    clickConnectionInProcess: !!l,
    valid: v && h
  };
};
function Q8({ type: t = "source", position: r = _e.Top, isValidConnection: i, isConnectable: u = !0, isConnectableStart: l = !0, isConnectableEnd: s = !0, id: c, onConnect: f, children: g, className: h, onMouseDown: v, onTouchStart: y, ...m }, b) {
  var X, z;
  const _ = c || null, A = t === "target", w = Ze(), E = uC(), { connectOnClick: M, noPanClassName: S, rfId: T } = I8(), { connectingFrom: N, connectingTo: R, clickConnecting: q, isPossibleEndHandle: L, connectionInProcess: j, clickConnectionInProcess: Z, valid: Y } = Oe(K8(E, _, t), $e);
  E || (z = (X = w.getState()).onError) == null || z.call(X, "010", vn.error010());
  const J = (B) => {
    const { defaultEdgeOptions: $, onConnect: U, hasDefaultEdges: ae } = w.getState(), O = {
      ...$,
      ...B
    };
    if (ae) {
      const { edges: G, setEdges: ie, onError: H } = w.getState();
      ie(T8(O, G, { onError: H }));
    }
    U == null || U(O), f == null || f(O);
  }, Q = (B) => {
    if (!E)
      return;
    const $ = OM(B.nativeEvent);
    if (l && ($ && B.button === 0 || !$)) {
      const U = w.getState();
      Ny.onPointerDown(B.nativeEvent, {
        handleDomNode: B.currentTarget,
        autoPanOnConnect: U.autoPanOnConnect,
        connectionMode: U.connectionMode,
        connectionRadius: U.connectionRadius,
        domNode: U.domNode,
        nodeLookup: U.nodeLookup,
        lib: U.lib,
        isTarget: A,
        handleId: _,
        nodeId: E,
        flowId: U.rfId,
        panBy: U.panBy,
        cancelConnection: U.cancelConnection,
        onConnectStart: U.onConnectStart,
        onConnectEnd: (...ae) => {
          var O, G;
          return (G = (O = w.getState()).onConnectEnd) == null ? void 0 : G.call(O, ...ae);
        },
        updateConnection: U.updateConnection,
        onConnect: J,
        isValidConnection: i || ((...ae) => {
          var O, G;
          return ((G = (O = w.getState()).isValidConnection) == null ? void 0 : G.call(O, ...ae)) ?? !0;
        }),
        getTransform: () => w.getState().transform,
        getFromHandle: () => w.getState().connection.fromHandle,
        autoPanSpeed: U.autoPanSpeed,
        dragThreshold: U.connectionDragThreshold
      });
    }
    $ ? v == null || v(B) : y == null || y(B);
  }, D = (B) => {
    const { onClickConnectStart: $, onClickConnectEnd: U, connectionClickStartHandle: ae, connectionMode: O, isValidConnection: G, lib: ie, rfId: H, nodeLookup: K, connection: P } = w.getState();
    if (!E || !ae && !l)
      return;
    if (!ae) {
      $ == null || $(B.nativeEvent, { nodeId: E, handleId: _, handleType: t }), w.setState({ connectionClickStartHandle: { nodeId: E, type: t, id: _ } });
      return;
    }
    const ue = NM(B.target), W = i || G, { connection: se, isValid: de } = Ny.isValid(B.nativeEvent, {
      handle: {
        nodeId: E,
        id: _,
        type: t
      },
      connectionMode: O,
      fromNodeId: ae.nodeId,
      fromHandleId: ae.id || null,
      fromType: ae.type,
      isValidConnection: W,
      flowId: H,
      doc: ue,
      lib: ie,
      nodeLookup: K
    });
    de && se && J(se);
    const ve = structuredClone(P);
    delete ve.inProgress, ve.toPosition = ve.toHandle ? ve.toHandle.position : null, U == null || U(B, ve), w.setState({ connectionClickStartHandle: null });
  };
  return k.jsx("div", { "data-handleid": _, "data-nodeid": E, "data-handlepos": r, "data-id": `${T}-${E}-${_}-${t}`, className: ct([
    "react-flow__handle",
    `react-flow__handle-${r}`,
    "nodrag",
    S,
    h,
    {
      source: !A,
      target: A,
      connectable: u,
      connectablestart: l,
      connectableend: s,
      clickconnecting: q,
      connectingfrom: N,
      connectingto: R,
      valid: Y,
      /*
       * shows where you can start a connection from
       * and where you can end it while connecting
       */
      connectionindicator: u && (!j || L) && (j || Z ? s : l)
    }
  ]), onMouseDown: Q, onTouchStart: Q, onClick: M ? D : void 0, ref: b, ...m, children: g });
}
const gi = re.memo(tC(Q8));
function $8({ data: t, isConnectable: r, sourcePosition: i = _e.Bottom }) {
  return k.jsxs(k.Fragment, { children: [t == null ? void 0 : t.label, k.jsx(gi, { type: "source", position: i, isConnectable: r })] });
}
function F8({ data: t, isConnectable: r, targetPosition: i = _e.Top, sourcePosition: u = _e.Bottom }) {
  return k.jsxs(k.Fragment, { children: [k.jsx(gi, { type: "target", position: i, isConnectable: r }), t == null ? void 0 : t.label, k.jsx(gi, { type: "source", position: u, isConnectable: r })] });
}
function J8() {
  return null;
}
function P8({ data: t, isConnectable: r, targetPosition: i = _e.Top }) {
  return k.jsxs(k.Fragment, { children: [k.jsx(gi, { type: "target", position: i, isConnectable: r }), t == null ? void 0 : t.label] });
}
const Zo = {
  ArrowUp: { x: 0, y: -1 },
  ArrowDown: { x: 0, y: 1 },
  ArrowLeft: { x: -1, y: 0 },
  ArrowRight: { x: 1, y: 0 }
}, CA = {
  input: $8,
  default: F8,
  output: P8,
  group: J8
};
function W8(t) {
  var r, i, u, l;
  return t.internals.handleBounds === void 0 ? {
    width: t.width ?? t.initialWidth ?? ((r = t.style) == null ? void 0 : r.width),
    height: t.height ?? t.initialHeight ?? ((i = t.style) == null ? void 0 : i.height)
  } : {
    width: t.width ?? ((u = t.style) == null ? void 0 : u.width),
    height: t.height ?? ((l = t.style) == null ? void 0 : l.height)
  };
}
const eL = (t) => {
  const { width: r, height: i, x: u, y: l } = Zu(t.nodeLookup, {
    filter: (s) => !!s.selected
  });
  return {
    width: dn(r) ? r : null,
    height: dn(i) ? i : null,
    userSelectionActive: t.userSelectionActive,
    transformString: `translate(${t.transform[0]}px,${t.transform[1]}px) scale(${t.transform[2]}) translate(${u}px,${l}px)`
  };
};
function tL({ onSelectionContextMenu: t, noPanClassName: r, disableKeyboardA11y: i }) {
  const u = Ze(), { width: l, height: s, transformString: c, userSelectionActive: f } = Oe(eL, $e), g = iC(), h = re.useRef(null);
  re.useEffect(() => {
    var b;
    i || (b = h.current) == null || b.focus({
      preventScroll: !0
    });
  }, [i]);
  const v = !f && l !== null && s !== null;
  if (aC({
    nodeRef: h,
    disabled: !v
  }), !v)
    return null;
  const y = t ? (b) => {
    const _ = u.getState().nodes.filter((A) => A.selected);
    t(b, _);
  } : void 0, m = (b) => {
    Object.prototype.hasOwnProperty.call(Zo, b.key) && (b.preventDefault(), g({
      direction: Zo[b.key],
      factor: b.shiftKey ? 4 : 1
    }));
  };
  return k.jsx("div", { className: ct(["react-flow__nodesselection", "react-flow__container", r]), style: {
    transform: c
  }, children: k.jsx("div", { ref: h, className: "react-flow__nodesselection-rect", onContextMenu: y, tabIndex: i ? void 0 : -1, onKeyDown: i ? void 0 : m, style: {
    width: l,
    height: s
  } }) });
}
const qA = typeof window < "u" ? window : void 0, nL = (t) => ({ nodesSelectionActive: t.nodesSelectionActive, userSelectionActive: t.userSelectionActive });
function oC({ children: t, onPaneClick: r, onPaneMouseEnter: i, onPaneMouseMove: u, onPaneMouseLeave: l, onPaneContextMenu: s, onPaneScroll: c, paneClickDistance: f, deleteKeyCode: g, selectionKeyCode: h, selectionOnDrag: v, selectionMode: y, onSelectionStart: m, onSelectionEnd: b, multiSelectionKeyCode: _, panActivationKeyCode: A, zoomActivationKeyCode: w, elementsSelectable: E, zoomOnScroll: M, zoomOnPinch: S, panOnScroll: T, panOnScrollSpeed: N, panOnScrollMode: R, zoomOnDoubleClick: q, panOnDrag: L, autoPanOnSelection: j, defaultViewport: Z, translateExtent: Y, minZoom: J, maxZoom: Q, preventScrolling: D, onSelectionContextMenu: X, noWheelClassName: z, noPanClassName: B, disableKeyboardA11y: $, onViewportChange: U, isControlledViewport: ae }) {
  const { nodesSelectionActive: O, userSelectionActive: G } = Oe(nL, $e), ie = Hu(h, { target: qA }), H = Hu(A, { target: qA }), K = H || L, P = H || T, ue = v && K !== !0, W = ie || G || ue;
  return O8({ deleteKeyCode: g, multiSelectionKeyCode: _ }), k.jsx(H8, { onPaneContextMenu: s, elementsSelectable: E, zoomOnScroll: M, zoomOnPinch: S, panOnScroll: P, panOnScrollSpeed: N, panOnScrollMode: R, zoomOnDoubleClick: q, panOnDrag: !ie && K, defaultViewport: Z, translateExtent: Y, minZoom: J, maxZoom: Q, zoomActivationKeyCode: w, preventScrolling: D, noWheelClassName: z, noPanClassName: B, onViewportChange: U, isControlledViewport: ae, paneClickDistance: f, selectionOnDrag: ue, children: k.jsxs(G8, { onSelectionStart: m, onSelectionEnd: b, onPaneClick: r, onPaneMouseEnter: i, onPaneMouseMove: u, onPaneMouseLeave: l, onPaneContextMenu: s, onPaneScroll: c, panOnDrag: K, autoPanOnSelection: j, isSelecting: !!W, selectionMode: y, selectionKeyPressed: ie, paneClickDistance: f, selectionOnDrag: ue, children: [t, O && k.jsx(tL, { onSelectionContextMenu: X, noPanClassName: B, disableKeyboardA11y: $ })] }) });
}
oC.displayName = "FlowRenderer";
const rL = re.memo(oC), aL = (t) => (r) => t ? om(r.nodeLookup, { x: 0, y: 0, width: r.width, height: r.height }, r.transform, !0).map((i) => i.id) : Array.from(r.nodeLookup.keys());
function iL(t) {
  return Oe(re.useCallback(aL(t), [t]), $e);
}
const uL = (t) => t.updateNodeInternals;
function lL() {
  const t = Oe(uL), [r] = re.useState(() => typeof ResizeObserver > "u" ? null : new ResizeObserver((i) => {
    const u = /* @__PURE__ */ new Map();
    i.forEach((l) => {
      const s = l.target.getAttribute("data-id");
      u.set(s, {
        id: s,
        nodeElement: l.target,
        force: !0
      });
    }), t(u);
  }));
  return re.useEffect(() => () => {
    r == null || r.disconnect();
  }, [r]), r;
}
function oL({ node: t, nodeType: r, hasDimensions: i, resizeObserver: u }) {
  const l = Ze(), s = re.useRef(null), c = re.useRef(null), f = re.useRef(t.sourcePosition), g = re.useRef(t.targetPosition), h = re.useRef(r), v = i && !!t.internals.handleBounds;
  return re.useEffect(() => {
    s.current && !t.hidden && (!v || c.current !== s.current) && (c.current && (u == null || u.unobserve(c.current)), u == null || u.observe(s.current), c.current = s.current);
  }, [v, t.hidden]), re.useEffect(() => () => {
    c.current && (u == null || u.unobserve(c.current), c.current = null);
  }, []), re.useEffect(() => {
    if (s.current) {
      const y = h.current !== r, m = f.current !== t.sourcePosition, b = g.current !== t.targetPosition;
      (y || m || b) && (h.current = r, f.current = t.sourcePosition, g.current = t.targetPosition, l.getState().updateNodeInternals(/* @__PURE__ */ new Map([[t.id, { id: t.id, nodeElement: s.current, force: !0 }]])));
    }
  }, [t.id, r, t.sourcePosition, t.targetPosition]), s;
}
function sL({ id: t, onClick: r, onMouseEnter: i, onMouseMove: u, onMouseLeave: l, onContextMenu: s, onDoubleClick: c, nodesDraggable: f, elementsSelectable: g, nodesConnectable: h, nodesFocusable: v, resizeObserver: y, noDragClassName: m, noPanClassName: b, disableKeyboardA11y: _, rfId: A, nodeTypes: w, nodeClickDistance: E, onError: M }) {
  const { node: S, internals: T, isParent: N } = Oe((W) => {
    const se = W.nodeLookup.get(t), de = W.parentLookup.has(t);
    return {
      node: se,
      internals: se.internals,
      isParent: de
    };
  }, $e);
  let R = S.type || "default", q = (w == null ? void 0 : w[R]) || CA[R];
  q === void 0 && (M == null || M("003", vn.error003(R)), R = "default", q = (w == null ? void 0 : w.default) || CA.default);
  const L = !!(S.draggable || f && typeof S.draggable > "u"), j = !!(S.selectable || g && typeof S.selectable > "u"), Z = !!(S.connectable || h && typeof S.connectable > "u"), Y = !!(S.focusable || v && typeof S.focusable > "u"), J = Ze(), Q = qM(S), D = oL({ node: S, nodeType: R, hasDimensions: Q, resizeObserver: y }), X = aC({
    nodeRef: D,
    disabled: S.hidden || !L,
    noDragClassName: m,
    handleSelector: S.dragHandle,
    nodeId: t,
    isSelectable: j,
    nodeClickDistance: E
  }), z = iC();
  if (S.hidden)
    return null;
  const B = Cn(S), $ = W8(S), U = j || L || r || i || u || l, ae = i ? (W) => i(W, { ...T.userNode }) : void 0, O = u ? (W) => u(W, { ...T.userNode }) : void 0, G = l ? (W) => l(W, { ...T.userNode }) : void 0, ie = s ? (W) => s(W, { ...T.userNode }) : void 0, H = c ? (W) => c(W, { ...T.userNode }) : void 0, K = (W) => {
    const { selectNodesOnDrag: se, nodeDragThreshold: de } = J.getState();
    j && (!se || !L || de > 0) && zy({
      id: t,
      store: J,
      nodeRef: D
    }), r && r(W, { ...T.userNode });
  }, P = (W) => {
    if (!(zM(W.nativeEvent) || _)) {
      if (bM.includes(W.key) && j) {
        const se = W.key === "Escape";
        zy({
          id: t,
          store: J,
          unselect: se,
          nodeRef: D
        });
      } else if (L && S.selected && Object.prototype.hasOwnProperty.call(Zo, W.key)) {
        W.preventDefault();
        const { ariaLabelConfig: se } = J.getState();
        J.setState({
          ariaLiveMessage: se["node.a11yDescription.ariaLiveMessage"]({
            direction: W.key.replace("Arrow", "").toLowerCase(),
            x: ~~T.positionAbsolute.x,
            y: ~~T.positionAbsolute.y
          })
        }), z({
          direction: Zo[W.key],
          factor: W.shiftKey ? 4 : 1
        });
      }
    }
  }, ue = () => {
    var me;
    if (_ || !((me = D.current) != null && me.matches(":focus-visible")))
      return;
    const { transform: W, width: se, height: de, autoPanOnNodeFocus: ve, setCenter: ye } = J.getState();
    if (!ve)
      return;
    om(/* @__PURE__ */ new Map([[t, S]]), { x: 0, y: 0, width: se, height: de }, W, !0).length > 0 || ye(S.position.x + B.width / 2, S.position.y + B.height / 2, {
      zoom: W[2]
    });
  };
  return k.jsx("div", { className: ct([
    "react-flow__node",
    `react-flow__node-${R}`,
    {
      // this is overwritable by passing `nopan` as a class name
      [b]: L
    },
    S.className,
    {
      selected: S.selected,
      selectable: j,
      parent: N,
      draggable: L,
      dragging: X
    }
  ]), ref: D, style: {
    zIndex: T.z,
    transform: `translate(${T.positionAbsolute.x}px,${T.positionAbsolute.y}px)`,
    pointerEvents: U ? "all" : "none",
    visibility: Q ? "visible" : "hidden",
    ...S.style,
    ...$
  }, "data-id": t, "data-testid": `rf__node-${t}`, onMouseEnter: ae, onMouseMove: O, onMouseLeave: G, onContextMenu: ie, onClick: K, onDoubleClick: H, onKeyDown: Y ? P : void 0, tabIndex: Y ? 0 : void 0, onFocus: Y ? ue : void 0, role: S.ariaRole ?? (Y ? "group" : void 0), "aria-roledescription": "node", "aria-describedby": _ ? void 0 : `${JM}-${A}`, "aria-label": S.ariaLabel, ...S.domAttributes, children: k.jsx(Y8, { value: t, children: k.jsx(q, { id: t, data: S.data, type: R, positionAbsoluteX: T.positionAbsolute.x, positionAbsoluteY: T.positionAbsolute.y, selected: S.selected ?? !1, selectable: j, draggable: L, deletable: S.deletable ?? !0, isConnectable: Z, sourcePosition: S.sourcePosition, targetPosition: S.targetPosition, dragging: X, dragHandle: S.dragHandle, zIndex: T.z, parentId: S.parentId, ...B }) }) });
}
var cL = re.memo(sL);
const fL = (t) => ({
  nodesConnectable: t.nodesConnectable,
  nodesFocusable: t.nodesFocusable,
  elementsSelectable: t.elementsSelectable,
  onError: t.onError
});
function sC(t) {
  const { nodesConnectable: r, nodesFocusable: i, elementsSelectable: u, onError: l } = Oe(fL, $e), s = iL(t.onlyRenderVisibleElements), c = lL();
  return k.jsx("div", { className: "react-flow__nodes", style: _s, children: s.map((f) => (
    /*
     * The split of responsibilities between NodeRenderer and
     * NodeComponentWrapper may appear weird. However, it’s designed to
     * minimize the cost of updates when individual nodes change.
     *
     * For example, when you’re dragging a single node, that node gets
     * updated multiple times per second. If `NodeRenderer` were to update
     * every time, it would have to re-run the `nodes.map()` loop every
     * time. This gets pricey with hundreds of nodes, especially if every
     * loop cycle does more than just rendering a JSX element!
     *
     * As a result of this choice, we took the following implementation
     * decisions:
     * - NodeRenderer subscribes *only* to node IDs – and therefore
     *   rerender *only* when visible nodes are added or removed.
     * - NodeRenderer performs all operations the result of which can be
     *   shared between nodes (such as creating the `ResizeObserver`
     *   instance, or subscribing to `selector`). This means extra prop
     *   drilling into `NodeComponentWrapper`, but it means we need to run
     *   these operations only once – instead of once per node.
     * - Any operations that you’d normally write inside `nodes.map` are
     *   moved into `NodeComponentWrapper`. This ensures they are
     *   memorized – so if `NodeRenderer` *has* to rerender, it only
     *   needs to regenerate the list of nodes, nothing else.
     */
    k.jsx(cL, { id: f, nodeTypes: t.nodeTypes, nodeExtent: t.nodeExtent, onClick: t.onNodeClick, onMouseEnter: t.onNodeMouseEnter, onMouseMove: t.onNodeMouseMove, onMouseLeave: t.onNodeMouseLeave, onContextMenu: t.onNodeContextMenu, onDoubleClick: t.onNodeDoubleClick, noDragClassName: t.noDragClassName, noPanClassName: t.noPanClassName, rfId: t.rfId, disableKeyboardA11y: t.disableKeyboardA11y, resizeObserver: c, nodesDraggable: t.nodesDraggable ?? !0, nodesConnectable: r, nodesFocusable: i, elementsSelectable: u, nodeClickDistance: t.nodeClickDistance, onError: l }, f)
  )) });
}
sC.displayName = "NodeRenderer";
const dL = re.memo(sC);
function hL(t) {
  return Oe(re.useCallback((i) => {
    if (!t)
      return i.edges.map((l) => l.id);
    const u = [];
    if (i.width && i.height)
      for (const l of i.edges) {
        const s = i.nodeLookup.get(l.source), c = i.nodeLookup.get(l.target);
        s && c && u6({
          sourceNode: s,
          targetNode: c,
          width: i.width,
          height: i.height,
          transform: i.transform
        }) && u.push(l.id);
      }
    return u;
  }, [t]), $e);
}
const gL = ({ color: t = "none", strokeWidth: r = 1 }) => {
  const i = {
    strokeWidth: r,
    ...t && { stroke: t }
  };
  return k.jsx("polyline", { className: "arrow", style: i, strokeLinecap: "round", fill: "none", strokeLinejoin: "round", points: "-5,-4 0,0 -5,4" });
}, vL = ({ color: t = "none", strokeWidth: r = 1 }) => {
  const i = {
    strokeWidth: r,
    ...t && { stroke: t, fill: t }
  };
  return k.jsx("polyline", { className: "arrowclosed", style: i, strokeLinecap: "round", strokeLinejoin: "round", points: "-5,-4 0,0 -5,4 -5,-4" });
}, RA = {
  [ko.Arrow]: gL,
  [ko.ArrowClosed]: vL
};
function pL(t) {
  const r = Ze();
  return re.useMemo(() => {
    var l, s;
    return Object.prototype.hasOwnProperty.call(RA, t) ? RA[t] : ((s = (l = r.getState()).onError) == null || s.call(l, "009", vn.error009(t)), null);
  }, [t]);
}
const yL = ({ id: t, type: r, color: i, width: u = 12.5, height: l = 12.5, markerUnits: s = "strokeWidth", strokeWidth: c, orient: f = "auto-start-reverse" }) => {
  const g = pL(r);
  return g ? k.jsx("marker", { className: "react-flow__arrowhead", id: t, markerWidth: `${u}`, markerHeight: `${l}`, viewBox: "-10 -10 20 20", markerUnits: s, orient: f, refX: "0", refY: "0", children: k.jsx(g, { color: i, strokeWidth: c }) }) : null;
}, cC = ({ defaultColor: t, rfId: r }) => {
  const i = Oe((s) => s.edges), u = Oe((s) => s.defaultEdgeOptions), l = re.useMemo(() => g6(i, {
    id: r,
    defaultColor: t,
    defaultMarkerStart: u == null ? void 0 : u.markerStart,
    defaultMarkerEnd: u == null ? void 0 : u.markerEnd
  }), [i, u, r, t]);
  return l.length ? k.jsx("svg", { className: "react-flow__marker", "aria-hidden": "true", children: k.jsx("defs", { children: l.map((s) => k.jsx(yL, { id: s.id, type: s.type, color: s.color, width: s.width, height: s.height, markerUnits: s.markerUnits, strokeWidth: s.strokeWidth, orient: s.orient }, s.id)) }) }) : null;
};
cC.displayName = "MarkerDefinitions";
var mL = re.memo(cC);
function fC({ x: t, y: r, label: i, labelStyle: u, labelShowBg: l = !0, labelBgStyle: s, labelBgPadding: c = [2, 4], labelBgBorderRadius: f = 2, children: g, className: h, ...v }) {
  const [y, m] = re.useState({ x: 1, y: 0, width: 0, height: 0 }), b = ct(["react-flow__edge-textwrapper", h]), _ = re.useRef(null);
  return re.useEffect(() => {
    if (_.current) {
      const A = _.current.getBBox();
      m({
        x: A.x,
        y: A.y,
        width: A.width,
        height: A.height
      });
    }
  }, [i]), i ? k.jsxs("g", { transform: `translate(${t - y.width / 2} ${r - y.height / 2})`, className: b, visibility: y.width ? "visible" : "hidden", ...v, children: [l && k.jsx("rect", { width: y.width + 2 * c[0], x: -c[0], y: -c[1], height: y.height + 2 * c[1], className: "react-flow__edge-textbg", style: s, rx: f, ry: f }), k.jsx("text", { className: "react-flow__edge-text", y: y.height / 2, dy: "0.3em", ref: _, style: u, children: i }), g] }) : null;
}
fC.displayName = "EdgeText";
const bL = re.memo(fC);
function xs({ path: t, labelX: r, labelY: i, label: u, labelStyle: l, labelShowBg: s, labelBgStyle: c, labelBgPadding: f, labelBgBorderRadius: g, interactionWidth: h = 20, ...v }) {
  return k.jsxs(k.Fragment, { children: [k.jsx("path", { ...v, d: t, fill: "none", className: ct(["react-flow__edge-path", v.className]) }), h ? k.jsx("path", { d: t, fill: "none", strokeOpacity: 0, strokeWidth: h, className: "react-flow__edge-interaction" }) : null, u && dn(r) && dn(i) ? k.jsx(bL, { x: r, y: i, label: u, labelStyle: l, labelShowBg: s, labelBgStyle: c, labelBgPadding: f, labelBgBorderRadius: g }) : null] });
}
function NA({ pos: t, x1: r, y1: i, x2: u, y2: l }) {
  return t === _e.Left || t === _e.Right ? [0.5 * (r + u), i] : [r, 0.5 * (i + l)];
}
function dC({ sourceX: t, sourceY: r, sourcePosition: i = _e.Bottom, targetX: u, targetY: l, targetPosition: s = _e.Top }) {
  const [c, f] = NA({
    pos: i,
    x1: t,
    y1: r,
    x2: u,
    y2: l
  }), [g, h] = NA({
    pos: s,
    x1: u,
    y1: l,
    x2: t,
    y2: r
  }), [v, y, m, b] = DM({
    sourceX: t,
    sourceY: r,
    targetX: u,
    targetY: l,
    sourceControlX: c,
    sourceControlY: f,
    targetControlX: g,
    targetControlY: h
  });
  return [
    `M${t},${r} C${c},${f} ${g},${h} ${u},${l}`,
    v,
    y,
    m,
    b
  ];
}
function hC(t) {
  return re.memo(({ id: r, sourceX: i, sourceY: u, targetX: l, targetY: s, sourcePosition: c, targetPosition: f, label: g, labelStyle: h, labelShowBg: v, labelBgStyle: y, labelBgPadding: m, labelBgBorderRadius: b, style: _, markerEnd: A, markerStart: w, interactionWidth: E }) => {
    const [M, S, T] = dC({
      sourceX: i,
      sourceY: u,
      sourcePosition: c,
      targetX: l,
      targetY: s,
      targetPosition: f
    }), N = t.isInternal ? void 0 : r;
    return k.jsx(xs, { id: N, path: M, labelX: S, labelY: T, label: g, labelStyle: h, labelShowBg: v, labelBgStyle: y, labelBgPadding: m, labelBgBorderRadius: b, style: _, markerEnd: A, markerStart: w, interactionWidth: E });
  });
}
const _L = hC({ isInternal: !1 }), gC = hC({ isInternal: !0 });
_L.displayName = "SimpleBezierEdge";
gC.displayName = "SimpleBezierEdgeInternal";
function vC(t) {
  return re.memo(({ id: r, sourceX: i, sourceY: u, targetX: l, targetY: s, label: c, labelStyle: f, labelShowBg: g, labelBgStyle: h, labelBgPadding: v, labelBgBorderRadius: y, style: m, sourcePosition: b = _e.Bottom, targetPosition: _ = _e.Top, markerEnd: A, markerStart: w, pathOptions: E, interactionWidth: M }) => {
    const [S, T, N] = Cy({
      sourceX: i,
      sourceY: u,
      sourcePosition: b,
      targetX: l,
      targetY: s,
      targetPosition: _,
      borderRadius: E == null ? void 0 : E.borderRadius,
      offset: E == null ? void 0 : E.offset,
      stepPosition: E == null ? void 0 : E.stepPosition
    }), R = t.isInternal ? void 0 : r;
    return k.jsx(xs, { id: R, path: S, labelX: T, labelY: N, label: c, labelStyle: f, labelShowBg: g, labelBgStyle: h, labelBgPadding: v, labelBgBorderRadius: y, style: m, markerEnd: A, markerStart: w, interactionWidth: M });
  });
}
const pC = vC({ isInternal: !1 }), yC = vC({ isInternal: !0 });
pC.displayName = "SmoothStepEdge";
yC.displayName = "SmoothStepEdgeInternal";
function mC(t) {
  return re.memo(({ id: r, ...i }) => {
    var l;
    const u = t.isInternal ? void 0 : r;
    return k.jsx(pC, { ...i, id: u, pathOptions: re.useMemo(() => {
      var s;
      return { borderRadius: 0, offset: (s = i.pathOptions) == null ? void 0 : s.offset };
    }, [(l = i.pathOptions) == null ? void 0 : l.offset]) });
  });
}
const xL = mC({ isInternal: !1 }), bC = mC({ isInternal: !0 });
xL.displayName = "StepEdge";
bC.displayName = "StepEdgeInternal";
function _C(t) {
  return re.memo(({ id: r, sourceX: i, sourceY: u, targetX: l, targetY: s, label: c, labelStyle: f, labelShowBg: g, labelBgStyle: h, labelBgPadding: v, labelBgBorderRadius: y, style: m, markerEnd: b, markerStart: _, interactionWidth: A }) => {
    const [w, E, M] = jM({ sourceX: i, sourceY: u, targetX: l, targetY: s }), S = t.isInternal ? void 0 : r;
    return k.jsx(xs, { id: S, path: w, labelX: E, labelY: M, label: c, labelStyle: f, labelShowBg: g, labelBgStyle: h, labelBgPadding: v, labelBgBorderRadius: y, style: m, markerEnd: b, markerStart: _, interactionWidth: A });
  });
}
const SL = _C({ isInternal: !1 }), xC = _C({ isInternal: !0 });
SL.displayName = "StraightEdge";
xC.displayName = "StraightEdgeInternal";
function SC(t) {
  return re.memo(({ id: r, sourceX: i, sourceY: u, targetX: l, targetY: s, sourcePosition: c = _e.Bottom, targetPosition: f = _e.Top, label: g, labelStyle: h, labelShowBg: v, labelBgStyle: y, labelBgPadding: m, labelBgBorderRadius: b, style: _, markerEnd: A, markerStart: w, pathOptions: E, interactionWidth: M }) => {
    const [S, T, N] = LM({
      sourceX: i,
      sourceY: u,
      sourcePosition: c,
      targetX: l,
      targetY: s,
      targetPosition: f,
      curvature: E == null ? void 0 : E.curvature
    }), R = t.isInternal ? void 0 : r;
    return k.jsx(xs, { id: R, path: S, labelX: T, labelY: N, label: g, labelStyle: h, labelShowBg: v, labelBgStyle: y, labelBgPadding: m, labelBgBorderRadius: b, style: _, markerEnd: A, markerStart: w, interactionWidth: M });
  });
}
const EL = SC({ isInternal: !1 }), EC = SC({ isInternal: !0 });
EL.displayName = "BezierEdge";
EC.displayName = "BezierEdgeInternal";
const zA = {
  default: EC,
  straight: xC,
  step: bC,
  smoothstep: yC,
  simplebezier: gC
}, OA = {
  sourceX: null,
  sourceY: null,
  targetX: null,
  targetY: null,
  sourcePosition: null,
  targetPosition: null,
  zIndex: void 0
}, wL = (t, r, i) => i === _e.Left ? t - r : i === _e.Right ? t + r : t, AL = (t, r, i) => i === _e.Top ? t - r : i === _e.Bottom ? t + r : t, DA = "react-flow__edgeupdater";
function LA({ position: t, centerX: r, centerY: i, radius: u = 10, onMouseDown: l, onMouseEnter: s, onMouseOut: c, type: f }) {
  return k.jsx("circle", { onMouseDown: l, onMouseEnter: s, onMouseOut: c, className: ct([DA, `${DA}-${f}`]), cx: wL(r, u, t), cy: AL(i, u, t), r: u, stroke: "transparent", fill: "transparent" });
}
function TL({ isReconnectable: t, reconnectRadius: r, edge: i, sourceX: u, sourceY: l, targetX: s, targetY: c, sourcePosition: f, targetPosition: g, onReconnect: h, onReconnectStart: v, onReconnectEnd: y, setReconnecting: m, setUpdateHover: b }) {
  const _ = Ze(), A = (T, N) => {
    if (T.button !== 0)
      return;
    const { autoPanOnConnect: R, domNode: q, connectionMode: L, connectionRadius: j, lib: Z, onConnectStart: Y, cancelConnection: J, nodeLookup: Q, rfId: D, panBy: X, updateConnection: z } = _.getState(), B = N.type === "target", $ = (O, G) => {
      m(!1), y == null || y(O, i, N.type, G);
    }, U = (O) => h == null ? void 0 : h(i, O), ae = (O, G) => {
      m(!0), v == null || v(T, i, N.type), Y == null || Y(O, G);
    };
    Ny.onPointerDown(T.nativeEvent, {
      autoPanOnConnect: R,
      connectionMode: L,
      connectionRadius: j,
      domNode: q,
      handleId: N.id,
      nodeId: N.nodeId,
      nodeLookup: Q,
      isTarget: B,
      edgeUpdaterType: N.type,
      lib: Z,
      flowId: D,
      cancelConnection: J,
      panBy: X,
      isValidConnection: (...O) => {
        var G, ie;
        return ((ie = (G = _.getState()).isValidConnection) == null ? void 0 : ie.call(G, ...O)) ?? !0;
      },
      onConnect: U,
      onConnectStart: ae,
      onConnectEnd: (...O) => {
        var G, ie;
        return (ie = (G = _.getState()).onConnectEnd) == null ? void 0 : ie.call(G, ...O);
      },
      onReconnectEnd: $,
      updateConnection: z,
      getTransform: () => _.getState().transform,
      getFromHandle: () => _.getState().connection.fromHandle,
      dragThreshold: _.getState().connectionDragThreshold,
      handleDomNode: T.currentTarget
    });
  }, w = (T) => A(T, { nodeId: i.target, id: i.targetHandle ?? null, type: "target" }), E = (T) => A(T, { nodeId: i.source, id: i.sourceHandle ?? null, type: "source" }), M = () => b(!0), S = () => b(!1);
  return k.jsxs(k.Fragment, { children: [(t === !0 || t === "source") && k.jsx(LA, { position: f, centerX: u, centerY: l, radius: r, onMouseDown: w, onMouseEnter: M, onMouseOut: S, type: "source" }), (t === !0 || t === "target") && k.jsx(LA, { position: g, centerX: s, centerY: c, radius: r, onMouseDown: E, onMouseEnter: M, onMouseOut: S, type: "target" })] });
}
function ML({ id: t, edgesFocusable: r, edgesReconnectable: i, elementsSelectable: u, onClick: l, onDoubleClick: s, onContextMenu: c, onMouseEnter: f, onMouseMove: g, onMouseLeave: h, reconnectRadius: v, onReconnect: y, onReconnectStart: m, onReconnectEnd: b, rfId: _, edgeTypes: A, noPanClassName: w, onError: E, disableKeyboardA11y: M }) {
  let S = Oe((ye) => ye.edgeLookup.get(t));
  const T = Oe((ye) => ye.defaultEdgeOptions);
  S = T ? { ...T, ...S } : S;
  let N = S.type || "default", R = (A == null ? void 0 : A[N]) || zA[N];
  R === void 0 && (E == null || E("011", vn.error011(N)), N = "default", R = (A == null ? void 0 : A.default) || zA.default);
  const q = !!(S.focusable || r && typeof S.focusable > "u"), L = typeof y < "u" && (S.reconnectable || i && typeof S.reconnectable > "u"), j = !!(S.selectable || u && typeof S.selectable > "u"), Z = re.useRef(null), [Y, J] = re.useState(!1), [Q, D] = re.useState(!1), X = Ze(), { zIndex: z = S.zIndex, sourceX: B, sourceY: $, targetX: U, targetY: ae, sourcePosition: O, targetPosition: G } = Oe(re.useCallback((ye) => {
    const he = ye.nodeLookup.get(S.source), me = ye.nodeLookup.get(S.target);
    if (!he || !me)
      return OA;
    const ge = h6({
      id: t,
      sourceNode: he,
      targetNode: me,
      sourceHandle: S.sourceHandle || null,
      targetHandle: S.targetHandle || null,
      connectionMode: ye.connectionMode,
      onError: E
    }), Ae = i6({
      selected: S.selected,
      zIndex: S.zIndex,
      sourceNode: he,
      targetNode: me,
      elevateOnSelect: ye.elevateEdgesOnSelect,
      zIndexMode: ye.zIndexMode
    });
    return {
      ...ge || OA,
      zIndex: Ae
    };
  }, [S.source, S.target, S.sourceHandle, S.targetHandle, S.selected, S.zIndex]), $e), ie = re.useMemo(() => S.markerStart ? `url('#${qy(S.markerStart, _)}')` : void 0, [S.markerStart, _]), H = re.useMemo(() => S.markerEnd ? `url('#${qy(S.markerEnd, _)}')` : void 0, [S.markerEnd, _]);
  if (S.hidden || B === null || $ === null || U === null || ae === null)
    return null;
  const K = (ye) => {
    var Ae;
    const { addSelectedEdges: he, unselectNodesAndEdges: me, multiSelectionActive: ge } = X.getState();
    j && (X.setState({ nodesSelectionActive: !1 }), S.selected && ge ? (me({ nodes: [], edges: [S] }), (Ae = Z.current) == null || Ae.blur()) : he([t])), l && l(ye, S);
  }, P = s ? (ye) => {
    s(ye, { ...S });
  } : void 0, ue = c ? (ye) => {
    c(ye, { ...S });
  } : void 0, W = f ? (ye) => {
    f(ye, { ...S });
  } : void 0, se = g ? (ye) => {
    g(ye, { ...S });
  } : void 0, de = h ? (ye) => {
    h(ye, { ...S });
  } : void 0, ve = (ye) => {
    var he;
    if (!M && bM.includes(ye.key) && j) {
      const { unselectNodesAndEdges: me, addSelectedEdges: ge } = X.getState();
      ye.key === "Escape" ? ((he = Z.current) == null || he.blur(), me({ edges: [S] })) : ge([t]);
    }
  };
  return k.jsx("svg", { style: { zIndex: z }, children: k.jsxs("g", { className: ct([
    "react-flow__edge",
    `react-flow__edge-${N}`,
    S.className,
    w,
    {
      selected: S.selected,
      animated: S.animated,
      inactive: !j && !l,
      updating: Y,
      selectable: j
    }
  ]), onClick: K, onDoubleClick: P, onContextMenu: ue, onMouseEnter: W, onMouseMove: se, onMouseLeave: de, onKeyDown: q ? ve : void 0, tabIndex: q ? 0 : void 0, role: S.ariaRole ?? (q ? "group" : "img"), "aria-roledescription": "edge", "data-id": t, "data-testid": `rf__edge-${t}`, "aria-label": S.ariaLabel === null ? void 0 : S.ariaLabel || `Edge from ${S.source} to ${S.target}`, "aria-describedby": q ? `${PM}-${_}` : void 0, ref: Z, ...S.domAttributes, children: [!Q && k.jsx(R, { id: t, source: S.source, target: S.target, type: S.type, selected: S.selected, animated: S.animated, selectable: j, deletable: S.deletable ?? !0, label: S.label, labelStyle: S.labelStyle, labelShowBg: S.labelShowBg, labelBgStyle: S.labelBgStyle, labelBgPadding: S.labelBgPadding, labelBgBorderRadius: S.labelBgBorderRadius, sourceX: B, sourceY: $, targetX: U, targetY: ae, sourcePosition: O, targetPosition: G, data: S.data, style: S.style, sourceHandleId: S.sourceHandle, targetHandleId: S.targetHandle, markerStart: ie, markerEnd: H, pathOptions: "pathOptions" in S ? S.pathOptions : void 0, interactionWidth: S.interactionWidth }), L && k.jsx(TL, { edge: S, isReconnectable: L, reconnectRadius: v, onReconnect: y, onReconnectStart: m, onReconnectEnd: b, sourceX: B, sourceY: $, targetX: U, targetY: ae, sourcePosition: O, targetPosition: G, setUpdateHover: J, setReconnecting: D })] }) });
}
var CL = re.memo(ML);
const qL = (t) => ({
  edgesFocusable: t.edgesFocusable,
  edgesReconnectable: t.edgesReconnectable,
  elementsSelectable: t.elementsSelectable,
  connectionMode: t.connectionMode,
  onError: t.onError
});
function wC({ defaultMarkerColor: t, onlyRenderVisibleElements: r, rfId: i, edgeTypes: u, noPanClassName: l, onReconnect: s, onEdgeContextMenu: c, onEdgeMouseEnter: f, onEdgeMouseMove: g, onEdgeMouseLeave: h, onEdgeClick: v, reconnectRadius: y, onEdgeDoubleClick: m, onReconnectStart: b, onReconnectEnd: _, disableKeyboardA11y: A }) {
  const { edgesFocusable: w, edgesReconnectable: E, elementsSelectable: M, onError: S } = Oe(qL, $e), T = hL(r);
  return k.jsxs("div", { className: "react-flow__edges", children: [k.jsx(mL, { defaultColor: t, rfId: i }), T.map((N) => k.jsx(CL, { id: N, edgesFocusable: w, edgesReconnectable: E, elementsSelectable: M, noPanClassName: l, onReconnect: s, onContextMenu: c, onMouseEnter: f, onMouseMove: g, onMouseLeave: h, onClick: v, reconnectRadius: y, onDoubleClick: m, onReconnectStart: b, onReconnectEnd: _, rfId: i, onError: S, edgeTypes: u, disableKeyboardA11y: A }, N))] });
}
wC.displayName = "EdgeRenderer";
const RL = re.memo(wC), HA = (t) => `translate(${t[0]}px,${t[1]}px) scale(${t[2]})`;
function NL({ children: t }) {
  const r = Ze(), i = re.useRef(null), [u] = re.useState(() => r.getState().transform);
  return nC(() => {
    let l = null;
    const s = () => {
      const c = r.getState().transform;
      l && c[0] === l[0] && c[1] === l[1] && c[2] === l[2] || (l = c, i.current && (i.current.style.transform = HA(c)));
    };
    return s(), r.subscribe(s);
  }, [r]), k.jsx("div", { ref: i, className: "react-flow__viewport xyflow__viewport react-flow__container", style: { transform: HA(u) }, children: t });
}
function zL(t) {
  const r = bs(), i = re.useRef(!1);
  re.useEffect(() => {
    !i.current && r.viewportInitialized && t && (setTimeout(() => t(r), 1), i.current = !0);
  }, [t, r.viewportInitialized]);
}
const OL = (t) => {
  var r;
  return (r = t.panZoom) == null ? void 0 : r.syncViewport;
};
function DL(t) {
  const r = Oe(OL), i = Ze();
  return re.useEffect(() => {
    t && (r == null || r(t), i.setState({ transform: [t.x, t.y, t.zoom] }));
  }, [t, r]), null;
}
function LL(t) {
  return t.connection.inProgress ? { ...t.connection, to: Qu(t.connection.to, t.transform) } : { ...t.connection };
}
function HL(t) {
  return LL;
}
function jL(t) {
  const r = HL();
  return Oe(r, $e);
}
const BL = (t) => ({
  nodesConnectable: t.nodesConnectable,
  isValid: t.connection.isValid,
  inProgress: t.connection.inProgress,
  width: t.width,
  height: t.height
});
function UL({ containerStyle: t, style: r, type: i, component: u }) {
  const { nodesConnectable: l, width: s, height: c, isValid: f, inProgress: g } = Oe(BL, $e);
  return !(s && l && g) ? null : k.jsx("svg", { style: t, width: s, height: c, className: "react-flow__connectionline react-flow__container", children: k.jsx("g", { className: ct(["react-flow__connection", SM(f)]), children: k.jsx(AC, { style: r, type: i, CustomComponent: u, isValid: f }) }) });
}
const AC = ({ style: t, type: r = Or.Bezier, CustomComponent: i, isValid: u }) => {
  const { inProgress: l, from: s, fromNode: c, fromHandle: f, fromPosition: g, to: h, toNode: v, toHandle: y, toPosition: m, pointer: b } = jL();
  if (!l)
    return;
  if (i)
    return k.jsx(i, { connectionLineType: r, connectionLineStyle: t, fromNode: c, fromHandle: f, fromX: s.x, fromY: s.y, toX: h.x, toY: h.y, fromPosition: g, toPosition: m, connectionStatus: SM(u), toNode: v, toHandle: y, pointer: b });
  let _ = "";
  const A = {
    sourceX: s.x,
    sourceY: s.y,
    sourcePosition: g,
    targetX: h.x,
    targetY: h.y,
    targetPosition: m
  };
  switch (r) {
    case Or.Bezier:
      [_] = LM(A);
      break;
    case Or.SimpleBezier:
      [_] = dC(A);
      break;
    case Or.Step:
      [_] = Cy({
        ...A,
        borderRadius: 0
      });
      break;
    case Or.SmoothStep:
      [_] = Cy(A);
      break;
    default:
      [_] = jM(A);
  }
  return k.jsx("path", { d: _, fill: "none", className: "react-flow__connection-path", style: t });
};
AC.displayName = "ConnectionLine";
const GL = {};
function jA(t = GL) {
  re.useRef(t), Ze(), re.useEffect(() => {
  }, [t]);
}
function VL() {
  Ze(), re.useRef(!1), re.useEffect(() => {
  }, []);
}
function TC({ nodeTypes: t, edgeTypes: r, onInit: i, onNodeClick: u, onEdgeClick: l, onNodeDoubleClick: s, onEdgeDoubleClick: c, onNodeMouseEnter: f, onNodeMouseMove: g, onNodeMouseLeave: h, onNodeContextMenu: v, onSelectionContextMenu: y, onSelectionStart: m, onSelectionEnd: b, connectionLineType: _, connectionLineStyle: A, connectionLineComponent: w, connectionLineContainerStyle: E, selectionKeyCode: M, selectionOnDrag: S, selectionMode: T, multiSelectionKeyCode: N, panActivationKeyCode: R, zoomActivationKeyCode: q, deleteKeyCode: L, onlyRenderVisibleElements: j, elementsSelectable: Z, defaultViewport: Y, translateExtent: J, minZoom: Q, maxZoom: D, preventScrolling: X, defaultMarkerColor: z, zoomOnScroll: B, zoomOnPinch: $, panOnScroll: U, panOnScrollSpeed: ae, panOnScrollMode: O, zoomOnDoubleClick: G, panOnDrag: ie, autoPanOnSelection: H, onPaneClick: K, onPaneMouseEnter: P, onPaneMouseMove: ue, onPaneMouseLeave: W, onPaneScroll: se, onPaneContextMenu: de, paneClickDistance: ve, nodeClickDistance: ye, onEdgeContextMenu: he, onEdgeMouseEnter: me, onEdgeMouseMove: ge, onEdgeMouseLeave: Ae, reconnectRadius: xe, onReconnect: Pe, onReconnectStart: tt, onReconnectEnd: xt, noDragClassName: gt, noWheelClassName: St, noPanClassName: Ke, disableKeyboardA11y: ke, nodeExtent: vt, rfId: qt, viewport: Tt, onViewportChange: ft, nodesDraggable: tr }) {
  return jA(t), jA(r), VL(), zL(i), DL(Tt), k.jsx(rL, { onPaneClick: K, onPaneMouseEnter: P, onPaneMouseMove: ue, onPaneMouseLeave: W, onPaneContextMenu: de, onPaneScroll: se, paneClickDistance: ve, deleteKeyCode: L, selectionKeyCode: M, selectionOnDrag: S, selectionMode: T, onSelectionStart: m, onSelectionEnd: b, multiSelectionKeyCode: N, panActivationKeyCode: R, zoomActivationKeyCode: q, elementsSelectable: Z, zoomOnScroll: B, zoomOnPinch: $, zoomOnDoubleClick: G, panOnScroll: U, panOnScrollSpeed: ae, panOnScrollMode: O, panOnDrag: ie, autoPanOnSelection: H, defaultViewport: Y, translateExtent: J, minZoom: Q, maxZoom: D, onSelectionContextMenu: y, preventScrolling: X, noDragClassName: gt, noWheelClassName: St, noPanClassName: Ke, disableKeyboardA11y: ke, onViewportChange: ft, isControlledViewport: !!Tt, children: k.jsxs(NL, { children: [k.jsx(RL, { edgeTypes: r, onEdgeClick: l, onEdgeDoubleClick: c, onReconnect: Pe, onReconnectStart: tt, onReconnectEnd: xt, onlyRenderVisibleElements: j, onEdgeContextMenu: he, onEdgeMouseEnter: me, onEdgeMouseMove: ge, onEdgeMouseLeave: Ae, reconnectRadius: xe, defaultMarkerColor: z, noPanClassName: Ke, disableKeyboardA11y: ke, rfId: qt }), k.jsx(UL, { style: A, type: _, component: w, containerStyle: E }), k.jsx("div", { className: "react-flow__edgelabel-renderer" }), k.jsx(dL, { nodeTypes: t, onNodeClick: u, onNodeDoubleClick: s, onNodeMouseEnter: f, onNodeMouseMove: g, onNodeMouseLeave: h, onNodeContextMenu: v, nodeClickDistance: ye, onlyRenderVisibleElements: j, noPanClassName: Ke, noDragClassName: gt, disableKeyboardA11y: ke, nodeExtent: vt, rfId: qt, nodesDraggable: tr }), k.jsx("div", { className: "react-flow__viewport-portal" })] }) });
}
TC.displayName = "GraphView";
const YL = re.memo(TC), kL = CM(), BA = ({ nodes: t, edges: r, defaultNodes: i, defaultEdges: u, width: l, height: s, fitView: c, fitViewOptions: f, minZoom: g = 0.5, maxZoom: h = 2, nodeOrigin: v, nodeExtent: y, zIndexMode: m = "basic" } = {}) => {
  const b = /* @__PURE__ */ new Map(), _ = /* @__PURE__ */ new Map(), A = /* @__PURE__ */ new Map(), w = /* @__PURE__ */ new Map(), E = u ?? r ?? [], M = i ?? t ?? [], S = v ?? [0, 0], T = y ?? zu;
  GM(A, w, E);
  const { nodesInitialized: N } = Ry(M, b, _, {
    nodeOrigin: S,
    nodeExtent: T,
    zIndexMode: m
  });
  let R = [0, 0, 1];
  if (c && l && s) {
    const q = Zu(b, {
      filter: (Y) => !!((Y.width || Y.initialWidth) && (Y.height || Y.initialHeight))
    }), { x: L, y: j, zoom: Z } = cm(q, l, s, g, h, (f == null ? void 0 : f.padding) ?? 0.1);
    R = [L, j, Z];
  }
  return {
    rfId: "1",
    width: l ?? 0,
    height: s ?? 0,
    transform: R,
    nodes: M,
    nodesInitialized: N,
    nodeLookup: b,
    parentLookup: _,
    edges: E,
    edgeLookup: w,
    connectionLookup: A,
    onNodesChange: null,
    onEdgesChange: null,
    hasDefaultNodes: i !== void 0,
    hasDefaultEdges: u !== void 0,
    panZoom: null,
    minZoom: g,
    maxZoom: h,
    translateExtent: zu,
    nodeExtent: T,
    nodesSelectionActive: !1,
    userSelectionActive: !1,
    userSelectionRect: null,
    connectionMode: ci.Strict,
    domNode: null,
    paneDragging: !1,
    noPanClassName: "nopan",
    nodeOrigin: S,
    nodeDragThreshold: 1,
    connectionDragThreshold: 1,
    snapGrid: [15, 15],
    snapToGrid: !1,
    nodesDraggable: !0,
    nodesConnectable: !0,
    nodesFocusable: !0,
    edgesFocusable: !0,
    edgesReconnectable: !0,
    elementsSelectable: !0,
    elevateNodesOnSelect: !0,
    elevateEdgesOnSelect: !0,
    selectNodesOnDrag: !0,
    multiSelectionActive: !1,
    fitViewQueued: c ?? !1,
    fitViewOptions: f,
    fitViewResolver: null,
    connection: { ...xM },
    connectionClickStartHandle: null,
    connectOnClick: !0,
    ariaLiveMessage: "",
    autoPanOnConnect: !0,
    autoPanOnNodeDrag: !0,
    autoPanOnNodeFocus: !0,
    autoPanSpeed: 15,
    connectionRadius: 20,
    onError: kL,
    isValidConnection: void 0,
    onSelectionChangeHandlers: [],
    lib: "react",
    debug: !1,
    ariaLabelConfig: _M,
    zIndexMode: m,
    onNodesChangeMiddlewareMap: /* @__PURE__ */ new Map(),
    onEdgesChangeMiddlewareMap: /* @__PURE__ */ new Map()
  };
}, XL = ({ nodes: t, edges: r, defaultNodes: i, defaultEdges: u, width: l, height: s, fitView: c, fitViewOptions: f, minZoom: g, maxZoom: h, nodeOrigin: v, nodeExtent: y, zIndexMode: m }) => n8((b, _) => {
  async function A() {
    const { nodeLookup: w, panZoom: E, fitViewOptions: M, fitViewResolver: S, width: T, height: N, minZoom: R, maxZoom: q } = _();
    E && (await P5({
      nodes: w,
      width: T,
      height: N,
      panZoom: E,
      minZoom: R,
      maxZoom: q
    }, M), S == null || S.resolve(!0), b({ fitViewResolver: null }));
  }
  return {
    ...BA({
      nodes: t,
      edges: r,
      width: l,
      height: s,
      fitView: c,
      fitViewOptions: f,
      minZoom: g,
      maxZoom: h,
      nodeOrigin: v,
      nodeExtent: y,
      defaultNodes: i,
      defaultEdges: u,
      zIndexMode: m
    }),
    setNodes: (w) => {
      const { nodeLookup: E, parentLookup: M, nodeOrigin: S, elevateNodesOnSelect: T, fitViewQueued: N, zIndexMode: R, nodesSelectionActive: q } = _(), { nodesInitialized: L, hasSelectedNodes: j } = Ry(w, E, M, {
        nodeOrigin: S,
        nodeExtent: y,
        elevateNodesOnSelect: T,
        checkEquality: !0,
        zIndexMode: R
      }), Z = q && j;
      N && L ? (A(), b({
        nodes: w,
        nodesInitialized: L,
        fitViewQueued: !1,
        fitViewOptions: void 0,
        nodesSelectionActive: Z
      })) : b({ nodes: w, nodesInitialized: L, nodesSelectionActive: Z });
    },
    setEdges: (w) => {
      const { connectionLookup: E, edgeLookup: M } = _();
      GM(E, M, w), b({ edges: w });
    },
    setDefaultNodesAndEdges: (w, E) => {
      if (w) {
        const { setNodes: M } = _();
        M(w), b({ hasDefaultNodes: !0 });
      }
      if (E) {
        const { setEdges: M } = _();
        M(E), b({ hasDefaultEdges: !0 });
      }
    },
    /*
     * Every node gets registered at a ResizeObserver. Whenever a node
     * changes its dimensions, this function is called to measure the
     * new dimensions and update the nodes.
     */
    updateNodeInternals: (w) => {
      const { triggerNodeChanges: E, nodeLookup: M, parentLookup: S, domNode: T, nodeOrigin: N, nodeExtent: R, debug: q, fitViewQueued: L, zIndexMode: j } = _(), { changes: Z, updatedInternals: Y } = x6(w, M, S, T, N, R, j);
      Y && (y6(M, S, { nodeOrigin: N, nodeExtent: R, zIndexMode: j }), L ? (A(), b({ fitViewQueued: !1, fitViewOptions: void 0 })) : b({}), (Z == null ? void 0 : Z.length) > 0 && (q && console.log("React Flow: trigger node changes", Z), E == null || E(Z)));
    },
    updateNodePositions: (w, E = !1) => {
      const M = [];
      let S = [];
      const { nodeLookup: T, triggerNodeChanges: N, connection: R, updateConnection: q, onNodesChangeMiddlewareMap: L } = _();
      for (const [j, Z] of w) {
        const Y = T.get(j), J = !!(Y != null && Y.expandParent && (Y != null && Y.parentId) && (Z != null && Z.position)), Q = {
          id: j,
          type: "position",
          position: J ? {
            x: Math.max(0, Z.position.x),
            y: Math.max(0, Z.position.y)
          } : Z.position,
          dragging: E
        };
        if (Y && R.inProgress && R.fromNode.id === Y.id) {
          const D = ca(Y, R.fromHandle, _e.Left, !0);
          q({ ...R, from: D });
        }
        J && Y.parentId && M.push({
          id: j,
          parentId: Y.parentId,
          rect: {
            ...Z.internals.positionAbsolute,
            width: Z.measured.width ?? 0,
            height: Z.measured.height ?? 0
          }
        }), S.push(Q);
      }
      if (M.length > 0) {
        const { parentLookup: j, nodeOrigin: Z } = _(), Y = pm(M, T, j, Z);
        S.push(...Y);
      }
      for (const j of L.values())
        S = j(S);
      N(S);
    },
    triggerNodeChanges: (w) => {
      const { onNodesChange: E, setNodes: M, nodes: S, hasDefaultNodes: T, debug: N } = _();
      if (w != null && w.length) {
        if (T) {
          const R = E8(w, S);
          M(R);
        }
        N && console.log("React Flow: trigger node changes", w), E == null || E(w);
      }
    },
    triggerEdgeChanges: (w) => {
      const { onEdgesChange: E, setEdges: M, edges: S, hasDefaultEdges: T, debug: N } = _();
      if (w != null && w.length) {
        if (T) {
          const R = w8(w, S);
          M(R);
        }
        N && console.log("React Flow: trigger edge changes", w), E == null || E(w);
      }
    },
    addSelectedNodes: (w) => {
      const { multiSelectionActive: E, edgeLookup: M, nodeLookup: S, triggerNodeChanges: T, triggerEdgeChanges: N } = _();
      if (E) {
        const R = w.map((q) => na(q, !0));
        T(R);
        return;
      }
      T(ii(S, /* @__PURE__ */ new Set([...w]), !0)), N(ii(M));
    },
    addSelectedEdges: (w) => {
      const { multiSelectionActive: E, edgeLookup: M, nodeLookup: S, triggerNodeChanges: T, triggerEdgeChanges: N } = _();
      if (E) {
        const R = w.map((q) => na(q, !0));
        N(R);
        return;
      }
      N(ii(M, /* @__PURE__ */ new Set([...w]))), T(ii(S, /* @__PURE__ */ new Set(), !0));
    },
    unselectNodesAndEdges: ({ nodes: w, edges: E } = {}) => {
      const { edges: M, nodes: S, nodeLookup: T, triggerNodeChanges: N, triggerEdgeChanges: R } = _(), q = w || S, L = E || M, j = [];
      for (const Y of q) {
        if (!Y.selected)
          continue;
        const J = T.get(Y.id);
        J && (J.selected = !1), j.push(na(Y.id, !1));
      }
      const Z = [];
      for (const Y of L)
        Y.selected && Z.push(na(Y.id, !1));
      N(j), R(Z);
    },
    setMinZoom: (w) => {
      const { panZoom: E, maxZoom: M } = _();
      E == null || E.setScaleExtent([w, M]), b({ minZoom: w });
    },
    setMaxZoom: (w) => {
      const { panZoom: E, minZoom: M } = _();
      E == null || E.setScaleExtent([M, w]), b({ maxZoom: w });
    },
    setTranslateExtent: (w) => {
      var E;
      (E = _().panZoom) == null || E.setTranslateExtent(w), b({ translateExtent: w });
    },
    resetSelectedElements: () => {
      const { edges: w, nodes: E, triggerNodeChanges: M, triggerEdgeChanges: S, elementsSelectable: T } = _();
      if (!T)
        return;
      const N = E.reduce((q, L) => L.selected ? [...q, na(L.id, !1)] : q, []), R = w.reduce((q, L) => L.selected ? [...q, na(L.id, !1)] : q, []);
      M(N), S(R);
    },
    setNodeExtent: (w) => {
      const { nodes: E, nodeLookup: M, parentLookup: S, nodeOrigin: T, elevateNodesOnSelect: N, nodeExtent: R, zIndexMode: q } = _();
      w[0][0] === R[0][0] && w[0][1] === R[0][1] && w[1][0] === R[1][0] && w[1][1] === R[1][1] || (Ry(E, M, S, {
        nodeOrigin: T,
        nodeExtent: w,
        elevateNodesOnSelect: N,
        checkEquality: !1,
        zIndexMode: q
      }), b({ nodeExtent: w }));
    },
    panBy: (w) => {
      const { transform: E, width: M, height: S, panZoom: T, translateExtent: N } = _();
      return S6({ delta: w, panZoom: T, transform: E, translateExtent: N, width: M, height: S });
    },
    setCenter: async (w, E, M) => {
      const { width: S, height: T, maxZoom: N, panZoom: R } = _();
      if (!R)
        return !1;
      const q = typeof (M == null ? void 0 : M.zoom) < "u" ? M.zoom : N;
      return await R.setViewport({
        x: S / 2 - w * q,
        y: T / 2 - E * q,
        zoom: q
      }, { duration: M == null ? void 0 : M.duration, ease: M == null ? void 0 : M.ease, interpolate: M == null ? void 0 : M.interpolate }), !0;
    },
    cancelConnection: () => {
      b({
        connection: { ...xM }
      });
    },
    updateConnection: (w) => {
      b({ connection: w });
    },
    reset: () => b({ ...BA() })
  };
}, Object.is);
function MC({ initialNodes: t, initialEdges: r, defaultNodes: i, defaultEdges: u, initialWidth: l, initialHeight: s, initialMinZoom: c, initialMaxZoom: f, initialFitViewOptions: g, fitView: h, nodeOrigin: v, nodeExtent: y, zIndexMode: m, children: b }) {
  const [_] = re.useState(() => XL({
    nodes: t,
    edges: r,
    defaultNodes: i,
    defaultEdges: u,
    width: l,
    height: s,
    fitView: h,
    minZoom: c,
    maxZoom: f,
    fitViewOptions: g,
    nodeOrigin: v,
    nodeExtent: y,
    zIndexMode: m
  }));
  return k.jsx(r8, { value: _, children: k.jsx(q8, { children: k.jsx(X8, { children: b }) }) });
}
function IL({ children: t, nodes: r, edges: i, defaultNodes: u, defaultEdges: l, width: s, height: c, fitView: f, fitViewOptions: g, minZoom: h, maxZoom: v, nodeOrigin: y, nodeExtent: m, zIndexMode: b }) {
  return re.useContext(ms) ? k.jsx(k.Fragment, { children: t }) : k.jsx(MC, { initialNodes: r, initialEdges: i, defaultNodes: u, defaultEdges: l, initialWidth: s, initialHeight: c, fitView: f, initialFitViewOptions: g, initialMinZoom: h, initialMaxZoom: v, nodeOrigin: y, nodeExtent: m, zIndexMode: b, children: t });
}
const ZL = {
  width: "100%",
  height: "100%",
  overflow: "hidden",
  position: "relative",
  zIndex: 0
};
function KL({ nodes: t, edges: r, defaultNodes: i, defaultEdges: u, className: l, nodeTypes: s, edgeTypes: c, onNodeClick: f, onEdgeClick: g, onInit: h, onMove: v, onMoveStart: y, onMoveEnd: m, onConnect: b, onConnectStart: _, onConnectEnd: A, onClickConnectStart: w, onClickConnectEnd: E, onNodeMouseEnter: M, onNodeMouseMove: S, onNodeMouseLeave: T, onNodeContextMenu: N, onNodeDoubleClick: R, onNodeDragStart: q, onNodeDrag: L, onNodeDragStop: j, onNodesDelete: Z, onEdgesDelete: Y, onDelete: J, onSelectionChange: Q, onSelectionDragStart: D, onSelectionDrag: X, onSelectionDragStop: z, onSelectionContextMenu: B, onSelectionStart: $, onSelectionEnd: U, onBeforeDelete: ae, connectionMode: O, connectionLineType: G = Or.Bezier, connectionLineStyle: ie, connectionLineComponent: H, connectionLineContainerStyle: K, deleteKeyCode: P = "Backspace", selectionKeyCode: ue = "Shift", selectionOnDrag: W = !1, selectionMode: se = Ou.Full, panActivationKeyCode: de = "Space", multiSelectionKeyCode: ve = Lu() ? "Meta" : "Control", zoomActivationKeyCode: ye = Lu() ? "Meta" : "Control", snapToGrid: he, snapGrid: me, onlyRenderVisibleElements: ge = !1, selectNodesOnDrag: Ae, nodesDraggable: xe, autoPanOnNodeFocus: Pe, nodesConnectable: tt, nodesFocusable: xt, nodeOrigin: gt = WM, edgesFocusable: St, edgesReconnectable: Ke, elementsSelectable: ke = !0, defaultViewport: vt = p8, minZoom: qt = 0.5, maxZoom: Tt = 2, translateExtent: ft = zu, preventScrolling: tr = !0, nodeExtent: qn, defaultMarkerColor: _i = "#b1b1b7", zoomOnScroll: va = !0, zoomOnPinch: Et = !0, panOnScroll: Je = !1, panOnScrollSpeed: mn = 0.5, panOnScrollMode: Mt = ia.Free, zoomOnDoubleClick: Ss = !0, panOnDrag: Es = !0, onPaneClick: ws, onPaneMouseEnter: pa, onPaneMouseMove: ya, onPaneMouseLeave: ma, onPaneScroll: Rn, onPaneContextMenu: ba, paneClickDistance: Hr = 1, nodeClickDistance: As = 0, children: Fu, onReconnect: xi, onReconnectStart: jr, onReconnectEnd: Ts, onEdgeContextMenu: Ju, onEdgeDoubleClick: Pu, onEdgeMouseEnter: Wu, onEdgeMouseMove: Si, onEdgeMouseLeave: Ei, reconnectRadius: el = 10, onNodesChange: tl, onEdgesChange: bn, noDragClassName: dt = "nodrag", noWheelClassName: wt = "nowheel", noPanClassName: Nn = "nopan", fitView: _a, fitViewOptions: nl, connectOnClick: Ms, attributionPosition: rl, proOptions: Br, defaultEdgeOptions: wi, elevateNodesOnSelect: nr = !0, elevateEdgesOnSelect: rr = !1, disableKeyboardA11y: ar = !1, autoPanOnConnect: ir, autoPanOnNodeDrag: at, autoPanOnSelection: al = !0, autoPanSpeed: il, connectionRadius: zn, isValidConnection: ur, onError: Cs, style: ul, id: Ai, nodeDragThreshold: qs, connectionDragThreshold: xa, viewport: Sa, onViewportChange: un, width: Rt, height: ll, colorMode: Rs = "light", debug: Ti, onScroll: Ur, ariaLabelConfig: Ea, zIndexMode: ol = "basic", ...Nt }, Mi) {
  const wa = Ai || "1", Ci = _8(Rs), lr = re.useCallback((qi) => {
    qi.currentTarget.scrollTo({ top: 0, left: 0, behavior: "instant" }), Ur == null || Ur(qi);
  }, [Ur]);
  return k.jsx("div", { "data-testid": "rf__wrapper", ...Nt, onScroll: lr, style: { ...ul, ...ZL }, ref: Mi, className: ct(["react-flow", l, Ci]), id: Ai, role: "application", children: k.jsxs(IL, { nodes: t, edges: r, width: Rt, height: ll, fitView: _a, fitViewOptions: nl, minZoom: qt, maxZoom: Tt, nodeOrigin: gt, nodeExtent: qn, zIndexMode: ol, children: [k.jsx(b8, { nodes: t, edges: r, defaultNodes: i, defaultEdges: u, onConnect: b, onConnectStart: _, onConnectEnd: A, onClickConnectStart: w, onClickConnectEnd: E, nodesDraggable: xe, autoPanOnNodeFocus: Pe, nodesConnectable: tt, nodesFocusable: xt, edgesFocusable: St, edgesReconnectable: Ke, elementsSelectable: ke, elevateNodesOnSelect: nr, elevateEdgesOnSelect: rr, minZoom: qt, maxZoom: Tt, nodeExtent: qn, onNodesChange: tl, onEdgesChange: bn, snapToGrid: he, snapGrid: me, connectionMode: O, translateExtent: ft, connectOnClick: Ms, defaultEdgeOptions: wi, fitView: _a, fitViewOptions: nl, onNodesDelete: Z, onEdgesDelete: Y, onDelete: J, onNodeDragStart: q, onNodeDrag: L, onNodeDragStop: j, onSelectionDrag: X, onSelectionDragStart: D, onSelectionDragStop: z, onMove: v, onMoveStart: y, onMoveEnd: m, noPanClassName: Nn, nodeOrigin: gt, rfId: wa, autoPanOnConnect: ir, autoPanOnNodeDrag: at, autoPanSpeed: il, onError: Cs, connectionRadius: zn, isValidConnection: ur, selectNodesOnDrag: Ae, nodeDragThreshold: qs, connectionDragThreshold: xa, onBeforeDelete: ae, debug: Ti, ariaLabelConfig: Ea, zIndexMode: ol }), k.jsx(YL, { onInit: h, onNodeClick: f, onEdgeClick: g, onNodeMouseEnter: M, onNodeMouseMove: S, onNodeMouseLeave: T, onNodeContextMenu: N, onNodeDoubleClick: R, nodeTypes: s, edgeTypes: c, connectionLineType: G, connectionLineStyle: ie, connectionLineComponent: H, connectionLineContainerStyle: K, selectionKeyCode: ue, selectionOnDrag: W, selectionMode: se, deleteKeyCode: P, multiSelectionKeyCode: ve, panActivationKeyCode: de, zoomActivationKeyCode: ye, onlyRenderVisibleElements: ge, defaultViewport: vt, translateExtent: ft, minZoom: qt, maxZoom: Tt, preventScrolling: tr, zoomOnScroll: va, zoomOnPinch: Et, zoomOnDoubleClick: Ss, panOnScroll: Je, panOnScrollSpeed: mn, panOnScrollMode: Mt, panOnDrag: Es, autoPanOnSelection: al, onPaneClick: ws, onPaneMouseEnter: pa, onPaneMouseMove: ya, onPaneMouseLeave: ma, onPaneScroll: Rn, onPaneContextMenu: ba, paneClickDistance: Hr, nodeClickDistance: As, onSelectionContextMenu: B, onSelectionStart: $, onSelectionEnd: U, onReconnect: xi, onReconnectStart: jr, onReconnectEnd: Ts, onEdgeContextMenu: Ju, onEdgeDoubleClick: Pu, onEdgeMouseEnter: Wu, onEdgeMouseMove: Si, onEdgeMouseLeave: Ei, reconnectRadius: el, defaultMarkerColor: _i, noDragClassName: dt, noWheelClassName: wt, noPanClassName: Nn, rfId: wa, disableKeyboardA11y: ar, nodeExtent: qn, viewport: Sa, onViewportChange: un, nodesDraggable: xe }), k.jsx(v8, { onSelectionChange: Q }), Fu, k.jsx(c8, { proOptions: Br, position: rl }), k.jsx(s8, { rfId: wa, disableKeyboardA11y: ar })] }) });
}
var QL = tC(KL);
const $L = (t) => ({
  x: t.transform[0],
  y: t.transform[1],
  zoom: t.transform[2]
});
function FL() {
  return Oe($L, $e);
}
function JL({ dimensions: t, lineWidth: r, variant: i, className: u }) {
  return k.jsx("path", { strokeWidth: r, d: `M${t[0] / 2} 0 V${t[1]} M0 ${t[1] / 2} H${t[0]}`, className: ct(["react-flow__background-pattern", i, u]) });
}
function PL({ radius: t, className: r }) {
  return k.jsx("circle", { cx: t, cy: t, r: t, className: ct(["react-flow__background-pattern", "dots", r]) });
}
var Dr;
(function(t) {
  t.Lines = "lines", t.Dots = "dots", t.Cross = "cross";
})(Dr || (Dr = {}));
const WL = {
  [Dr.Dots]: 1,
  [Dr.Lines]: 1,
  [Dr.Cross]: 6
}, eH = (t) => ({ transform: t.transform, patternId: `pattern-${t.rfId}` });
function CC({
  id: t,
  variant: r = Dr.Dots,
  // only used for dots and cross
  gap: i = 20,
  // only used for lines and cross
  size: u,
  lineWidth: l = 1,
  offset: s = 0,
  color: c,
  bgColor: f,
  style: g,
  className: h,
  patternClassName: v
}) {
  const y = re.useRef(null), { transform: m, patternId: b } = Oe(eH, $e), _ = u || WL[r], A = r === Dr.Dots, w = r === Dr.Cross, E = Array.isArray(i) ? i : [i, i], M = [E[0] * m[2] || 1, E[1] * m[2] || 1], S = _ * m[2], T = Array.isArray(s) ? s : [s, s], N = w ? [S, S] : M, R = [
    T[0] * m[2] || 1 + N[0] / 2,
    T[1] * m[2] || 1 + N[1] / 2
  ], q = `${b}${t || ""}`;
  return k.jsxs("svg", { className: ct(["react-flow__background", h]), style: {
    ...g,
    ..._s,
    "--xy-background-color-props": f,
    "--xy-background-pattern-color-props": c
  }, ref: y, "data-testid": "rf__background", children: [k.jsx("pattern", { id: q, x: m[0] % M[0], y: m[1] % M[1], width: M[0], height: M[1], patternUnits: "userSpaceOnUse", patternTransform: `translate(-${R[0]},-${R[1]})`, children: A ? k.jsx(PL, { radius: S / 2, className: v }) : k.jsx(JL, { dimensions: N, lineWidth: l, variant: r, className: v }) }), k.jsx("rect", { x: "0", y: "0", width: "100%", height: "100%", fill: `url(#${q})` })] });
}
CC.displayName = "Background";
const tH = re.memo(CC);
function nH() {
  return k.jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 32 32", children: k.jsx("path", { d: "M32 18.133H18.133V32h-4.266V18.133H0v-4.266h13.867V0h4.266v13.867H32z" }) });
}
function rH() {
  return k.jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 32 5", children: k.jsx("path", { d: "M0 0h32v4.2H0z" }) });
}
function aH() {
  return k.jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 32 30", children: k.jsx("path", { d: "M3.692 4.63c0-.53.4-.938.939-.938h5.215V0H4.708C2.13 0 0 2.054 0 4.63v5.216h3.692V4.631zM27.354 0h-5.2v3.692h5.17c.53 0 .984.4.984.939v5.215H32V4.631A4.624 4.624 0 0027.354 0zm.954 24.83c0 .532-.4.94-.939.94h-5.215v3.768h5.215c2.577 0 4.631-2.13 4.631-4.707v-5.139h-3.692v5.139zm-23.677.94c-.531 0-.939-.4-.939-.94v-5.138H0v5.139c0 2.577 2.13 4.707 4.708 4.707h5.138V25.77H4.631z" }) });
}
function iH() {
  return k.jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 25 32", children: k.jsx("path", { d: "M21.333 10.667H19.81V7.619C19.81 3.429 16.38 0 12.19 0 8 0 4.571 3.429 4.571 7.619v3.048H3.048A3.056 3.056 0 000 13.714v15.238A3.056 3.056 0 003.048 32h18.285a3.056 3.056 0 003.048-3.048V13.714a3.056 3.056 0 00-3.048-3.047zM12.19 24.533a3.056 3.056 0 01-3.047-3.047 3.056 3.056 0 013.047-3.048 3.056 3.056 0 013.048 3.048 3.056 3.056 0 01-3.048 3.047zm4.724-13.866H7.467V7.619c0-2.59 2.133-4.724 4.723-4.724 2.591 0 4.724 2.133 4.724 4.724v3.048z" }) });
}
function uH() {
  return k.jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 25 32", children: k.jsx("path", { d: "M21.333 10.667H19.81V7.619C19.81 3.429 16.38 0 12.19 0c-4.114 1.828-1.37 2.133.305 2.438 1.676.305 4.42 2.59 4.42 5.181v3.048H3.047A3.056 3.056 0 000 13.714v15.238A3.056 3.056 0 003.048 32h18.285a3.056 3.056 0 003.048-3.048V13.714a3.056 3.056 0 00-3.048-3.047zM12.19 24.533a3.056 3.056 0 01-3.047-3.047 3.056 3.056 0 013.047-3.048 3.056 3.056 0 013.048 3.048 3.056 3.056 0 01-3.048 3.047z" }) });
}
function Co({ children: t, className: r, ...i }) {
  return k.jsx("button", { type: "button", className: ct(["react-flow__controls-button", r]), ...i, children: t });
}
const lH = (t) => ({
  isInteractive: t.nodesDraggable || t.nodesConnectable || t.elementsSelectable,
  minZoomReached: t.transform[2] <= t.minZoom,
  maxZoomReached: t.transform[2] >= t.maxZoom,
  ariaLabelConfig: t.ariaLabelConfig
});
function qC({ style: t, showZoom: r = !0, showFitView: i = !0, showInteractive: u = !0, fitViewOptions: l, onZoomIn: s, onZoomOut: c, onFitView: f, onInteractiveChange: g, className: h, children: v, position: y = "bottom-left", orientation: m = "vertical", "aria-label": b }) {
  const _ = Ze(), { isInteractive: A, minZoomReached: w, maxZoomReached: E, ariaLabelConfig: M } = Oe(lH, $e), { zoomIn: S, zoomOut: T, fitView: N } = bs(), R = () => {
    S(), s == null || s();
  }, q = () => {
    T(), c == null || c();
  }, L = () => {
    N(l), f == null || f();
  }, j = () => {
    _.setState({
      nodesDraggable: !A,
      nodesConnectable: !A,
      elementsSelectable: !A
    }), g == null || g(!A);
  }, Z = m === "horizontal" ? "horizontal" : "vertical";
  return k.jsxs($u, { className: ct(["react-flow__controls", Z, h]), position: y, style: t, "data-testid": "rf__controls", "aria-label": b ?? M["controls.ariaLabel"], children: [r && k.jsxs(k.Fragment, { children: [k.jsx(Co, { onClick: R, className: "react-flow__controls-zoomin", title: M["controls.zoomIn.ariaLabel"], "aria-label": M["controls.zoomIn.ariaLabel"], disabled: E, children: k.jsx(nH, {}) }), k.jsx(Co, { onClick: q, className: "react-flow__controls-zoomout", title: M["controls.zoomOut.ariaLabel"], "aria-label": M["controls.zoomOut.ariaLabel"], disabled: w, children: k.jsx(rH, {}) })] }), i && k.jsx(Co, { className: "react-flow__controls-fitview", onClick: L, title: M["controls.fitView.ariaLabel"], "aria-label": M["controls.fitView.ariaLabel"], children: k.jsx(aH, {}) }), u && k.jsx(Co, { className: "react-flow__controls-interactive", onClick: j, title: M["controls.interactive.ariaLabel"], "aria-label": M["controls.interactive.ariaLabel"], children: A ? k.jsx(uH, {}) : k.jsx(iH, {}) }), v] });
}
qC.displayName = "Controls";
re.memo(qC);
function oH({ id: t, x: r, y: i, width: u, height: l, style: s, color: c, strokeColor: f, strokeWidth: g, className: h, borderRadius: v, shapeRendering: y, selected: m, onClick: b }) {
  const { background: _, backgroundColor: A } = s || {}, w = c || _ || A;
  return k.jsx("rect", { className: ct(["react-flow__minimap-node", { selected: m }, h]), x: r, y: i, rx: v, ry: v, width: u, height: l, style: {
    fill: w,
    stroke: f,
    strokeWidth: g
  }, shapeRendering: y, onClick: b ? (E) => b(E, t) : void 0 });
}
const sH = re.memo(oH), cH = (t) => t.nodes.map((r) => r.id), my = (t) => t instanceof Function ? t : () => t;
function fH({
  nodeStrokeColor: t,
  nodeColor: r,
  nodeClassName: i = "",
  nodeBorderRadius: u = 5,
  nodeStrokeWidth: l,
  /*
   * We need to rename the prop to be `CapitalCase` so that JSX will render it as
   * a component properly.
   */
  nodeComponent: s = sH,
  onClick: c
}) {
  const f = Oe(cH, $e), g = my(r), h = my(t), v = my(i), y = typeof window > "u" || window.chrome ? "crispEdges" : "geometricPrecision";
  return k.jsx(k.Fragment, { children: f.map((m) => (
    /*
     * The split of responsibilities between MiniMapNodes and
     * NodeComponentWrapper may appear weird. However, it’s designed to
     * minimize the cost of updates when individual nodes change.
     *
     * For more details, see a similar commit in `NodeRenderer/index.tsx`.
     */
    k.jsx(hH, { id: m, nodeColorFunc: g, nodeStrokeColorFunc: h, nodeClassNameFunc: v, nodeBorderRadius: u, nodeStrokeWidth: l, NodeComponent: s, onClick: c, shapeRendering: y }, m)
  )) });
}
function dH({ id: t, nodeColorFunc: r, nodeStrokeColorFunc: i, nodeClassNameFunc: u, nodeBorderRadius: l, nodeStrokeWidth: s, shapeRendering: c, NodeComponent: f, onClick: g }) {
  const { node: h, x: v, y, width: m, height: b } = Oe((_) => {
    const A = _.nodeLookup.get(t);
    if (!A)
      return { node: void 0, x: 0, y: 0, width: 0, height: 0 };
    const w = A.internals.userNode, { x: E, y: M } = A.internals.positionAbsolute, { width: S, height: T } = Cn(w);
    return {
      node: w,
      x: E,
      y: M,
      width: S,
      height: T
    };
  }, $e);
  return !h || h.hidden || !qM(h) ? null : k.jsx(f, { x: v, y, width: m, height: b, style: h.style, selected: !!h.selected, className: u(h), color: r(h), borderRadius: l, strokeColor: i(h), strokeWidth: s, shapeRendering: c, onClick: g, id: h.id });
}
const hH = re.memo(dH);
var gH = re.memo(fH);
const vH = 200, pH = 150, yH = (t) => !t.hidden, mH = (t) => {
  const r = {
    x: -t.transform[0] / t.transform[2],
    y: -t.transform[1] / t.transform[2],
    width: t.width / t.transform[2],
    height: t.height / t.transform[2]
  };
  return {
    viewBB: r,
    boundingRect: t.nodeLookup.size > 0 ? TM(Zu(t.nodeLookup, { filter: yH }), r) : r,
    rfId: t.rfId,
    panZoom: t.panZoom,
    translateExtent: t.translateExtent,
    flowWidth: t.width,
    flowHeight: t.height,
    ariaLabelConfig: t.ariaLabelConfig
  };
}, UA = (t, r) => t.x === r.x && t.y === r.y && t.width === r.width && t.height === r.height, bH = (t, r) => UA(t.viewBB, r.viewBB) && UA(t.boundingRect, r.boundingRect) && t.rfId === r.rfId && t.panZoom === r.panZoom && t.translateExtent === r.translateExtent && t.flowWidth === r.flowWidth && t.flowHeight === r.flowHeight && t.ariaLabelConfig === r.ariaLabelConfig, _H = "react-flow__minimap-desc";
function RC({
  style: t,
  className: r,
  nodeStrokeColor: i,
  nodeColor: u,
  nodeClassName: l = "",
  nodeBorderRadius: s = 5,
  nodeStrokeWidth: c,
  /*
   * We need to rename the prop to be `CapitalCase` so that JSX will render it as
   * a component properly.
   */
  nodeComponent: f,
  bgColor: g,
  maskColor: h,
  maskStrokeColor: v,
  maskStrokeWidth: y,
  position: m = "bottom-right",
  onClick: b,
  onNodeClick: _,
  pannable: A = !1,
  zoomable: w = !1,
  ariaLabel: E,
  inversePan: M,
  zoomStep: S = 1,
  offsetScale: T = 5
}) {
  const N = Ze(), R = re.useRef(null), { boundingRect: q, viewBB: L, rfId: j, panZoom: Z, translateExtent: Y, flowWidth: J, flowHeight: Q, ariaLabelConfig: D } = Oe(mH, bH), X = (t == null ? void 0 : t.width) ?? vH, z = (t == null ? void 0 : t.height) ?? pH, B = q.width / X, $ = q.height / z, U = Math.max(B, $), ae = U * X, O = U * z, G = T * U, ie = q.x - (ae - q.width) / 2 - G, H = q.y - (O - q.height) / 2 - G, K = ae + G * 2, P = O + G * 2, ue = `${_H}-${j}`, W = re.useRef(0), se = re.useRef();
  W.current = U, re.useEffect(() => {
    if (R.current && Z)
      return se.current = N6({
        domNode: R.current,
        panZoom: Z,
        getTransform: () => N.getState().transform,
        getViewScale: () => W.current
      }), () => {
        var he;
        (he = se.current) == null || he.destroy();
      };
  }, [Z]), re.useEffect(() => {
    var he;
    (he = se.current) == null || he.update({
      translateExtent: Y,
      width: J,
      height: Q,
      inversePan: M,
      pannable: A,
      zoomStep: S,
      zoomable: w
    });
  }, [A, w, M, S, Y, J, Q]);
  const de = b ? (he) => {
    var Ae;
    const [me, ge] = ((Ae = se.current) == null ? void 0 : Ae.pointer(he)) || [0, 0];
    b(he, { x: me, y: ge });
  } : void 0, ve = _ ? re.useCallback((he, me) => {
    const ge = N.getState().nodeLookup.get(me).internals.userNode;
    _(he, ge);
  }, []) : void 0, ye = E ?? D["minimap.ariaLabel"];
  return k.jsx($u, { position: m, style: {
    ...t,
    "--xy-minimap-background-color-props": typeof g == "string" ? g : void 0,
    "--xy-minimap-mask-background-color-props": typeof h == "string" ? h : void 0,
    "--xy-minimap-mask-stroke-color-props": typeof v == "string" ? v : void 0,
    "--xy-minimap-mask-stroke-width-props": typeof y == "number" ? y * U : void 0,
    "--xy-minimap-node-background-color-props": typeof u == "string" ? u : void 0,
    "--xy-minimap-node-stroke-color-props": typeof i == "string" ? i : void 0,
    "--xy-minimap-node-stroke-width-props": typeof c == "number" ? c : void 0
  }, className: ct(["react-flow__minimap", r]), "data-testid": "rf__minimap", children: k.jsxs("svg", { width: X, height: z, viewBox: `${ie} ${H} ${K} ${P}`, className: "react-flow__minimap-svg", role: "img", "aria-labelledby": ue, ref: R, onClick: de, children: [ye && k.jsx("title", { id: ue, children: ye }), k.jsx(gH, { onClick: ve, nodeColor: u, nodeStrokeColor: i, nodeBorderRadius: s, nodeClassName: l, nodeStrokeWidth: c, nodeComponent: f }), k.jsx("path", { className: "react-flow__minimap-mask", d: `M${ie - G},${H - G}h${K + G * 2}v${P + G * 2}h${-K - G * 2}z
        M${L.x},${L.y}h${L.width}v${L.height}h${-L.width}z`, fillRule: "evenodd", pointerEvents: "none" })] }) });
}
RC.displayName = "MiniMap";
re.memo(RC);
const xH = (t) => (r) => t ? `${Math.max(1 / r.transform[2], 1)}` : void 0, SH = {
  [hi.Line]: "right",
  [hi.Handle]: "bottom-right"
};
function EH({ nodeId: t, position: r, variant: i = hi.Handle, className: u, style: l = void 0, children: s, color: c, minWidth: f = 10, minHeight: g = 10, maxWidth: h = Number.MAX_VALUE, maxHeight: v = Number.MAX_VALUE, keepAspectRatio: y = !1, resizeDirection: m, autoScale: b = !0, shouldResize: _, onResizeStart: A, onResize: w, onResizeEnd: E }) {
  const M = uC(), S = typeof t == "string" ? t : M, T = Ze(), N = re.useRef(null), R = i === hi.Handle, q = Oe(re.useCallback(xH(R && b), [R, b]), $e), L = re.useRef(null), j = r ?? SH[i];
  re.useEffect(() => {
    if (!(!N.current || !S))
      return L.current || (L.current = X6({
        domNode: N.current,
        nodeId: S,
        getStoreItems: () => {
          const { nodeLookup: Y, transform: J, snapGrid: Q, snapToGrid: D, nodeOrigin: X, domNode: z } = T.getState();
          return {
            nodeLookup: Y,
            transform: J,
            snapGrid: Q,
            snapToGrid: D,
            nodeOrigin: X,
            paneDomNode: z
          };
        },
        onChange: (Y, J) => {
          const { triggerNodeChanges: Q, nodeLookup: D, parentLookup: X, nodeOrigin: z } = T.getState(), B = [], $ = { x: Y.x, y: Y.y }, U = D.get(S);
          if (U && U.expandParent && U.parentId) {
            const ae = U.origin ?? z, O = Y.width ?? U.measured.width ?? 0, G = Y.height ?? U.measured.height ?? 0, ie = {
              id: U.id,
              parentId: U.parentId,
              rect: {
                width: O,
                height: G,
                ...RM({
                  x: Y.x ?? U.position.x,
                  y: Y.y ?? U.position.y
                }, { width: O, height: G }, U.parentId, D, ae)
              }
            }, H = pm([ie], D, X, z);
            B.push(...H), $.x = Y.x ? Math.max(ae[0] * O, Y.x) : void 0, $.y = Y.y ? Math.max(ae[1] * G, Y.y) : void 0;
          }
          if ($.x !== void 0 && $.y !== void 0) {
            const ae = {
              id: S,
              type: "position",
              position: { ...$ }
            };
            B.push(ae);
          }
          if (Y.width !== void 0 && Y.height !== void 0) {
            const O = {
              id: S,
              type: "dimensions",
              resizing: !0,
              setAttributes: m ? m === "horizontal" ? "width" : "height" : !0,
              dimensions: {
                width: Y.width,
                height: Y.height
              }
            };
            B.push(O);
          }
          for (const ae of J) {
            const O = {
              ...ae,
              type: "position"
            };
            B.push(O);
          }
          Q(B);
        },
        onEnd: ({ width: Y, height: J }) => {
          const Q = {
            id: S,
            type: "dimensions",
            resizing: !1,
            dimensions: {
              width: Y,
              height: J
            }
          };
          T.getState().triggerNodeChanges([Q]);
        }
      })), L.current.update({
        controlPosition: j,
        boundaries: {
          minWidth: f,
          minHeight: g,
          maxWidth: h,
          maxHeight: v
        },
        keepAspectRatio: y,
        resizeDirection: m,
        onResizeStart: A,
        onResize: w,
        onResizeEnd: E,
        shouldResize: _
      }), () => {
        var Y;
        (Y = L.current) == null || Y.destroy();
      };
  }, [
    j,
    f,
    g,
    h,
    v,
    y,
    A,
    w,
    E,
    _
  ]);
  const Z = j.split("-");
  return k.jsx("div", { className: ct(["react-flow__resize-control", "nodrag", ...Z, i, u]), ref: N, style: {
    ...l,
    scale: q,
    ...c && { [R ? "backgroundColor" : "borderColor"]: c }
  }, children: s });
}
re.memo(EH);
function wH({ flat: t, selected: r, color: i, state: u }) {
  const l = [t.agentType, t.model, t.badge].filter(Boolean), s = [
    "nwf-node",
    `nwf-kind-${t.kind}`,
    r ? "nwf-selected" : "",
    u ? `nwf-${u}` : ""
  ].filter(Boolean).join(" ");
  return /* @__PURE__ */ k.jsxs(
    "div",
    {
      className: s,
      style: i ? { "--nwf-phase-color": i } : void 0,
      title: t.fullLabel ?? t.label,
      children: [
        t.phase ? /* @__PURE__ */ k.jsx("div", { className: "nwf-node-phase", children: t.phase }) : null,
        /* @__PURE__ */ k.jsx("div", { className: "nwf-node-label", children: t.label }),
        l.length > 0 ? /* @__PURE__ */ k.jsx("div", { className: "nwf-node-badges", children: l.map((c) => /* @__PURE__ */ k.jsx("span", { className: "nwf-node-badge", children: c }, c)) }) : null
      ]
    }
  );
}
var AH = {
  nodeWidth: 230,
  nodeHeight: 84,
  direction: "LR",
  nodeSep: 28,
  rankSep: 110
};
function TH(t, r, i = {}) {
  const { nodeWidth: u, nodeHeight: l, direction: s, nodeSep: c, rankSep: f } = { ...AH, ...i }, g = {};
  if (t.length === 0) return g;
  const h = new R2.graphlib.Graph();
  h.setGraph({ rankdir: s, nodesep: c, ranksep: f, marginx: 24, marginy: 24 }), h.setDefaultEdgeLabel(() => ({}));
  for (const v of t) h.setNode(v.id, { width: u, height: l });
  for (const v of r)
    v.source !== v.target && h.hasNode(v.source) && h.hasNode(v.target) && h.setEdge(v.source, v.target);
  R2.layout(h);
  for (const v of t) {
    const y = h.node(v.id);
    g[v.id] = y ? { x: y.x - u / 2, y: y.y - l / 2 } : { x: 0, y: 0 };
  }
  return g;
}
var Oy = 0.5, Dy = 2, qo = 80, GA = 1.2;
function MH({ empty: t }) {
  const { setViewport: r, fitView: i } = bs(), u = FL(), l = (c, f) => {
    r({ ...u, x: u.x + c, y: u.y + f }, { duration: 0 });
  }, s = (c) => {
    const f = Math.max(Oy, Math.min(Dy, u.zoom * c));
    r({ ...u, zoom: f }, { duration: 0 });
  };
  return /* @__PURE__ */ k.jsxs($u, { position: "top-left", className: "nwf-navigation", "aria-label": "Workflow navigation", children: [
    /* @__PURE__ */ k.jsx("button", { type: "button", onClick: () => l(qo, 0), disabled: t, children: "Pan left" }),
    /* @__PURE__ */ k.jsx("button", { type: "button", onClick: () => l(-qo, 0), disabled: t, children: "Pan right" }),
    /* @__PURE__ */ k.jsx("button", { type: "button", onClick: () => l(0, qo), disabled: t, children: "Pan up" }),
    /* @__PURE__ */ k.jsx("button", { type: "button", onClick: () => l(0, -qo), disabled: t, children: "Pan down" }),
    /* @__PURE__ */ k.jsx(
      "button",
      {
        type: "button",
        onClick: () => s(GA),
        disabled: t || u.zoom >= Dy,
        children: "Zoom in"
      }
    ),
    /* @__PURE__ */ k.jsx(
      "button",
      {
        type: "button",
        onClick: () => s(1 / GA),
        disabled: t || u.zoom <= Oy,
        children: "Zoom out"
      }
    ),
    /* @__PURE__ */ k.jsx(
      "button",
      {
        type: "button",
        onClick: () => {
          i({ duration: 0 });
        },
        disabled: t,
        children: "Fit workflow"
      }
    )
  ] });
}
function CH({ nodes: t, edges: r }) {
  const i = new Map(t.map((l) => [l.id, l.fullLabel ?? l.label])), u = (l) => `${i.get(l) ?? "Unknown node"} (${l})`;
  return /* @__PURE__ */ k.jsxs("details", { className: "nwf-details", children: [
    /* @__PURE__ */ k.jsx("summary", { children: "Workflow details" }),
    /* @__PURE__ */ k.jsx("h2", { children: "Nodes" }),
    t.length === 0 ? /* @__PURE__ */ k.jsx("p", { children: "No workflow nodes." }) : /* @__PURE__ */ k.jsx("ol", { children: t.map((l) => /* @__PURE__ */ k.jsxs("li", { children: [
      /* @__PURE__ */ k.jsx("strong", { children: l.fullLabel ?? l.label }),
      " (",
      l.id,
      ")",
      /* @__PURE__ */ k.jsxs("dl", { children: [
        /* @__PURE__ */ k.jsx("dt", { children: "Kind" }),
        /* @__PURE__ */ k.jsx("dd", { children: l.kind }),
        Object.entries({
          Phase: l.phase,
          "Agent type": l.agentType,
          Model: l.model,
          Isolation: l.isolation,
          Multiplicity: l.badge,
          Prompt: l.prompt,
          Source: l.source
        }).map(
          ([s, c]) => c ? /* @__PURE__ */ k.jsxs("div", { children: [
            /* @__PURE__ */ k.jsx("dt", { children: s }),
            /* @__PURE__ */ k.jsx("dd", { children: c })
          ] }, s) : null
        ),
        l.hasSchema !== void 0 ? /* @__PURE__ */ k.jsxs(k.Fragment, { children: [
          /* @__PURE__ */ k.jsx("dt", { children: "Schema" }),
          /* @__PURE__ */ k.jsx("dd", { children: l.hasSchema ? "Yes" : "No" })
        ] }) : null
      ] })
    ] }, l.id)) }),
    /* @__PURE__ */ k.jsx("h2", { children: "Connections" }),
    r.length === 0 ? /* @__PURE__ */ k.jsx("p", { children: "No workflow connections." }) : /* @__PURE__ */ k.jsx("ul", { children: r.map((l) => /* @__PURE__ */ k.jsxs("li", { children: [
      u(l.source),
      " → ",
      u(l.target),
      " (",
      l.kind,
      ")"
    ] }, l.id)) })
  ] });
}
var qH = [
  "#6ea8fe",
  "#5fd0a8",
  "#f7a072",
  "#c98bdb",
  "#e8c468",
  "#7ec9e8",
  "#e87e9e",
  "#9ee87e"
];
function RH(t) {
  const { flat: r, color: i, state: u, renderNode: l, vertical: s } = t.data, c = t.selected ?? !1;
  return /* @__PURE__ */ k.jsxs(k.Fragment, { children: [
    /* @__PURE__ */ k.jsx(
      gi,
      {
        type: "target",
        position: s ? _e.Top : _e.Left,
        isConnectable: t.isConnectable,
        isConnectableStart: t.isConnectable,
        isConnectableEnd: t.isConnectable,
        "aria-hidden": "true"
      }
    ),
    l ? /* @__PURE__ */ k.jsx("div", { className: "nwf-custom-node", children: l(r, { selected: c }) }) : /* @__PURE__ */ k.jsx(wH, { flat: r, selected: c, color: i, state: u }),
    /* @__PURE__ */ k.jsx(
      gi,
      {
        type: "source",
        position: s ? _e.Bottom : _e.Right,
        isConnectable: t.isConnectable,
        isConnectableStart: t.isConnectable,
        isConnectableEnd: t.isConnectable,
        "aria-hidden": "true"
      }
    )
  ] });
}
var NH = (t) => t === "loop" ? { strokeDasharray: "4 4" } : void 0;
function zH({
  nodes: t,
  edges: r,
  readOnly: i = !1,
  selectedId: u,
  onNodeClick: l,
  onNodeDoubleClick: s,
  renderNode: c,
  theme: f = "dark",
  phaseColors: g = qH,
  edgeStyle: h,
  nodeState: v,
  layoutOptions: y,
  verticalRender: m,
  fitView: b = !0,
  className: _,
  style: A,
  children: w
}) {
  const [E, M] = re.useState(null), S = u !== void 0, T = S ? u : i ? null : E, N = re.useId(), R = re.useId(), q = re.useRef(null);
  E !== null && (i || S || !t.some((U) => U.id === E)) && M(null), re.useEffect(() => {
    const U = q.current;
    if (!U) return;
    U.setAttribute("role", i ? "region" : "application");
    const ae = U.ownerDocument.activeElement;
    i && ae instanceof HTMLElement && (ae.classList.contains("react-flow__node") || ae.classList.contains("react-flow__edge")) && U.focus();
  }, [i]);
  const L = (y == null ? void 0 : y.direction) ?? (m ? "TB" : "LR"), j = L === "TB" || L === "BT", Z = re.useMemo(
    () => ({ ...y, direction: L }),
    [y, L]
  ), Y = re.useMemo(() => ({ nwf: RH }), []), J = re.useMemo(
    () => TH(t, r, Z),
    [t, r, Z]
  ), Q = re.useMemo(() => new Map(t.map((U) => [U.id, U])), [t]), D = re.useMemo(
    () => t.map((U) => {
      const ae = U.phaseIndex == null ? void 0 : g[U.phaseIndex % g.length];
      return {
        id: U.id,
        type: "nwf",
        position: J[U.id] ?? { x: 0, y: 0 },
        selected: U.id === T,
        draggable: !i,
        connectable: !i,
        selectable: !i,
        deletable: !i,
        focusable: !i,
        ariaLabel: [
          U.fullLabel ?? U.label,
          U.phase,
          U.agentType,
          U.model,
          U.badge,
          v == null ? void 0 : v[U.id]
        ].filter(Boolean).join(", "),
        data: { flat: U, color: ae, state: v == null ? void 0 : v[U.id], renderNode: c, vertical: j }
      };
    }),
    [t, J, T, c, g, v, j, i]
  ), X = re.useMemo(
    () => r.map((U) => {
      var ae, O, G, ie;
      return {
        id: U.id,
        source: U.source,
        target: U.target,
        type: "smoothstep",
        animated: U.kind === "loop",
        selectable: !i,
        deletable: !i,
        focusable: !i,
        reconnectable: !i,
        ariaLabel: `${((ae = Q.get(U.source)) == null ? void 0 : ae.fullLabel) ?? ((O = Q.get(U.source)) == null ? void 0 : O.label) ?? U.source} (${U.source}) to ${((G = Q.get(U.target)) == null ? void 0 : G.fullLabel) ?? ((ie = Q.get(U.target)) == null ? void 0 : ie.label) ?? U.target} (${U.target}), ${U.kind}`,
        style: (h == null ? void 0 : h(U)) ?? NH(U.kind)
      };
    }),
    [r, h, Q, i]
  ), z = re.useCallback(
    (U, ae) => {
      if (i) return;
      const O = Q.get(ae.id);
      O && (S || M(ae.id), l == null || l(O, U));
    },
    [Q, S, l, i]
  ), B = re.useCallback(
    (U, ae) => {
      if (i) return;
      const O = Q.get(ae.id);
      O && (s == null || s(O, U));
    },
    [Q, s, i]
  ), $ = [
    "nwf-graph",
    f === "dark" ? "nwf-theme-dark" : "",
    j ? "nwf-vertical" : "",
    i ? "nwf-readonly" : "",
    _ ?? ""
  ].filter(Boolean).join(" ");
  return /* @__PURE__ */ k.jsxs("div", { className: $, style: { width: "100%", height: "100%", ...A }, children: [
    /* @__PURE__ */ k.jsxs(
      QL,
      {
        ref: q,
        id: N,
        "aria-label": "Workflow graph",
        "aria-describedby": i ? R : void 0,
        tabIndex: -1,
        nodes: D,
        edges: X,
        nodeTypes: Y,
        onNodeClick: z,
        onNodeDoubleClick: B,
        fitView: b,
        minZoom: Oy,
        maxZoom: Dy,
        nodesDraggable: !i,
        nodesConnectable: !i,
        nodesFocusable: !i,
        edgesFocusable: !i,
        edgesReconnectable: !i,
        elementsSelectable: !i,
        deleteKeyCode: i ? null : "Backspace",
        selectionKeyCode: i ? null : "Shift",
        multiSelectionKeyCode: i ? null : void 0,
        disableKeyboardA11y: i,
        connectOnClick: !i,
        ariaLabelConfig: i ? {
          "node.a11yDescription.default": "",
          "node.a11yDescription.keyboardDisabled": "",
          "edge.a11yDescription.default": ""
        } : void 0,
        proOptions: { hideAttribution: !0 },
        children: [
          w ?? /* @__PURE__ */ k.jsx(tH, {}),
          i ? /* @__PURE__ */ k.jsx(MH, { empty: t.length === 0 }) : null
        ]
      }
    ),
    i ? /* @__PURE__ */ k.jsxs(k.Fragment, { children: [
      /* @__PURE__ */ k.jsx("p", { id: R, className: "nwf-navigation-hint", children: "Read-only workflow. Use the pan, zoom, and fit buttons to explore the graph, or expand Workflow details to read nodes and connections." }),
      /* @__PURE__ */ k.jsx(CH, { nodes: t, edges: r })
    ] }) : null
  ] });
}
function OH(t) {
  return /* @__PURE__ */ k.jsx(MC, { children: /* @__PURE__ */ k.jsx(zH, { ...t }) });
}
const DH = [
  "--color-cornflower",
  "--color-teal",
  "--color-salmon",
  "--color-coral",
  "--color-green-l2"
];
function VA() {
  return typeof document > "u" ? "light" : document.documentElement.getAttribute("data-theme") === "dark" ? "dark" : "light";
}
function NC() {
  const [t, r] = re.useState(VA);
  return re.useEffect(() => {
    const i = new MutationObserver(() => r(VA()));
    return i.observe(document.documentElement, {
      attributes: !0,
      attributeFilter: ["data-theme"]
    }), () => i.disconnect();
  }, []), t;
}
function LH() {
  if (NC(), typeof document > "u")
    return [];
  const t = getComputedStyle(document.documentElement);
  return DH.map((r) => t.getPropertyValue(r).trim()).filter(Boolean);
}
const HH = /* @__PURE__ */ new Set([
  "task",
  "classifier",
  "rag",
  "delegator",
  "mcp",
  "multi_model",
  "multi_prompt",
  "matrixed",
  "pipeline"
]);
function zC(t) {
  return t.type ?? "task";
}
function jH(t) {
  var r;
  return ((r = t.display_name) == null ? void 0 : r.trim()) || t.name;
}
function BH(t) {
  const r = new Map(t.map((s) => [s.name, s])), i = [], u = /* @__PURE__ */ new Set();
  let l = t[0];
  for (; l && !u.has(l.name); )
    u.add(l.name), i.push(l), l = l.next_step ? r.get(l.next_step) : void 0;
  for (const s of t)
    u.has(s.name) || (u.add(s.name), i.push(s));
  return i;
}
function UH(t) {
  const r = t.review_type;
  return r && r !== "none" ? `${r} review` : void 0;
}
function GH(t) {
  var s, c, f, g, h, v, y, m;
  const r = [], i = [], u = zC(t), l = (b, _, A) => {
    for (const [E, M] of b.entries()) {
      const S = `${t.name}::${E}`;
      r.push({ id: S, kind: "agent", label: M, phase: t.name }), i.push({ id: `${t.name}->${S}`, source: t.name, target: S, kind: _ });
    }
    const w = `${t.name}::aggregate`;
    r.push({
      id: w,
      kind: "workflow",
      label: A,
      phase: t.name,
      badge: `×${b.length}`
    });
    for (const [E] of b.entries()) {
      const M = `${t.name}::${E}`;
      i.push({ id: `${M}->${w}`, source: M, target: w, kind: "seq" });
    }
    return w;
  };
  switch (u) {
    case "multi_model": {
      const b = ((s = t.multi_model_config) == null ? void 0 : s.delegators) ?? [];
      if (!b.length)
        break;
      const _ = ((c = t.multi_model_config) == null ? void 0 : c.voting_strategy) ?? "vote";
      return { nodes: r, edges: i, exit: l(b, "parallel", `vote: ${_}`) };
    }
    case "multi_prompt": {
      const b = ((f = t.multi_prompt_config) == null ? void 0 : f.prompt_variations) ?? [];
      if (!b.length)
        break;
      const _ = ((g = t.multi_prompt_config) == null ? void 0 : g.selection_strategy) ?? "select", A = b.map((w, E) => `variation ${E + 1}`);
      return { nodes: r, edges: i, exit: l(A, "parallel", `select: ${_}`) };
    }
    case "matrixed": {
      const b = ((h = t.matrixed_config) == null ? void 0 : h.delegators) ?? [], _ = ((v = t.matrixed_config) == null ? void 0 : v.prompt_variations) ?? [];
      if (!b.length || !_.length)
        break;
      const A = b.flatMap(
        (w) => _.map((E, M) => `${w} · variation ${M + 1}`)
      );
      return { nodes: r, edges: i, exit: l(A, "parallel", "matrix aggregate") };
    }
    case "pipeline": {
      const b = ((y = t.pipeline_config) == null ? void 0 : y.stages) ?? [];
      if (!b.length)
        break;
      let _ = t.name;
      for (const [A, w] of b.entries()) {
        const E = `${t.name}::stage${A}`;
        r.push({
          id: E,
          kind: "agent",
          label: ((m = w.label) == null ? void 0 : m.trim()) || `stage ${A + 1}`,
          phase: t.name,
          model: w.model ?? void 0,
          agentType: w.agent ?? void 0
        }), i.push({ id: `${_}->${E}`, source: _, target: E, kind: "pipeline" }), _ = E;
      }
      return { nodes: r, edges: i, exit: _ };
    }
    // Single-node step types: they run one session and contribute no sub-nodes.
    // Listed explicitly rather than swept up by `default` so the `never` check
    // below turns a new StepTypeTag variant in Rust into a compile error here,
    // forcing a decision about how it should be drawn.
    case "task":
    case "classifier":
    case "rag":
    case "delegator":
    case "mcp":
      break;
    default: {
      const b = u;
      throw new Error(`unhandled step type: ${String(b)}`);
    }
  }
  return { nodes: r, edges: i, exit: t.name };
}
function VH(t) {
  var c, f;
  const r = BH(t.steps ?? []), i = [], u = [], l = /* @__PURE__ */ new Map();
  for (const g of r) {
    const h = zC(g);
    i.push({
      id: g.name,
      kind: HH.has(h) ? "agent" : "note",
      label: jH(g),
      phase: g.name,
      prompt: g.prompt ?? void 0,
      agentType: g.agent ?? ((c = g.delegator_config) == null ? void 0 : c.delegator) ?? void 0,
      badge: UH(g),
      source: h
    });
    const v = GH(g);
    i.push(...v.nodes), u.push(...v.edges), l.set(g.name, v.exit);
  }
  for (let g = 0; g < r.length - 1; g += 1) {
    const h = l.get(r[g].name) ?? r[g].name, v = r[g + 1].name;
    u.push({ id: `${h}->${v}`, source: h, target: v, kind: "seq" });
  }
  const s = new Set(r.map((g) => g.name));
  for (const g of r) {
    const h = (f = g.on_reject) == null ? void 0 : f.goto_step;
    !h || !s.has(h) || u.push({
      id: `${g.name}~reject~${h}`,
      source: g.name,
      target: h,
      kind: "loop"
    });
  }
  return { nodes: i, edges: u };
}
function YH({
  issueType: t,
  height: r = 520,
  vertical: i = !1,
  className: u
}) {
  const l = NC(), s = LH(), { nodes: c, edges: f } = re.useMemo(() => VH(t), [t]), g = re.useMemo(() => ({ height: r }), [r]);
  return c.length ? /* @__PURE__ */ k.jsx("div", { className: u ?? "operator-workflow-canvas", style: g, children: /* @__PURE__ */ k.jsx(
    OH,
    {
      nodes: c,
      edges: f,
      theme: l,
      phaseColors: s,
      verticalRender: i,
      fitView: !0
    },
    t.key
  ) }) : /* @__PURE__ */ k.jsx("div", { className: "operator-workflow-empty", children: "This issue type defines no steps." });
}
function kH({
  entries: t,
  selected: r,
  document: i,
  error: u,
  onSelect: l
}) {
  return u ? /* @__PURE__ */ k.jsxs("div", { className: "workflow-explorer-error", children: [
    "Could not load this collection: ",
    u
  ] }) : t ? t.length === 0 ? /* @__PURE__ */ k.jsx("div", { className: "workflow-explorer-loading", children: "This collection defines no issue types." }) : /* @__PURE__ */ k.jsxs("div", { className: "workflow-explorer-body", children: [
    /* @__PURE__ */ k.jsx("nav", { className: "workflow-explorer-rail", "aria-label": "Issue types", children: /* @__PURE__ */ k.jsx("ul", { children: t.map((s) => /* @__PURE__ */ k.jsx("li", { children: /* @__PURE__ */ k.jsx(XH, { entry: s, selected: s.key === r, onSelect: l }) }, s.key)) }) }),
    /* @__PURE__ */ k.jsx("div", { className: "workflow-explorer-canvas", children: i ? /* @__PURE__ */ k.jsxs(k.Fragment, { children: [
      /* @__PURE__ */ k.jsxs("h3", { className: "workflow-explorer-title", children: [
        i.name,
        " ",
        /* @__PURE__ */ k.jsx("code", { children: i.key })
      ] }),
      i.description && /* @__PURE__ */ k.jsx("p", { className: "workflow-explorer-description", children: i.description }),
      /* @__PURE__ */ k.jsx(YH, { issueType: i, vertical: !0 })
    ] }) : /* @__PURE__ */ k.jsx("div", { className: "workflow-explorer-loading", children: "Loading workflow..." }) })
  ] }) : /* @__PURE__ */ k.jsx("div", { className: "workflow-explorer-loading", children: "Loading collection..." });
}
function XH({
  entry: t,
  selected: r,
  onSelect: i
}) {
  return /* @__PURE__ */ k.jsx(
    "button",
    {
      type: "button",
      "aria-current": r,
      className: r ? "is-selected" : void 0,
      onClick: () => i(t.key),
      children: t.key
    }
  );
}
async function YA(t) {
  const r = await fetch(t);
  if (!r.ok)
    throw new Error(`${r.status} ${r.statusText} for ${t}`);
  return await r.json();
}
function IH({ base: t, initial: r }) {
  const [i, u] = re.useState(null), [l, s] = re.useState(r ?? null), [c, f] = re.useState(null), [g, h] = re.useState(null);
  re.useEffect(() => {
    let m = !1;
    return YA(`${t}collection.json`).then((b) => {
      if (!m) {
        const _ = b.issue_types ?? [];
        u(_), s((A) => {
          var w;
          return A ?? ((w = _[0]) == null ? void 0 : w.key) ?? null;
        });
      }
    }).catch((b) => !m && h(b.message)), () => {
      m = !0;
    };
  }, [t]), re.useEffect(() => {
    if (!i || !l)
      return;
    const m = i.find((_) => _.key === l);
    if (!m)
      return;
    let b = !1;
    return YA(`${t}${m.schema_path}`).then((_) => {
      b || f(_);
    }).catch((_) => !b && h(_.message)), () => {
      b = !0;
    };
  }, [t, i, l]);
  const v = re.useCallback((m) => {
    h(null), s(m);
  }, []), y = (c == null ? void 0 : c.key) === l ? c : null;
  return /* @__PURE__ */ k.jsx(
    kH,
    {
      entries: i,
      selected: l,
      document: y,
      error: g,
      onSelect: v
    }
  );
}
class ZH extends HTMLElement {
  constructor() {
    super(...arguments);
    Hf(this, "root");
  }
  connectedCallback() {
    if (this.root)
      return;
    const i = this.getAttribute("base");
    if (!i) {
      this.textContent = 'operator-workflow-explorer: missing required "base" attribute.';
      return;
    }
    this.root = PR.createRoot(this), this.root.render(
      /* @__PURE__ */ k.jsx(re.StrictMode, { children: /* @__PURE__ */ k.jsx(
        IH,
        {
          base: i.endsWith("/") ? i : `${i}/`,
          initial: this.getAttribute("selected")
        }
      ) })
    );
  }
  disconnectedCallback() {
    const i = this.root;
    this.root = void 0, queueMicrotask(() => i == null ? void 0 : i.unmount());
  }
}
const KH = "operator-workflow-explorer";
function OC(t, r) {
  customElements.get(t) || customElements.define(t, r);
}
OC(KH, ZH);
OC(YR, VR);
export {
  VR as OperatorCollectionSearch,
  ZH as OperatorWorkflowExplorer
};
