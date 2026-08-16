var HR = Object.defineProperty;
var LR = (t, r, i) => r in t ? HR(t, r, { enumerable: !0, configurable: !0, writable: !0, value: i }) : t[r] = i;
var Df = (t, r, i) => LR(t, typeof r != "symbol" ? r + "" : r, i);
const Fb = "data-search";
class BR extends HTMLElement {
  constructor() {
    super(...arguments);
    Df(this, "input");
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
    u.type = "search", u.placeholder = this.getAttribute("placeholder") ?? "Filter by name, author, tag, loop shape, issue type…", u.setAttribute("aria-label", "Filter collections"), u.addEventListener("input", () => this.applyFilter(u.value)), this.input = u;
    const o = document.createElement("button");
    o.type = "button", o.className = "collection-view-toggle", o.addEventListener("click", () => this.toggleView(o));
    const s = document.createElement("span");
    s.className = "collection-search-count", s.setAttribute("role", "status"), i.append(u, o, s), this.append(i), this.syncToggleLabel(o), this.applyFilter("");
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
    var o;
    const u = this.catalog;
    u && (u.dataset.view = this.currentView() === "cards" ? "table" : "cards", this.syncToggleLabel(i), this.applyFilter(((o = this.input) == null ? void 0 : o.value) ?? ""));
  }
  applyFilter(i) {
    const u = this.catalog;
    if (!u) return;
    const o = i.toLowerCase().split(/\s+/).filter(Boolean), s = u.querySelector(
      `[data-view-target="${this.currentView()}"]`
    );
    let c = 0, f = 0;
    for (const h of u.querySelectorAll(`[${Fb}]`)) {
      const v = (h.getAttribute(Fb) ?? "").toLowerCase(), p = o.every((m) => v.includes(m));
      h.hidden = !p, s != null && s.contains(h) && (f += 1, p && (c += 1));
    }
    const g = this.querySelector(".collection-search-count");
    g && (g.textContent = c === f ? `${f} collections` : `${c} of ${f} collections`), u.dataset.empty = c === 0 ? "true" : "false";
  }
}
const jR = "operator-collection-search";
var mo = typeof globalThis < "u" ? globalThis : typeof window < "u" ? window : typeof global < "u" ? global : typeof self < "u" ? self : {};
function Np(t) {
  return t && t.__esModule && Object.prototype.hasOwnProperty.call(t, "default") ? t.default : t;
}
var Hf = { exports: {} }, pu = {};
/**
 * @license React
 * react-jsx-runtime.production.js
 *
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
var Jb;
function UR() {
  if (Jb) return pu;
  Jb = 1;
  var t = Symbol.for("react.transitional.element"), r = Symbol.for("react.fragment");
  function i(u, o, s) {
    var c = null;
    if (s !== void 0 && (c = "" + s), o.key !== void 0 && (c = "" + o.key), "key" in o) {
      s = {};
      for (var f in o)
        f !== "key" && (s[f] = o[f]);
    } else s = o;
    return o = s.ref, {
      $$typeof: t,
      type: u,
      key: c,
      ref: o !== void 0 ? o : null,
      props: s
    };
  }
  return pu.Fragment = r, pu.jsx = i, pu.jsxs = i, pu;
}
var Pb;
function GR() {
  return Pb || (Pb = 1, Hf.exports = UR()), Hf.exports;
}
var J = GR(), Lf = { exports: {} }, we = {};
/**
 * @license React
 * react.production.js
 *
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
var Wb;
function VR() {
  if (Wb) return we;
  Wb = 1;
  var t = Symbol.for("react.transitional.element"), r = Symbol.for("react.portal"), i = Symbol.for("react.fragment"), u = Symbol.for("react.strict_mode"), o = Symbol.for("react.profiler"), s = Symbol.for("react.consumer"), c = Symbol.for("react.context"), f = Symbol.for("react.forward_ref"), g = Symbol.for("react.suspense"), h = Symbol.for("react.memo"), v = Symbol.for("react.lazy"), p = Symbol.for("react.activity"), m = Symbol.iterator;
  function b(z) {
    return z === null || typeof z != "object" ? null : (z = m && z[m] || z["@@iterator"], typeof z == "function" ? z : null);
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
  function E(z, V, ie) {
    this.props = z, this.context = V, this.refs = w, this.updater = ie || _;
  }
  E.prototype.isReactComponent = {}, E.prototype.setState = function(z, V) {
    if (typeof z != "object" && typeof z != "function" && z != null)
      throw Error(
        "takes an object of state variables to update or a function which returns an object of state variables."
      );
    this.updater.enqueueSetState(this, z, V, "setState");
  }, E.prototype.forceUpdate = function(z) {
    this.updater.enqueueForceUpdate(this, z, "forceUpdate");
  };
  function M() {
  }
  M.prototype = E.prototype;
  function S(z, V, ie) {
    this.props = z, this.context = V, this.refs = w, this.updater = ie || _;
  }
  var T = S.prototype = new M();
  T.constructor = S, A(T, E.prototype), T.isPureReactComponent = !0;
  var O = Array.isArray;
  function C() {
  }
  var R = { H: null, A: null, T: null, S: null }, H = Object.prototype.hasOwnProperty;
  function B(z, V, ie) {
    var L = ie.ref;
    return {
      $$typeof: t,
      type: z,
      key: V,
      ref: L !== void 0 ? L : null,
      props: ie
    };
  }
  function X(z, V) {
    return B(z.type, V, z.props);
  }
  function Y(z) {
    return typeof z == "object" && z !== null && z.$$typeof === t;
  }
  function F(z) {
    var V = { "=": "=0", ":": "=2" };
    return "$" + z.replace(/[=:]/g, function(ie) {
      return V[ie];
    });
  }
  var K = /\/+/g;
  function D(z, V) {
    return typeof z == "object" && z !== null && z.key != null ? F("" + z.key) : V.toString(36);
  }
  function G(z) {
    switch (z.status) {
      case "fulfilled":
        return z.value;
      case "rejected":
        throw z.reason;
      default:
        switch (typeof z.status == "string" ? z.then(C, C) : (z.status = "pending", z.then(
          function(V) {
            z.status === "pending" && (z.status = "fulfilled", z.value = V);
          },
          function(V) {
            z.status === "pending" && (z.status = "rejected", z.reason = V);
          }
        )), z.status) {
          case "fulfilled":
            return z.value;
          case "rejected":
            throw z.reason;
        }
    }
    throw z;
  }
  function N(z, V, ie, L, I) {
    var P = typeof z;
    (P === "undefined" || P === "boolean") && (z = null);
    var ae = !1;
    if (z === null) ae = !0;
    else
      switch (P) {
        case "bigint":
        case "string":
        case "number":
          ae = !0;
          break;
        case "object":
          switch (z.$$typeof) {
            case t:
            case r:
              ae = !0;
              break;
            case v:
              return ae = z._init, N(
                ae(z._payload),
                V,
                ie,
                L,
                I
              );
          }
      }
    if (ae)
      return I = I(z), ae = L === "" ? "." + D(z, 0) : L, O(I) ? (ie = "", ae != null && (ie = ae.replace(K, "$&/") + "/"), N(I, V, ie, "", function(de) {
        return de;
      })) : I != null && (Y(I) && (I = X(
        I,
        ie + (I.key == null || z && z.key === I.key ? "" : ("" + I.key).replace(
          K,
          "$&/"
        ) + "/") + ae
      )), V.push(I)), 1;
    ae = 0;
    var W = L === "" ? "." : L + ":";
    if (O(z))
      for (var se = 0; se < z.length; se++)
        L = z[se], P = W + D(L, se), ae += N(
          L,
          V,
          ie,
          P,
          I
        );
    else if (se = b(z), typeof se == "function")
      for (z = se.call(z), se = 0; !(L = z.next()).done; )
        L = L.value, P = W + D(L, se++), ae += N(
          L,
          V,
          ie,
          P,
          I
        );
    else if (P === "object") {
      if (typeof z.then == "function")
        return N(
          G(z),
          V,
          ie,
          L,
          I
        );
      throw V = String(z), Error(
        "Objects are not valid as a React child (found: " + (V === "[object Object]" ? "object with keys {" + Object.keys(z).join(", ") + "}" : V) + "). If you meant to render a collection of children, use an array instead."
      );
    }
    return ae;
  }
  function j(z, V, ie) {
    if (z == null) return z;
    var L = [], I = 0;
    return N(z, L, "", "", function(P) {
      return V.call(ie, P, I++);
    }), L;
  }
  function Z(z) {
    if (z._status === -1) {
      var V = z._result;
      V = V(), V.then(
        function(ie) {
          (z._status === 0 || z._status === -1) && (z._status = 1, z._result = ie);
        },
        function(ie) {
          (z._status === 0 || z._status === -1) && (z._status = 2, z._result = ie);
        }
      ), z._status === -1 && (z._status = 0, z._result = V);
    }
    if (z._status === 1) return z._result.default;
    throw z._result;
  }
  var Q = typeof reportError == "function" ? reportError : function(z) {
    if (typeof window == "object" && typeof window.ErrorEvent == "function") {
      var V = new window.ErrorEvent("error", {
        bubbles: !0,
        cancelable: !0,
        message: typeof z == "object" && z !== null && typeof z.message == "string" ? String(z.message) : String(z),
        error: z
      });
      if (!window.dispatchEvent(V)) return;
    } else if (typeof process == "object" && typeof process.emit == "function") {
      process.emit("uncaughtException", z);
      return;
    }
    console.error(z);
  }, le = {
    map: j,
    forEach: function(z, V, ie) {
      j(
        z,
        function() {
          V.apply(this, arguments);
        },
        ie
      );
    },
    count: function(z) {
      var V = 0;
      return j(z, function() {
        V++;
      }), V;
    },
    toArray: function(z) {
      return j(z, function(V) {
        return V;
      }) || [];
    },
    only: function(z) {
      if (!Y(z))
        throw Error(
          "React.Children.only expected to receive a single React element child."
        );
      return z;
    }
  };
  return we.Activity = p, we.Children = le, we.Component = E, we.Fragment = i, we.Profiler = o, we.PureComponent = S, we.StrictMode = u, we.Suspense = g, we.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE = R, we.__COMPILER_RUNTIME = {
    __proto__: null,
    c: function(z) {
      return R.H.useMemoCache(z);
    }
  }, we.cache = function(z) {
    return function() {
      return z.apply(null, arguments);
    };
  }, we.cacheSignal = function() {
    return null;
  }, we.cloneElement = function(z, V, ie) {
    if (z == null)
      throw Error(
        "The argument must be a React element, but you passed " + z + "."
      );
    var L = A({}, z.props), I = z.key;
    if (V != null)
      for (P in V.key !== void 0 && (I = "" + V.key), V)
        !H.call(V, P) || P === "key" || P === "__self" || P === "__source" || P === "ref" && V.ref === void 0 || (L[P] = V[P]);
    var P = arguments.length - 2;
    if (P === 1) L.children = ie;
    else if (1 < P) {
      for (var ae = Array(P), W = 0; W < P; W++)
        ae[W] = arguments[W + 2];
      L.children = ae;
    }
    return B(z.type, I, L);
  }, we.createContext = function(z) {
    return z = {
      $$typeof: c,
      _currentValue: z,
      _currentValue2: z,
      _threadCount: 0,
      Provider: null,
      Consumer: null
    }, z.Provider = z, z.Consumer = {
      $$typeof: s,
      _context: z
    }, z;
  }, we.createElement = function(z, V, ie) {
    var L, I = {}, P = null;
    if (V != null)
      for (L in V.key !== void 0 && (P = "" + V.key), V)
        H.call(V, L) && L !== "key" && L !== "__self" && L !== "__source" && (I[L] = V[L]);
    var ae = arguments.length - 2;
    if (ae === 1) I.children = ie;
    else if (1 < ae) {
      for (var W = Array(ae), se = 0; se < ae; se++)
        W[se] = arguments[se + 2];
      I.children = W;
    }
    if (z && z.defaultProps)
      for (L in ae = z.defaultProps, ae)
        I[L] === void 0 && (I[L] = ae[L]);
    return B(z, P, I);
  }, we.createRef = function() {
    return { current: null };
  }, we.forwardRef = function(z) {
    return { $$typeof: f, render: z };
  }, we.isValidElement = Y, we.lazy = function(z) {
    return {
      $$typeof: v,
      _payload: { _status: -1, _result: z },
      _init: Z
    };
  }, we.memo = function(z, V) {
    return {
      $$typeof: h,
      type: z,
      compare: V === void 0 ? null : V
    };
  }, we.startTransition = function(z) {
    var V = R.T, ie = {};
    R.T = ie;
    try {
      var L = z(), I = R.S;
      I !== null && I(ie, L), typeof L == "object" && L !== null && typeof L.then == "function" && L.then(C, Q);
    } catch (P) {
      Q(P);
    } finally {
      V !== null && ie.types !== null && (V.types = ie.types), R.T = V;
    }
  }, we.unstable_useCacheRefresh = function() {
    return R.H.useCacheRefresh();
  }, we.use = function(z) {
    return R.H.use(z);
  }, we.useActionState = function(z, V, ie) {
    return R.H.useActionState(z, V, ie);
  }, we.useCallback = function(z, V) {
    return R.H.useCallback(z, V);
  }, we.useContext = function(z) {
    return R.H.useContext(z);
  }, we.useDebugValue = function() {
  }, we.useDeferredValue = function(z, V) {
    return R.H.useDeferredValue(z, V);
  }, we.useEffect = function(z, V) {
    return R.H.useEffect(z, V);
  }, we.useEffectEvent = function(z) {
    return R.H.useEffectEvent(z);
  }, we.useId = function() {
    return R.H.useId();
  }, we.useImperativeHandle = function(z, V, ie) {
    return R.H.useImperativeHandle(z, V, ie);
  }, we.useInsertionEffect = function(z, V) {
    return R.H.useInsertionEffect(z, V);
  }, we.useLayoutEffect = function(z, V) {
    return R.H.useLayoutEffect(z, V);
  }, we.useMemo = function(z, V) {
    return R.H.useMemo(z, V);
  }, we.useOptimistic = function(z, V) {
    return R.H.useOptimistic(z, V);
  }, we.useReducer = function(z, V, ie) {
    return R.H.useReducer(z, V, ie);
  }, we.useRef = function(z) {
    return R.H.useRef(z);
  }, we.useState = function(z) {
    return R.H.useState(z);
  }, we.useSyncExternalStore = function(z, V, ie) {
    return R.H.useSyncExternalStore(
      z,
      V,
      ie
    );
  }, we.useTransition = function() {
    return R.H.useTransition();
  }, we.version = "19.2.8", we;
}
var e_;
function Bu() {
  return e_ || (e_ = 1, Lf.exports = VR()), Lf.exports;
}
var re = Bu();
const YR = /* @__PURE__ */ Np(re);
var Bf = { exports: {} }, mu = {}, jf = { exports: {} }, Uf = {};
/**
 * @license React
 * scheduler.production.js
 *
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
var t_;
function kR() {
  return t_ || (t_ = 1, (function(t) {
    function r(N, j) {
      var Z = N.length;
      N.push(j);
      e: for (; 0 < Z; ) {
        var Q = Z - 1 >>> 1, le = N[Q];
        if (0 < o(le, j))
          N[Q] = j, N[Z] = le, Z = Q;
        else break e;
      }
    }
    function i(N) {
      return N.length === 0 ? null : N[0];
    }
    function u(N) {
      if (N.length === 0) return null;
      var j = N[0], Z = N.pop();
      if (Z !== j) {
        N[0] = Z;
        e: for (var Q = 0, le = N.length, z = le >>> 1; Q < z; ) {
          var V = 2 * (Q + 1) - 1, ie = N[V], L = V + 1, I = N[L];
          if (0 > o(ie, Z))
            L < le && 0 > o(I, ie) ? (N[Q] = I, N[L] = Z, Q = L) : (N[Q] = ie, N[V] = Z, Q = V);
          else if (L < le && 0 > o(I, Z))
            N[Q] = I, N[L] = Z, Q = L;
          else break e;
        }
      }
      return j;
    }
    function o(N, j) {
      var Z = N.sortIndex - j.sortIndex;
      return Z !== 0 ? Z : N.id - j.id;
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
    var g = [], h = [], v = 1, p = null, m = 3, b = !1, _ = !1, A = !1, w = !1, E = typeof setTimeout == "function" ? setTimeout : null, M = typeof clearTimeout == "function" ? clearTimeout : null, S = typeof setImmediate < "u" ? setImmediate : null;
    function T(N) {
      for (var j = i(h); j !== null; ) {
        if (j.callback === null) u(h);
        else if (j.startTime <= N)
          u(h), j.sortIndex = j.expirationTime, r(g, j);
        else break;
        j = i(h);
      }
    }
    function O(N) {
      if (A = !1, T(N), !_)
        if (i(g) !== null)
          _ = !0, C || (C = !0, F());
        else {
          var j = i(h);
          j !== null && G(O, j.startTime - N);
        }
    }
    var C = !1, R = -1, H = 5, B = -1;
    function X() {
      return w ? !0 : !(t.unstable_now() - B < H);
    }
    function Y() {
      if (w = !1, C) {
        var N = t.unstable_now();
        B = N;
        var j = !0;
        try {
          e: {
            _ = !1, A && (A = !1, M(R), R = -1), b = !0;
            var Z = m;
            try {
              t: {
                for (T(N), p = i(g); p !== null && !(p.expirationTime > N && X()); ) {
                  var Q = p.callback;
                  if (typeof Q == "function") {
                    p.callback = null, m = p.priorityLevel;
                    var le = Q(
                      p.expirationTime <= N
                    );
                    if (N = t.unstable_now(), typeof le == "function") {
                      p.callback = le, T(N), j = !0;
                      break t;
                    }
                    p === i(g) && u(g), T(N);
                  } else u(g);
                  p = i(g);
                }
                if (p !== null) j = !0;
                else {
                  var z = i(h);
                  z !== null && G(
                    O,
                    z.startTime - N
                  ), j = !1;
                }
              }
              break e;
            } finally {
              p = null, m = Z, b = !1;
            }
            j = void 0;
          }
        } finally {
          j ? F() : C = !1;
        }
      }
    }
    var F;
    if (typeof S == "function")
      F = function() {
        S(Y);
      };
    else if (typeof MessageChannel < "u") {
      var K = new MessageChannel(), D = K.port2;
      K.port1.onmessage = Y, F = function() {
        D.postMessage(null);
      };
    } else
      F = function() {
        E(Y, 0);
      };
    function G(N, j) {
      R = E(function() {
        N(t.unstable_now());
      }, j);
    }
    t.unstable_IdlePriority = 5, t.unstable_ImmediatePriority = 1, t.unstable_LowPriority = 4, t.unstable_NormalPriority = 3, t.unstable_Profiling = null, t.unstable_UserBlockingPriority = 2, t.unstable_cancelCallback = function(N) {
      N.callback = null;
    }, t.unstable_forceFrameRate = function(N) {
      0 > N || 125 < N ? console.error(
        "forceFrameRate takes a positive int between 0 and 125, forcing frame rates higher than 125 fps is not supported"
      ) : H = 0 < N ? Math.floor(1e3 / N) : 5;
    }, t.unstable_getCurrentPriorityLevel = function() {
      return m;
    }, t.unstable_next = function(N) {
      switch (m) {
        case 1:
        case 2:
        case 3:
          var j = 3;
          break;
        default:
          j = m;
      }
      var Z = m;
      m = j;
      try {
        return N();
      } finally {
        m = Z;
      }
    }, t.unstable_requestPaint = function() {
      w = !0;
    }, t.unstable_runWithPriority = function(N, j) {
      switch (N) {
        case 1:
        case 2:
        case 3:
        case 4:
        case 5:
          break;
        default:
          N = 3;
      }
      var Z = m;
      m = N;
      try {
        return j();
      } finally {
        m = Z;
      }
    }, t.unstable_scheduleCallback = function(N, j, Z) {
      var Q = t.unstable_now();
      switch (typeof Z == "object" && Z !== null ? (Z = Z.delay, Z = typeof Z == "number" && 0 < Z ? Q + Z : Q) : Z = Q, N) {
        case 1:
          var le = -1;
          break;
        case 2:
          le = 250;
          break;
        case 5:
          le = 1073741823;
          break;
        case 4:
          le = 1e4;
          break;
        default:
          le = 5e3;
      }
      return le = Z + le, N = {
        id: v++,
        callback: j,
        priorityLevel: N,
        startTime: Z,
        expirationTime: le,
        sortIndex: -1
      }, Z > Q ? (N.sortIndex = Z, r(h, N), i(g) === null && N === i(h) && (A ? (M(R), R = -1) : A = !0, G(O, Z - Q))) : (N.sortIndex = le, r(g, N), _ || b || (_ = !0, C || (C = !0, F()))), N;
    }, t.unstable_shouldYield = X, t.unstable_wrapCallback = function(N) {
      var j = m;
      return function() {
        var Z = m;
        m = j;
        try {
          return N.apply(this, arguments);
        } finally {
          m = Z;
        }
      };
    };
  })(Uf)), Uf;
}
var n_;
function XR() {
  return n_ || (n_ = 1, jf.exports = kR()), jf.exports;
}
var Gf = { exports: {} }, At = {};
/**
 * @license React
 * react-dom.production.js
 *
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
var r_;
function IR() {
  if (r_) return At;
  r_ = 1;
  var t = Bu();
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
  }, o = Symbol.for("react.portal");
  function s(g, h, v) {
    var p = 3 < arguments.length && arguments[3] !== void 0 ? arguments[3] : null;
    return {
      $$typeof: o,
      key: p == null ? null : "" + p,
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
      var v = h.as, p = f(v, h.crossOrigin), m = typeof h.integrity == "string" ? h.integrity : void 0, b = typeof h.fetchPriority == "string" ? h.fetchPriority : void 0;
      v === "style" ? u.d.S(
        g,
        typeof h.precedence == "string" ? h.precedence : void 0,
        {
          crossOrigin: p,
          integrity: m,
          fetchPriority: b
        }
      ) : v === "script" && u.d.X(g, {
        crossOrigin: p,
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
      var v = h.as, p = f(v, h.crossOrigin);
      u.d.L(g, v, {
        crossOrigin: p,
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
var a_;
function UA() {
  if (a_) return Gf.exports;
  a_ = 1;
  function t() {
    if (!(typeof __REACT_DEVTOOLS_GLOBAL_HOOK__ > "u" || typeof __REACT_DEVTOOLS_GLOBAL_HOOK__.checkDCE != "function"))
      try {
        __REACT_DEVTOOLS_GLOBAL_HOOK__.checkDCE(t);
      } catch (r) {
        console.error(r);
      }
  }
  return t(), Gf.exports = IR(), Gf.exports;
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
var i_;
function QR() {
  if (i_) return mu;
  i_ = 1;
  var t = XR(), r = Bu(), i = UA();
  function u(e) {
    var n = "https://react.dev/errors/" + e;
    if (1 < arguments.length) {
      n += "?args[]=" + encodeURIComponent(arguments[1]);
      for (var a = 2; a < arguments.length; a++)
        n += "&args[]=" + encodeURIComponent(arguments[a]);
    }
    return "Minified React error #" + e + "; visit " + n + " for the full message or use the non-minified dev environment for full errors and additional helpful warnings.";
  }
  function o(e) {
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
    for (var a = e, l = n; ; ) {
      var d = a.return;
      if (d === null) break;
      var y = d.alternate;
      if (y === null) {
        if (l = d.return, l !== null) {
          a = l;
          continue;
        }
        break;
      }
      if (d.child === y.child) {
        for (y = d.child; y; ) {
          if (y === a) return g(d), e;
          if (y === l) return g(d), n;
          y = y.sibling;
        }
        throw Error(u(188));
      }
      if (a.return !== l.return) a = d, l = y;
      else {
        for (var x = !1, q = d.child; q; ) {
          if (q === a) {
            x = !0, a = d, l = y;
            break;
          }
          if (q === l) {
            x = !0, l = d, a = y;
            break;
          }
          q = q.sibling;
        }
        if (!x) {
          for (q = y.child; q; ) {
            if (q === a) {
              x = !0, a = y, l = d;
              break;
            }
            if (q === l) {
              x = !0, l = y, a = d;
              break;
            }
            q = q.sibling;
          }
          if (!x) throw Error(u(189));
        }
      }
      if (a.alternate !== l) throw Error(u(190));
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
  var p = Object.assign, m = Symbol.for("react.element"), b = Symbol.for("react.transitional.element"), _ = Symbol.for("react.portal"), A = Symbol.for("react.fragment"), w = Symbol.for("react.strict_mode"), E = Symbol.for("react.profiler"), M = Symbol.for("react.consumer"), S = Symbol.for("react.context"), T = Symbol.for("react.forward_ref"), O = Symbol.for("react.suspense"), C = Symbol.for("react.suspense_list"), R = Symbol.for("react.memo"), H = Symbol.for("react.lazy"), B = Symbol.for("react.activity"), X = Symbol.for("react.memo_cache_sentinel"), Y = Symbol.iterator;
  function F(e) {
    return e === null || typeof e != "object" ? null : (e = Y && e[Y] || e["@@iterator"], typeof e == "function" ? e : null);
  }
  var K = Symbol.for("react.client.reference");
  function D(e) {
    if (e == null) return null;
    if (typeof e == "function")
      return e.$$typeof === K ? null : e.displayName || e.name || null;
    if (typeof e == "string") return e;
    switch (e) {
      case A:
        return "Fragment";
      case E:
        return "Profiler";
      case w:
        return "StrictMode";
      case O:
        return "Suspense";
      case C:
        return "SuspenseList";
      case B:
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
        case R:
          return n = e.displayName || null, n !== null ? n : D(e.type) || "Memo";
        case H:
          n = e._payload, e = e._init;
          try {
            return D(e(n));
          } catch {
          }
      }
    return null;
  }
  var G = Array.isArray, N = r.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE, j = i.__DOM_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE, Z = {
    pending: !1,
    data: null,
    method: null,
    action: null
  }, Q = [], le = -1;
  function z(e) {
    return { current: e };
  }
  function V(e) {
    0 > le || (e.current = Q[le], Q[le] = null, le--);
  }
  function ie(e, n) {
    le++, Q[le] = e.current, e.current = n;
  }
  var L = z(null), I = z(null), P = z(null), ae = z(null);
  function W(e, n) {
    switch (ie(P, n), ie(I, e), ie(L, null), n.nodeType) {
      case 9:
      case 11:
        e = (e = n.documentElement) && (e = e.namespaceURI) ? bb(e) : 0;
        break;
      default:
        if (e = n.tagName, n = n.namespaceURI)
          n = bb(n), e = _b(n, e);
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
    V(L), ie(L, e);
  }
  function se() {
    V(L), V(I), V(P);
  }
  function de(e) {
    e.memoizedState !== null && ie(ae, e);
    var n = L.current, a = _b(n, e.type);
    n !== a && (ie(I, e), ie(L, a));
  }
  function ve(e) {
    I.current === e && (V(L), V(I)), ae.current === e && (V(ae), hu._currentValue = Z);
  }
  var pe, he;
  function me(e) {
    if (pe === void 0)
      try {
        throw Error();
      } catch (a) {
        var n = a.stack.trim().match(/\n( *(at )?)/);
        pe = n && n[1] || "", he = -1 < a.stack.indexOf(`
    at`) ? " (<anonymous>)" : -1 < a.stack.indexOf("@") ? "@unknown:0:0" : "";
      }
    return `
` + pe + e + he;
  }
  var ge = !1;
  function Ae(e, n) {
    if (!e || ge) return "";
    ge = !0;
    var a = Error.prepareStackTrace;
    Error.prepareStackTrace = void 0;
    try {
      var l = {
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
                } catch (ue) {
                  var ne = ue;
                }
                Reflect.construct(e, [], fe);
              } else {
                try {
                  fe.call();
                } catch (ue) {
                  ne = ue;
                }
                e.call(fe.prototype);
              }
            } else {
              try {
                throw Error();
              } catch (ue) {
                ne = ue;
              }
              (fe = e()) && typeof fe.catch == "function" && fe.catch(function() {
              });
            }
          } catch (ue) {
            if (ue && ne && typeof ue.stack == "string")
              return [ue.stack, ne.stack];
          }
          return [null, null];
        }
      };
      l.DetermineComponentFrameRoot.displayName = "DetermineComponentFrameRoot";
      var d = Object.getOwnPropertyDescriptor(
        l.DetermineComponentFrameRoot,
        "name"
      );
      d && d.configurable && Object.defineProperty(
        l.DetermineComponentFrameRoot,
        "name",
        { value: "DetermineComponentFrameRoot" }
      );
      var y = l.DetermineComponentFrameRoot(), x = y[0], q = y[1];
      if (x && q) {
        var U = x.split(`
`), te = q.split(`
`);
        for (d = l = 0; l < U.length && !U[l].includes("DetermineComponentFrameRoot"); )
          l++;
        for (; d < te.length && !te[d].includes(
          "DetermineComponentFrameRoot"
        ); )
          d++;
        if (l === U.length || d === te.length)
          for (l = U.length - 1, d = te.length - 1; 1 <= l && 0 <= d && U[l] !== te[d]; )
            d--;
        for (; 1 <= l && 0 <= d; l--, d--)
          if (U[l] !== te[d]) {
            if (l !== 1 || d !== 1)
              do
                if (l--, d--, 0 > d || U[l] !== te[d]) {
                  var oe = `
` + U[l].replace(" at new ", " at ");
                  return e.displayName && oe.includes("<anonymous>") && (oe = oe.replace("<anonymous>", e.displayName)), oe;
                }
              while (1 <= l && 0 <= d);
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
    } catch (l) {
      return `
Error generating stack: ` + l.message + `
` + l.stack;
    }
  }
  var tt = Object.prototype.hasOwnProperty, xt = t.unstable_scheduleCallback, gt = t.unstable_cancelCallback, St = t.unstable_shouldYield, Ze = t.unstable_requestPaint, ke = t.unstable_now, vt = t.unstable_getCurrentPriorityLevel, Ct = t.unstable_ImmediatePriority, Tt = t.unstable_UserBlockingPriority, ft = t.unstable_NormalPriority, tr = t.unstable_LowPriority, Cn = t.unstable_IdlePriority, _i = t.log, va = t.unstable_setDisableYieldValue, Et = null, Fe = null;
  function mn(e) {
    if (typeof _i == "function" && va(e), Fe && typeof Fe.setStrictMode == "function")
      try {
        Fe.setStrictMode(Et, e);
      } catch {
      }
  }
  var Mt = Math.clz32 ? Math.clz32 : Ss, _s = Math.log, xs = Math.LN2;
  function Ss(e) {
    return e >>>= 0, e === 0 ? 32 : 31 - (_s(e) / xs | 0) | 0;
  }
  var ya = 256, pa = 262144, ma = 4194304;
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
    var l = e.pendingLanes;
    if (l === 0) return 0;
    var d = 0, y = e.suspendedLanes, x = e.pingedLanes;
    e = e.warmLanes;
    var q = l & 134217727;
    return q !== 0 ? (l = q & ~y, l !== 0 ? d = Rn(l) : (x &= q, x !== 0 ? d = Rn(x) : a || (a = q & ~e, a !== 0 && (d = Rn(a))))) : (q = l & ~y, q !== 0 ? d = Rn(q) : x !== 0 ? d = Rn(x) : a || (a = l & ~e, a !== 0 && (d = Rn(a)))), d === 0 ? 0 : n !== 0 && n !== d && (n & y) === 0 && (y = d & -d, a = n & -n, y >= a || y === 32 && (a & 4194048) !== 0) ? n : d;
  }
  function Lr(e, n) {
    return (e.pendingLanes & ~(e.suspendedLanes & ~e.pingedLanes) & n) === 0;
  }
  function Es(e, n) {
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
  function $u() {
    var e = ma;
    return ma <<= 1, (ma & 62914560) === 0 && (ma = 4194304), e;
  }
  function xi(e) {
    for (var n = [], a = 0; 31 > a; a++) n.push(e);
    return n;
  }
  function Br(e, n) {
    e.pendingLanes |= n, n !== 268435456 && (e.suspendedLanes = 0, e.pingedLanes = 0, e.warmLanes = 0);
  }
  function ws(e, n, a, l, d, y) {
    var x = e.pendingLanes;
    e.pendingLanes = a, e.suspendedLanes = 0, e.pingedLanes = 0, e.warmLanes = 0, e.expiredLanes &= a, e.entangledLanes &= a, e.errorRecoveryDisabledLanes &= a, e.shellSuspendCounter = 0;
    var q = e.entanglements, U = e.expirationTimes, te = e.hiddenUpdates;
    for (a = x & ~a; 0 < a; ) {
      var oe = 31 - Mt(a), fe = 1 << oe;
      q[oe] = 0, U[oe] = -1;
      var ne = te[oe];
      if (ne !== null)
        for (te[oe] = null, oe = 0; oe < ne.length; oe++) {
          var ue = ne[oe];
          ue !== null && (ue.lane &= -536870913);
        }
      a &= ~fe;
    }
    l !== 0 && Fu(e, l, 0), y !== 0 && d === 0 && e.tag !== 0 && (e.suspendedLanes |= y & ~(x & ~n));
  }
  function Fu(e, n, a) {
    e.pendingLanes |= n, e.suspendedLanes &= ~n;
    var l = 31 - Mt(n);
    e.entangledLanes |= n, e.entanglements[l] = e.entanglements[l] | 1073741824 | a & 261930;
  }
  function Ju(e, n) {
    var a = e.entangledLanes |= n;
    for (e = e.entanglements; a; ) {
      var l = 31 - Mt(a), d = 1 << l;
      d & n | e[l] & n && (e[l] |= n), a &= ~d;
    }
  }
  function Pu(e, n) {
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
  function Wu() {
    var e = j.p;
    return e !== 0 ? e : (e = window.event, e === void 0 ? 32 : kb(e.type));
  }
  function el(e, n) {
    var a = j.p;
    try {
      return j.p = e, n();
    } finally {
      j.p = a;
    }
  }
  var bn = Math.random().toString(36).slice(2), dt = "__reactFiber$" + bn, wt = "__reactProps$" + bn, Nn = "__reactContainer$" + bn, _a = "__reactEvents$" + bn, tl = "__reactListeners$" + bn, As = "__reactHandles$" + bn, nl = "__reactResources$" + bn, jr = "__reactMarker$" + bn;
  function wi(e) {
    delete e[dt], delete e[wt], delete e[_a], delete e[tl], delete e[As];
  }
  function nr(e) {
    var n = e[dt];
    if (n) return n;
    for (var a = e.parentNode; a; ) {
      if (n = a[Nn] || a[dt]) {
        if (a = n.alternate, n.child !== null || a !== null && a.child !== null)
          for (e = Mb(e); e !== null; ) {
            if (a = e[dt]) return a;
            e = Mb(e);
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
    var n = e[nl];
    return n || (n = e[nl] = { hoistableStyles: /* @__PURE__ */ new Map(), hoistableScripts: /* @__PURE__ */ new Map() }), n;
  }
  function at(e) {
    e[jr] = !0;
  }
  var rl = /* @__PURE__ */ new Set(), al = {};
  function On(e, n) {
    ur(e, n), ur(e + "Capture", n);
  }
  function ur(e, n) {
    for (al[e] = n, e = 0; e < n.length; e++)
      rl.add(n[e]);
  }
  var Ts = RegExp(
    "^[:A-Z_a-z\\u00C0-\\u00D6\\u00D8-\\u00F6\\u00F8-\\u02FF\\u0370-\\u037D\\u037F-\\u1FFF\\u200C-\\u200D\\u2070-\\u218F\\u2C00-\\u2FEF\\u3001-\\uD7FF\\uF900-\\uFDCF\\uFDF0-\\uFFFD][:A-Z_a-z\\u00C0-\\u00D6\\u00D8-\\u00F6\\u00F8-\\u02FF\\u0370-\\u037D\\u037F-\\u1FFF\\u200C-\\u200D\\u2070-\\u218F\\u2C00-\\u2FEF\\u3001-\\uD7FF\\uF900-\\uFDCF\\uFDF0-\\uFFFD\\-.0-9\\u00B7\\u0300-\\u036F\\u203F-\\u2040]*$"
  ), il = {}, Ai = {};
  function Ms(e) {
    return tt.call(Ai, e) ? !0 : tt.call(il, e) ? !1 : Ts.test(e) ? Ai[e] = !0 : (il[e] = !0, !1);
  }
  function xa(e, n, a) {
    if (Ms(n))
      if (a === null) e.removeAttribute(n);
      else {
        switch (typeof a) {
          case "undefined":
          case "function":
          case "symbol":
            e.removeAttribute(n);
            return;
          case "boolean":
            var l = n.toLowerCase().slice(0, 5);
            if (l !== "data-" && l !== "aria-") {
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
  function un(e, n, a, l) {
    if (l === null) e.removeAttribute(a);
    else {
      switch (typeof l) {
        case "undefined":
        case "function":
        case "symbol":
        case "boolean":
          e.removeAttribute(a);
          return;
      }
      e.setAttributeNS(n, a, "" + l);
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
  function ul(e) {
    var n = e.type;
    return (e = e.nodeName) && e.toLowerCase() === "input" && (n === "checkbox" || n === "radio");
  }
  function qs(e, n, a) {
    var l = Object.getOwnPropertyDescriptor(
      e.constructor.prototype,
      n
    );
    if (!e.hasOwnProperty(n) && typeof l < "u" && typeof l.get == "function" && typeof l.set == "function") {
      var d = l.get, y = l.set;
      return Object.defineProperty(e, n, {
        configurable: !0,
        get: function() {
          return d.call(this);
        },
        set: function(x) {
          a = "" + x, y.call(this, x);
        }
      }), Object.defineProperty(e, n, {
        enumerable: l.enumerable
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
      var n = ul(e) ? "checked" : "value";
      e._valueTracker = qs(
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
    var a = n.getValue(), l = "";
    return e && (l = ul(e) ? e.checked ? "true" : "false" : e.value), e = l, e !== a ? (n.setValue(e), !0) : !1;
  }
  function Ea(e) {
    if (e = e || (typeof document < "u" ? document : void 0), typeof e > "u") return null;
    try {
      return e.activeElement || e.body;
    } catch {
      return e.body;
    }
  }
  var ll = /[\n"\\]/g;
  function Nt(e) {
    return e.replace(
      ll,
      function(n) {
        return "\\" + n.charCodeAt(0).toString(16) + " ";
      }
    );
  }
  function Mi(e, n, a, l, d, y, x, q) {
    e.name = "", x != null && typeof x != "function" && typeof x != "symbol" && typeof x != "boolean" ? e.type = x : e.removeAttribute("type"), n != null ? x === "number" ? (n === 0 && e.value === "" || e.value != n) && (e.value = "" + Rt(n)) : e.value !== "" + Rt(n) && (e.value = "" + Rt(n)) : x !== "submit" && x !== "reset" || e.removeAttribute("value"), n != null ? qi(e, x, Rt(n)) : a != null ? qi(e, x, Rt(a)) : l != null && e.removeAttribute("value"), d == null && y != null && (e.defaultChecked = !!y), d != null && (e.checked = d && typeof d != "function" && typeof d != "symbol"), q != null && typeof q != "function" && typeof q != "symbol" && typeof q != "boolean" ? e.name = "" + Rt(q) : e.removeAttribute("name");
  }
  function wa(e, n, a, l, d, y, x, q) {
    if (y != null && typeof y != "function" && typeof y != "symbol" && typeof y != "boolean" && (e.type = y), n != null || a != null) {
      if (!(y !== "submit" && y !== "reset" || n != null)) {
        Ti(e);
        return;
      }
      a = a != null ? "" + Rt(a) : "", n = n != null ? "" + Rt(n) : a, q || n === e.value || (e.value = n), e.defaultValue = n;
    }
    l = l ?? d, l = typeof l != "function" && typeof l != "symbol" && !!l, e.checked = q ? e.checked : !!l, e.defaultChecked = !!l, x != null && typeof x != "function" && typeof x != "symbol" && typeof x != "boolean" && (e.name = x), Ti(e);
  }
  function qi(e, n, a) {
    n === "number" && Ea(e.ownerDocument) === e || e.defaultValue === "" + a || (e.defaultValue = "" + a);
  }
  function lr(e, n, a, l) {
    if (e = e.options, n) {
      n = {};
      for (var d = 0; d < a.length; d++)
        n["$" + a[d]] = !0;
      for (a = 0; a < e.length; a++)
        d = n.hasOwnProperty("$" + e[a].value), e[a].selected !== d && (e[a].selected = d), d && l && (e[a].defaultSelected = !0);
    } else {
      for (a = "" + Rt(a), n = null, d = 0; d < e.length; d++) {
        if (e[d].value === a) {
          e[d].selected = !0, l && (e[d].defaultSelected = !0);
          return;
        }
        n !== null || e[d].disabled || (n = e[d]);
      }
      n !== null && (n.selected = !0);
    }
  }
  function Ci(e, n, a) {
    if (n != null && (n = "" + Rt(n), n !== e.value && (e.value = n), a == null)) {
      e.defaultValue !== n && (e.defaultValue = n);
      return;
    }
    e.defaultValue = a != null ? "" + Rt(a) : "";
  }
  function vm(e, n, a, l) {
    if (n == null) {
      if (l != null) {
        if (a != null) throw Error(u(92));
        if (G(l)) {
          if (1 < l.length) throw Error(u(93));
          l = l[0];
        }
        a = l;
      }
      a == null && (a = ""), n = a;
    }
    a = Rt(n), e.defaultValue = a, l = e.textContent, l === a && l !== "" && l !== null && (e.value = l), Ti(e);
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
  var Rq = new Set(
    "animationIterationCount aspectRatio borderImageOutset borderImageSlice borderImageWidth boxFlex boxFlexGroup boxOrdinalGroup columnCount columns flex flexGrow flexPositive flexShrink flexNegative flexOrder gridArea gridRow gridRowEnd gridRowSpan gridRowStart gridColumn gridColumnEnd gridColumnSpan gridColumnStart fontWeight lineClamp lineHeight opacity order orphans scale tabSize widows zIndex zoom fillOpacity floodOpacity stopOpacity strokeDasharray strokeDashoffset strokeMiterlimit strokeOpacity strokeWidth MozAnimationIterationCount MozBoxFlex MozBoxFlexGroup MozLineClamp msAnimationIterationCount msFlex msZoom msFlexGrow msFlexNegative msFlexOrder msFlexPositive msFlexShrink msGridColumn msGridColumnSpan msGridRow msGridRowSpan WebkitAnimationIterationCount WebkitBoxFlex WebKitBoxFlexGroup WebkitBoxOrdinalGroup WebkitColumnCount WebkitColumns WebkitFlex WebkitFlexGrow WebkitFlexPositive WebkitFlexShrink WebkitLineClamp".split(
      " "
    )
  );
  function ym(e, n, a) {
    var l = n.indexOf("--") === 0;
    a == null || typeof a == "boolean" || a === "" ? l ? e.setProperty(n, "") : n === "float" ? e.cssFloat = "" : e[n] = "" : l ? e.setProperty(n, a) : typeof a != "number" || a === 0 || Rq.has(n) ? n === "float" ? e.cssFloat = a : e[n] = ("" + a).trim() : e[n] = a + "px";
  }
  function pm(e, n, a) {
    if (n != null && typeof n != "object")
      throw Error(u(62));
    if (e = e.style, a != null) {
      for (var l in a)
        !a.hasOwnProperty(l) || n != null && n.hasOwnProperty(l) || (l.indexOf("--") === 0 ? e.setProperty(l, "") : l === "float" ? e.cssFloat = "" : e[l] = "");
      for (var d in n)
        l = n[d], n.hasOwnProperty(d) && a[d] !== l && ym(e, d, l);
    } else
      for (var y in n)
        n.hasOwnProperty(y) && ym(e, y, n[y]);
  }
  function Cs(e) {
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
  var Nq = /* @__PURE__ */ new Map([
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
  ]), Oq = /^[\u0000-\u001F ]*j[\r\n\t]*a[\r\n\t]*v[\r\n\t]*a[\r\n\t]*s[\r\n\t]*c[\r\n\t]*r[\r\n\t]*i[\r\n\t]*p[\r\n\t]*t[\r\n\t]*:/i;
  function ol(e) {
    return Oq.test("" + e) ? "javascript:throw new Error('React has blocked a javascript: URL as a security precaution.')" : e;
  }
  function zn() {
  }
  var Rs = null;
  function Ns(e) {
    return e = e.target || e.srcElement || window, e.correspondingUseElement && (e = e.correspondingUseElement), e.nodeType === 3 ? e.parentNode : e;
  }
  var Ta = null, Ma = null;
  function mm(e) {
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
              var l = a[n];
              if (l !== e && l.form === e.form) {
                var d = l[wt] || null;
                if (!d) throw Error(u(90));
                Mi(
                  l,
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
              l = a[n], l.form === e.form && Ur(l);
          }
          break e;
        case "textarea":
          Ci(e, a.value, a.defaultValue);
          break e;
        case "select":
          n = a.value, n != null && lr(e, !!a.multiple, n, !1);
      }
    }
  }
  var Os = !1;
  function bm(e, n, a) {
    if (Os) return e(n, a);
    Os = !0;
    try {
      var l = e(n);
      return l;
    } finally {
      if (Os = !1, (Ta !== null || Ma !== null) && ($l(), Ta && (n = Ta, e = Ma, Ma = Ta = null, mm(n), e)))
        for (n = 0; n < e.length; n++) mm(e[n]);
    }
  }
  function Ri(e, n) {
    var a = e.stateNode;
    if (a === null) return null;
    var l = a[wt] || null;
    if (l === null) return null;
    a = l[n];
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
        (l = !l.disabled) || (e = e.type, l = !(e === "button" || e === "input" || e === "select" || e === "textarea")), e = !l;
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
  var Dn = !(typeof window > "u" || typeof window.document > "u" || typeof window.document.createElement > "u"), zs = !1;
  if (Dn)
    try {
      var Ni = {};
      Object.defineProperty(Ni, "passive", {
        get: function() {
          zs = !0;
        }
      }), window.addEventListener("test", Ni, Ni), window.removeEventListener("test", Ni, Ni);
    } catch {
      zs = !1;
    }
  var or = null, Ds = null, sl = null;
  function _m() {
    if (sl) return sl;
    var e, n = Ds, a = n.length, l, d = "value" in or ? or.value : or.textContent, y = d.length;
    for (e = 0; e < a && n[e] === d[e]; e++) ;
    var x = a - e;
    for (l = 1; l <= x && n[a - l] === d[y - l]; l++) ;
    return sl = d.slice(e, 1 < l ? 1 - l : void 0);
  }
  function cl(e) {
    var n = e.keyCode;
    return "charCode" in e ? (e = e.charCode, e === 0 && n === 13 && (e = 13)) : e = n, e === 10 && (e = 13), 32 <= e || e === 13 ? e : 0;
  }
  function fl() {
    return !0;
  }
  function xm() {
    return !1;
  }
  function Ot(e) {
    function n(a, l, d, y, x) {
      this._reactName = a, this._targetInst = d, this.type = l, this.nativeEvent = y, this.target = x, this.currentTarget = null;
      for (var q in e)
        e.hasOwnProperty(q) && (a = e[q], this[q] = a ? a(y) : y[q]);
      return this.isDefaultPrevented = (y.defaultPrevented != null ? y.defaultPrevented : y.returnValue === !1) ? fl : xm, this.isPropagationStopped = xm, this;
    }
    return p(n.prototype, {
      preventDefault: function() {
        this.defaultPrevented = !0;
        var a = this.nativeEvent;
        a && (a.preventDefault ? a.preventDefault() : typeof a.returnValue != "unknown" && (a.returnValue = !1), this.isDefaultPrevented = fl);
      },
      stopPropagation: function() {
        var a = this.nativeEvent;
        a && (a.stopPropagation ? a.stopPropagation() : typeof a.cancelBubble != "unknown" && (a.cancelBubble = !0), this.isPropagationStopped = fl);
      },
      persist: function() {
      },
      isPersistent: fl
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
  }, dl = Ot(Gr), Oi = p({}, Gr, { view: 0, detail: 0 }), zq = Ot(Oi), Hs, Ls, zi, hl = p({}, Oi, {
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
    getModifierState: js,
    button: 0,
    buttons: 0,
    relatedTarget: function(e) {
      return e.relatedTarget === void 0 ? e.fromElement === e.srcElement ? e.toElement : e.fromElement : e.relatedTarget;
    },
    movementX: function(e) {
      return "movementX" in e ? e.movementX : (e !== zi && (zi && e.type === "mousemove" ? (Hs = e.screenX - zi.screenX, Ls = e.screenY - zi.screenY) : Ls = Hs = 0, zi = e), Hs);
    },
    movementY: function(e) {
      return "movementY" in e ? e.movementY : Ls;
    }
  }), Sm = Ot(hl), Dq = p({}, hl, { dataTransfer: 0 }), Hq = Ot(Dq), Lq = p({}, Oi, { relatedTarget: 0 }), Bs = Ot(Lq), Bq = p({}, Gr, {
    animationName: 0,
    elapsedTime: 0,
    pseudoElement: 0
  }), jq = Ot(Bq), Uq = p({}, Gr, {
    clipboardData: function(e) {
      return "clipboardData" in e ? e.clipboardData : window.clipboardData;
    }
  }), Gq = Ot(Uq), Vq = p({}, Gr, { data: 0 }), Em = Ot(Vq), Yq = {
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
  }, kq = {
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
  }, Xq = {
    Alt: "altKey",
    Control: "ctrlKey",
    Meta: "metaKey",
    Shift: "shiftKey"
  };
  function Iq(e) {
    var n = this.nativeEvent;
    return n.getModifierState ? n.getModifierState(e) : (e = Xq[e]) ? !!n[e] : !1;
  }
  function js() {
    return Iq;
  }
  var Qq = p({}, Oi, {
    key: function(e) {
      if (e.key) {
        var n = Yq[e.key] || e.key;
        if (n !== "Unidentified") return n;
      }
      return e.type === "keypress" ? (e = cl(e), e === 13 ? "Enter" : String.fromCharCode(e)) : e.type === "keydown" || e.type === "keyup" ? kq[e.keyCode] || "Unidentified" : "";
    },
    code: 0,
    location: 0,
    ctrlKey: 0,
    shiftKey: 0,
    altKey: 0,
    metaKey: 0,
    repeat: 0,
    locale: 0,
    getModifierState: js,
    charCode: function(e) {
      return e.type === "keypress" ? cl(e) : 0;
    },
    keyCode: function(e) {
      return e.type === "keydown" || e.type === "keyup" ? e.keyCode : 0;
    },
    which: function(e) {
      return e.type === "keypress" ? cl(e) : e.type === "keydown" || e.type === "keyup" ? e.keyCode : 0;
    }
  }), Zq = Ot(Qq), Kq = p({}, hl, {
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
  }), wm = Ot(Kq), $q = p({}, Oi, {
    touches: 0,
    targetTouches: 0,
    changedTouches: 0,
    altKey: 0,
    metaKey: 0,
    ctrlKey: 0,
    shiftKey: 0,
    getModifierState: js
  }), Fq = Ot($q), Jq = p({}, Gr, {
    propertyName: 0,
    elapsedTime: 0,
    pseudoElement: 0
  }), Pq = Ot(Jq), Wq = p({}, hl, {
    deltaX: function(e) {
      return "deltaX" in e ? e.deltaX : "wheelDeltaX" in e ? -e.wheelDeltaX : 0;
    },
    deltaY: function(e) {
      return "deltaY" in e ? e.deltaY : "wheelDeltaY" in e ? -e.wheelDeltaY : "wheelDelta" in e ? -e.wheelDelta : 0;
    },
    deltaZ: 0,
    deltaMode: 0
  }), eC = Ot(Wq), tC = p({}, Gr, {
    newState: 0,
    oldState: 0
  }), nC = Ot(tC), rC = [9, 13, 27, 32], Us = Dn && "CompositionEvent" in window, Di = null;
  Dn && "documentMode" in document && (Di = document.documentMode);
  var aC = Dn && "TextEvent" in window && !Di, Am = Dn && (!Us || Di && 8 < Di && 11 >= Di), Tm = " ", Mm = !1;
  function qm(e, n) {
    switch (e) {
      case "keyup":
        return rC.indexOf(n.keyCode) !== -1;
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
  function Cm(e) {
    return e = e.detail, typeof e == "object" && "data" in e ? e.data : null;
  }
  var qa = !1;
  function iC(e, n) {
    switch (e) {
      case "compositionend":
        return Cm(n);
      case "keypress":
        return n.which !== 32 ? null : (Mm = !0, Tm);
      case "textInput":
        return e = n.data, e === Tm && Mm ? null : e;
      default:
        return null;
    }
  }
  function uC(e, n) {
    if (qa)
      return e === "compositionend" || !Us && qm(e, n) ? (e = _m(), sl = Ds = or = null, qa = !1, e) : null;
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
        return Am && n.locale !== "ko" ? null : n.data;
      default:
        return null;
    }
  }
  var lC = {
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
  function Rm(e) {
    var n = e && e.nodeName && e.nodeName.toLowerCase();
    return n === "input" ? !!lC[e.type] : n === "textarea";
  }
  function Nm(e, n, a, l) {
    Ta ? Ma ? Ma.push(l) : Ma = [l] : Ta = l, n = no(n, "onChange"), 0 < n.length && (a = new dl(
      "onChange",
      "change",
      null,
      a,
      l
    ), e.push({ event: a, listeners: n }));
  }
  var Hi = null, Li = null;
  function oC(e) {
    hb(e, 0);
  }
  function gl(e) {
    var n = ar(e);
    if (Ur(n)) return e;
  }
  function Om(e, n) {
    if (e === "change") return n;
  }
  var zm = !1;
  if (Dn) {
    var Gs;
    if (Dn) {
      var Vs = "oninput" in document;
      if (!Vs) {
        var Dm = document.createElement("div");
        Dm.setAttribute("oninput", "return;"), Vs = typeof Dm.oninput == "function";
      }
      Gs = Vs;
    } else Gs = !1;
    zm = Gs && (!document.documentMode || 9 < document.documentMode);
  }
  function Hm() {
    Hi && (Hi.detachEvent("onpropertychange", Lm), Li = Hi = null);
  }
  function Lm(e) {
    if (e.propertyName === "value" && gl(Li)) {
      var n = [];
      Nm(
        n,
        Li,
        e,
        Ns(e)
      ), bm(oC, n);
    }
  }
  function sC(e, n, a) {
    e === "focusin" ? (Hm(), Hi = n, Li = a, Hi.attachEvent("onpropertychange", Lm)) : e === "focusout" && Hm();
  }
  function cC(e) {
    if (e === "selectionchange" || e === "keyup" || e === "keydown")
      return gl(Li);
  }
  function fC(e, n) {
    if (e === "click") return gl(n);
  }
  function dC(e, n) {
    if (e === "input" || e === "change")
      return gl(n);
  }
  function hC(e, n) {
    return e === n && (e !== 0 || 1 / e === 1 / n) || e !== e && n !== n;
  }
  var Gt = typeof Object.is == "function" ? Object.is : hC;
  function Bi(e, n) {
    if (Gt(e, n)) return !0;
    if (typeof e != "object" || e === null || typeof n != "object" || n === null)
      return !1;
    var a = Object.keys(e), l = Object.keys(n);
    if (a.length !== l.length) return !1;
    for (l = 0; l < a.length; l++) {
      var d = a[l];
      if (!tt.call(n, d) || !Gt(e[d], n[d]))
        return !1;
    }
    return !0;
  }
  function Bm(e) {
    for (; e && e.firstChild; ) e = e.firstChild;
    return e;
  }
  function jm(e, n) {
    var a = Bm(e);
    e = 0;
    for (var l; a; ) {
      if (a.nodeType === 3) {
        if (l = e + a.textContent.length, e <= n && l >= n)
          return { node: a, offset: n - e };
        e = l;
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
      a = Bm(a);
    }
  }
  function Um(e, n) {
    return e && n ? e === n ? !0 : e && e.nodeType === 3 ? !1 : n && n.nodeType === 3 ? Um(e, n.parentNode) : "contains" in e ? e.contains(n) : e.compareDocumentPosition ? !!(e.compareDocumentPosition(n) & 16) : !1 : !1;
  }
  function Gm(e) {
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
  function Ys(e) {
    var n = e && e.nodeName && e.nodeName.toLowerCase();
    return n && (n === "input" && (e.type === "text" || e.type === "search" || e.type === "tel" || e.type === "url" || e.type === "password") || n === "textarea" || e.contentEditable === "true");
  }
  var gC = Dn && "documentMode" in document && 11 >= document.documentMode, Ca = null, ks = null, ji = null, Xs = !1;
  function Vm(e, n, a) {
    var l = a.window === a ? a.document : a.nodeType === 9 ? a : a.ownerDocument;
    Xs || Ca == null || Ca !== Ea(l) || (l = Ca, "selectionStart" in l && Ys(l) ? l = { start: l.selectionStart, end: l.selectionEnd } : (l = (l.ownerDocument && l.ownerDocument.defaultView || window).getSelection(), l = {
      anchorNode: l.anchorNode,
      anchorOffset: l.anchorOffset,
      focusNode: l.focusNode,
      focusOffset: l.focusOffset
    }), ji && Bi(ji, l) || (ji = l, l = no(ks, "onSelect"), 0 < l.length && (n = new dl(
      "onSelect",
      "select",
      null,
      n,
      a
    ), e.push({ event: n, listeners: l }), n.target = Ca)));
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
  }, Is = {}, Ym = {};
  Dn && (Ym = document.createElement("div").style, "AnimationEvent" in window || (delete Ra.animationend.animation, delete Ra.animationiteration.animation, delete Ra.animationstart.animation), "TransitionEvent" in window || delete Ra.transitionend.transition);
  function Yr(e) {
    if (Is[e]) return Is[e];
    if (!Ra[e]) return e;
    var n = Ra[e], a;
    for (a in n)
      if (n.hasOwnProperty(a) && a in Ym)
        return Is[e] = n[a];
    return e;
  }
  var km = Yr("animationend"), Xm = Yr("animationiteration"), Im = Yr("animationstart"), vC = Yr("transitionrun"), yC = Yr("transitionstart"), pC = Yr("transitioncancel"), Qm = Yr("transitionend"), Zm = /* @__PURE__ */ new Map(), Qs = "abort auxClick beforeToggle cancel canPlay canPlayThrough click close contextMenu copy cut drag dragEnd dragEnter dragExit dragLeave dragOver dragStart drop durationChange emptied encrypted ended error gotPointerCapture input invalid keyDown keyPress keyUp load loadedData loadedMetadata loadStart lostPointerCapture mouseDown mouseMove mouseOut mouseOver mouseUp paste pause play playing pointerCancel pointerDown pointerMove pointerOut pointerOver pointerUp progress rateChange reset resize seeked seeking stalled submit suspend timeUpdate touchCancel touchEnd touchStart volumeChange scroll toggle touchMove waiting wheel".split(
    " "
  );
  Qs.push("scrollEnd");
  function ln(e, n) {
    Zm.set(e, n), On(n, [e]);
  }
  var vl = typeof reportError == "function" ? reportError : function(e) {
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
  }, $t = [], Na = 0, Zs = 0;
  function yl() {
    for (var e = Na, n = Zs = Na = 0; n < e; ) {
      var a = $t[n];
      $t[n++] = null;
      var l = $t[n];
      $t[n++] = null;
      var d = $t[n];
      $t[n++] = null;
      var y = $t[n];
      if ($t[n++] = null, l !== null && d !== null) {
        var x = l.pending;
        x === null ? d.next = d : (d.next = x.next, x.next = d), l.pending = d;
      }
      y !== 0 && Km(a, d, y);
    }
  }
  function pl(e, n, a, l) {
    $t[Na++] = e, $t[Na++] = n, $t[Na++] = a, $t[Na++] = l, Zs |= l, e.lanes |= l, e = e.alternate, e !== null && (e.lanes |= l);
  }
  function Ks(e, n, a, l) {
    return pl(e, n, a, l), ml(e);
  }
  function kr(e, n) {
    return pl(e, null, null, n), ml(e);
  }
  function Km(e, n, a) {
    e.lanes |= a;
    var l = e.alternate;
    l !== null && (l.lanes |= a);
    for (var d = !1, y = e.return; y !== null; )
      y.childLanes |= a, l = y.alternate, l !== null && (l.childLanes |= a), y.tag === 22 && (e = y.stateNode, e === null || e._visibility & 1 || (d = !0)), e = y, y = y.return;
    return e.tag === 3 ? (y = e.stateNode, d && n !== null && (d = 31 - Mt(a), e = y.hiddenUpdates, l = e[d], l === null ? e[d] = [n] : l.push(n), n.lane = a | 536870912), y) : null;
  }
  function ml(e) {
    if (50 < uu)
      throw uu = 0, rf = null, Error(u(185));
    for (var n = e.return; n !== null; )
      e = n, n = e.return;
    return e.tag === 3 ? e.stateNode : null;
  }
  var Oa = {};
  function mC(e, n, a, l) {
    this.tag = e, this.key = a, this.sibling = this.child = this.return = this.stateNode = this.type = this.elementType = null, this.index = 0, this.refCleanup = this.ref = null, this.pendingProps = n, this.dependencies = this.memoizedState = this.updateQueue = this.memoizedProps = null, this.mode = l, this.subtreeFlags = this.flags = 0, this.deletions = null, this.childLanes = this.lanes = 0, this.alternate = null;
  }
  function Vt(e, n, a, l) {
    return new mC(e, n, a, l);
  }
  function $s(e) {
    return e = e.prototype, !(!e || !e.isReactComponent);
  }
  function Hn(e, n) {
    var a = e.alternate;
    return a === null ? (a = Vt(
      e.tag,
      n,
      e.key,
      e.mode
    ), a.elementType = e.elementType, a.type = e.type, a.stateNode = e.stateNode, a.alternate = e, e.alternate = a) : (a.pendingProps = n, a.type = e.type, a.flags = 0, a.subtreeFlags = 0, a.deletions = null), a.flags = e.flags & 65011712, a.childLanes = e.childLanes, a.lanes = e.lanes, a.child = e.child, a.memoizedProps = e.memoizedProps, a.memoizedState = e.memoizedState, a.updateQueue = e.updateQueue, n = e.dependencies, a.dependencies = n === null ? null : { lanes: n.lanes, firstContext: n.firstContext }, a.sibling = e.sibling, a.index = e.index, a.ref = e.ref, a.refCleanup = e.refCleanup, a;
  }
  function $m(e, n) {
    e.flags &= 65011714;
    var a = e.alternate;
    return a === null ? (e.childLanes = 0, e.lanes = n, e.child = null, e.subtreeFlags = 0, e.memoizedProps = null, e.memoizedState = null, e.updateQueue = null, e.dependencies = null, e.stateNode = null) : (e.childLanes = a.childLanes, e.lanes = a.lanes, e.child = a.child, e.subtreeFlags = 0, e.deletions = null, e.memoizedProps = a.memoizedProps, e.memoizedState = a.memoizedState, e.updateQueue = a.updateQueue, e.type = a.type, n = a.dependencies, e.dependencies = n === null ? null : {
      lanes: n.lanes,
      firstContext: n.firstContext
    }), e;
  }
  function bl(e, n, a, l, d, y) {
    var x = 0;
    if (l = e, typeof e == "function") $s(e) && (x = 1);
    else if (typeof e == "string")
      x = ER(
        e,
        a,
        L.current
      ) ? 26 : e === "html" || e === "head" || e === "body" ? 27 : 5;
    else
      e: switch (e) {
        case B:
          return e = Vt(31, a, n, d), e.elementType = B, e.lanes = y, e;
        case A:
          return Xr(a.children, d, y, n);
        case w:
          x = 8, d |= 24;
          break;
        case E:
          return e = Vt(12, a, n, d | 2), e.elementType = E, e.lanes = y, e;
        case O:
          return e = Vt(13, a, n, d), e.elementType = O, e.lanes = y, e;
        case C:
          return e = Vt(19, a, n, d), e.elementType = C, e.lanes = y, e;
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
              case R:
                x = 14;
                break e;
              case H:
                x = 16, l = null;
                break e;
            }
          x = 29, a = Error(
            u(130, e === null ? "null" : typeof e, "")
          ), l = null;
      }
    return n = Vt(x, a, n, d), n.elementType = e, n.type = l, n.lanes = y, n;
  }
  function Xr(e, n, a, l) {
    return e = Vt(7, e, l, n), e.lanes = a, e;
  }
  function Fs(e, n, a) {
    return e = Vt(6, e, null, n), e.lanes = a, e;
  }
  function Fm(e) {
    var n = Vt(18, null, null, 0);
    return n.stateNode = e, n;
  }
  function Js(e, n, a) {
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
  var Jm = /* @__PURE__ */ new WeakMap();
  function Ft(e, n) {
    if (typeof e == "object" && e !== null) {
      var a = Jm.get(e);
      return a !== void 0 ? a : (n = {
        value: e,
        source: n,
        stack: Pe(n)
      }, Jm.set(e, n), n);
    }
    return {
      value: e,
      source: n,
      stack: Pe(n)
    };
  }
  var za = [], Da = 0, _l = null, Ui = 0, Jt = [], Pt = 0, sr = null, _n = 1, xn = "";
  function Ln(e, n) {
    za[Da++] = Ui, za[Da++] = _l, _l = e, Ui = n;
  }
  function Pm(e, n, a) {
    Jt[Pt++] = _n, Jt[Pt++] = xn, Jt[Pt++] = sr, sr = e;
    var l = _n;
    e = xn;
    var d = 32 - Mt(l) - 1;
    l &= ~(1 << d), a += 1;
    var y = 32 - Mt(n) + d;
    if (30 < y) {
      var x = d - d % 5;
      y = (l & (1 << x) - 1).toString(32), l >>= x, d -= x, _n = 1 << 32 - Mt(n) + d | a << d | l, xn = y + e;
    } else
      _n = 1 << y | a << d | l, xn = e;
  }
  function Ps(e) {
    e.return !== null && (Ln(e, 1), Pm(e, 1, 0));
  }
  function Ws(e) {
    for (; e === _l; )
      _l = za[--Da], za[Da] = null, Ui = za[--Da], za[Da] = null;
    for (; e === sr; )
      sr = Jt[--Pt], Jt[Pt] = null, xn = Jt[--Pt], Jt[Pt] = null, _n = Jt[--Pt], Jt[Pt] = null;
  }
  function Wm(e, n) {
    Jt[Pt++] = _n, Jt[Pt++] = xn, Jt[Pt++] = sr, _n = n.id, xn = n.overflow, sr = e;
  }
  var pt = null, Xe = null, Oe = !1, cr = null, Wt = !1, ec = Error(u(519));
  function fr(e) {
    var n = Error(
      u(
        418,
        1 < arguments.length && arguments[1] !== void 0 && arguments[1] ? "text" : "HTML",
        ""
      )
    );
    throw Gi(Ft(n, e)), ec;
  }
  function e0(e) {
    var n = e.stateNode, a = e.type, l = e.memoizedProps;
    switch (n[dt] = e, n[wt] = l, a) {
      case "dialog":
        Ce("cancel", n), Ce("close", n);
        break;
      case "iframe":
      case "object":
      case "embed":
        Ce("load", n);
        break;
      case "video":
      case "audio":
        for (a = 0; a < ou.length; a++)
          Ce(ou[a], n);
        break;
      case "source":
        Ce("error", n);
        break;
      case "img":
      case "image":
      case "link":
        Ce("error", n), Ce("load", n);
        break;
      case "details":
        Ce("toggle", n);
        break;
      case "input":
        Ce("invalid", n), wa(
          n,
          l.value,
          l.defaultValue,
          l.checked,
          l.defaultChecked,
          l.type,
          l.name,
          !0
        );
        break;
      case "select":
        Ce("invalid", n);
        break;
      case "textarea":
        Ce("invalid", n), vm(n, l.value, l.defaultValue, l.children);
    }
    a = l.children, typeof a != "string" && typeof a != "number" && typeof a != "bigint" || n.textContent === "" + a || l.suppressHydrationWarning === !0 || pb(n.textContent, a) ? (l.popover != null && (Ce("beforetoggle", n), Ce("toggle", n)), l.onScroll != null && Ce("scroll", n), l.onScrollEnd != null && Ce("scrollend", n), l.onClick != null && (n.onclick = zn), n = !0) : n = !1, n || fr(e, !0);
  }
  function t0(e) {
    for (pt = e.return; pt; )
      switch (pt.tag) {
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
          pt = pt.return;
      }
  }
  function Ha(e) {
    if (e !== pt) return !1;
    if (!Oe) return t0(e), Oe = !0, !1;
    var n = e.tag, a;
    if ((a = n !== 3 && n !== 27) && ((a = n === 5) && (a = e.type, a = !(a !== "form" && a !== "button") || bf(e.type, e.memoizedProps)), a = !a), a && Xe && fr(e), t0(e), n === 13) {
      if (e = e.memoizedState, e = e !== null ? e.dehydrated : null, !e) throw Error(u(317));
      Xe = Tb(e);
    } else if (n === 31) {
      if (e = e.memoizedState, e = e !== null ? e.dehydrated : null, !e) throw Error(u(317));
      Xe = Tb(e);
    } else
      n === 27 ? (n = Xe, Ar(e.type) ? (e = wf, wf = null, Xe = e) : Xe = n) : Xe = pt ? tn(e.stateNode.nextSibling) : null;
    return !0;
  }
  function Ir() {
    Xe = pt = null, Oe = !1;
  }
  function tc() {
    var e = cr;
    return e !== null && (Lt === null ? Lt = e : Lt.push.apply(
      Lt,
      e
    ), cr = null), e;
  }
  function Gi(e) {
    cr === null ? cr = [e] : cr.push(e);
  }
  var nc = z(null), Qr = null, Bn = null;
  function dr(e, n, a) {
    ie(nc, n._currentValue), n._currentValue = a;
  }
  function jn(e) {
    e._currentValue = nc.current, V(nc);
  }
  function rc(e, n, a) {
    for (; e !== null; ) {
      var l = e.alternate;
      if ((e.childLanes & n) !== n ? (e.childLanes |= n, l !== null && (l.childLanes |= n)) : l !== null && (l.childLanes & n) !== n && (l.childLanes |= n), e === a) break;
      e = e.return;
    }
  }
  function ac(e, n, a, l) {
    var d = e.child;
    for (d !== null && (d.return = e); d !== null; ) {
      var y = d.dependencies;
      if (y !== null) {
        var x = d.child;
        y = y.firstContext;
        e: for (; y !== null; ) {
          var q = y;
          y = d;
          for (var U = 0; U < n.length; U++)
            if (q.context === n[U]) {
              y.lanes |= a, q = y.alternate, q !== null && (q.lanes |= a), rc(
                y.return,
                a,
                e
              ), l || (x = null);
              break e;
            }
          y = q.next;
        }
      } else if (d.tag === 18) {
        if (x = d.return, x === null) throw Error(u(341));
        x.lanes |= a, y = x.alternate, y !== null && (y.lanes |= a), rc(x, a, e), x = null;
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
  function La(e, n, a, l) {
    e = null;
    for (var d = n, y = !1; d !== null; ) {
      if (!y) {
        if ((d.flags & 524288) !== 0) y = !0;
        else if ((d.flags & 262144) !== 0) break;
      }
      if (d.tag === 10) {
        var x = d.alternate;
        if (x === null) throw Error(u(387));
        if (x = x.memoizedProps, x !== null) {
          var q = d.type;
          Gt(d.pendingProps.value, x.value) || (e !== null ? e.push(q) : e = [q]);
        }
      } else if (d === ae.current) {
        if (x = d.alternate, x === null) throw Error(u(387));
        x.memoizedState.memoizedState !== d.memoizedState.memoizedState && (e !== null ? e.push(hu) : e = [hu]);
      }
      d = d.return;
    }
    e !== null && ac(
      n,
      e,
      a,
      l
    ), n.flags |= 262144;
  }
  function xl(e) {
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
  function Zr(e) {
    Qr = e, Bn = null, e = e.dependencies, e !== null && (e.firstContext = null);
  }
  function mt(e) {
    return n0(Qr, e);
  }
  function Sl(e, n) {
    return Qr === null && Zr(e), n0(e, n);
  }
  function n0(e, n) {
    var a = n._currentValue;
    if (n = { context: n, memoizedValue: a, next: null }, Bn === null) {
      if (e === null) throw Error(u(308));
      Bn = n, e.dependencies = { lanes: 0, firstContext: n }, e.flags |= 524288;
    } else Bn = Bn.next = n;
    return a;
  }
  var bC = typeof AbortController < "u" ? AbortController : function() {
    var e = [], n = this.signal = {
      aborted: !1,
      addEventListener: function(a, l) {
        e.push(l);
      }
    };
    this.abort = function() {
      n.aborted = !0, e.forEach(function(a) {
        return a();
      });
    };
  }, _C = t.unstable_scheduleCallback, xC = t.unstable_NormalPriority, it = {
    $$typeof: S,
    Consumer: null,
    Provider: null,
    _currentValue: null,
    _currentValue2: null,
    _threadCount: 0
  };
  function ic() {
    return {
      controller: new bC(),
      data: /* @__PURE__ */ new Map(),
      refCount: 0
    };
  }
  function Vi(e) {
    e.refCount--, e.refCount === 0 && _C(xC, function() {
      e.controller.abort();
    });
  }
  var Yi = null, uc = 0, Ba = 0, ja = null;
  function SC(e, n) {
    if (Yi === null) {
      var a = Yi = [];
      uc = 0, Ba = cf(), ja = {
        status: "pending",
        value: void 0,
        then: function(l) {
          a.push(l);
        }
      };
    }
    return uc++, n.then(r0, r0), n;
  }
  function r0() {
    if (--uc === 0 && Yi !== null) {
      ja !== null && (ja.status = "fulfilled");
      var e = Yi;
      Yi = null, Ba = 0, ja = null;
      for (var n = 0; n < e.length; n++) (0, e[n])();
    }
  }
  function EC(e, n) {
    var a = [], l = {
      status: "pending",
      value: null,
      reason: null,
      then: function(d) {
        a.push(d);
      }
    };
    return e.then(
      function() {
        l.status = "fulfilled", l.value = n;
        for (var d = 0; d < a.length; d++) (0, a[d])(n);
      },
      function(d) {
        for (l.status = "rejected", l.reason = d, d = 0; d < a.length; d++)
          (0, a[d])(void 0);
      }
    ), l;
  }
  var a0 = N.S;
  N.S = function(e, n) {
    V1 = ke(), typeof n == "object" && n !== null && typeof n.then == "function" && SC(e, n), a0 !== null && a0(e, n);
  };
  var Kr = z(null);
  function lc() {
    var e = Kr.current;
    return e !== null ? e : Ye.pooledCache;
  }
  function El(e, n) {
    n === null ? ie(Kr, Kr.current) : ie(Kr, n.pool);
  }
  function i0() {
    var e = lc();
    return e === null ? null : { parent: it._currentValue, pool: e };
  }
  var Ua = Error(u(460)), oc = Error(u(474)), wl = Error(u(542)), Al = { then: function() {
  } };
  function u0(e) {
    return e = e.status, e === "fulfilled" || e === "rejected";
  }
  function l0(e, n, a) {
    switch (a = e[a], a === void 0 ? e.push(n) : a !== n && (n.then(zn, zn), n = a), n.status) {
      case "fulfilled":
        return n.value;
      case "rejected":
        throw e = n.reason, s0(e), e;
      default:
        if (typeof n.status == "string") n.then(zn, zn);
        else {
          if (e = Ye, e !== null && 100 < e.shellSuspendCounter)
            throw Error(u(482));
          e = n, e.status = "pending", e.then(
            function(l) {
              if (n.status === "pending") {
                var d = n;
                d.status = "fulfilled", d.value = l;
              }
            },
            function(l) {
              if (n.status === "pending") {
                var d = n;
                d.status = "rejected", d.reason = l;
              }
            }
          );
        }
        switch (n.status) {
          case "fulfilled":
            return n.value;
          case "rejected":
            throw e = n.reason, s0(e), e;
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
  function o0() {
    if (Fr === null) throw Error(u(459));
    var e = Fr;
    return Fr = null, e;
  }
  function s0(e) {
    if (e === Ua || e === wl)
      throw Error(u(483));
  }
  var Ga = null, ki = 0;
  function Tl(e) {
    var n = ki;
    return ki += 1, Ga === null && (Ga = []), l0(Ga, e, n);
  }
  function Xi(e, n) {
    n = n.props.ref, e.ref = n !== void 0 ? n : null;
  }
  function Ml(e, n) {
    throw n.$$typeof === m ? Error(u(525)) : (e = Object.prototype.toString.call(n), Error(
      u(
        31,
        e === "[object Object]" ? "object with keys {" + Object.keys(n).join(", ") + "}" : e
      )
    ));
  }
  function c0(e) {
    function n($, k) {
      if (e) {
        var ee = $.deletions;
        ee === null ? ($.deletions = [k], $.flags |= 16) : ee.push(k);
      }
    }
    function a($, k) {
      if (!e) return null;
      for (; k !== null; )
        n($, k), k = k.sibling;
      return null;
    }
    function l($) {
      for (var k = /* @__PURE__ */ new Map(); $ !== null; )
        $.key !== null ? k.set($.key, $) : k.set($.index, $), $ = $.sibling;
      return k;
    }
    function d($, k) {
      return $ = Hn($, k), $.index = 0, $.sibling = null, $;
    }
    function y($, k, ee) {
      return $.index = ee, e ? (ee = $.alternate, ee !== null ? (ee = ee.index, ee < k ? ($.flags |= 67108866, k) : ee) : ($.flags |= 67108866, k)) : ($.flags |= 1048576, k);
    }
    function x($) {
      return e && $.alternate === null && ($.flags |= 67108866), $;
    }
    function q($, k, ee, ce) {
      return k === null || k.tag !== 6 ? (k = Fs(ee, $.mode, ce), k.return = $, k) : (k = d(k, ee), k.return = $, k);
    }
    function U($, k, ee, ce) {
      var Se = ee.type;
      return Se === A ? oe(
        $,
        k,
        ee.props.children,
        ce,
        ee.key
      ) : k !== null && (k.elementType === Se || typeof Se == "object" && Se !== null && Se.$$typeof === H && $r(Se) === k.type) ? (k = d(k, ee.props), Xi(k, ee), k.return = $, k) : (k = bl(
        ee.type,
        ee.key,
        ee.props,
        null,
        $.mode,
        ce
      ), Xi(k, ee), k.return = $, k);
    }
    function te($, k, ee, ce) {
      return k === null || k.tag !== 4 || k.stateNode.containerInfo !== ee.containerInfo || k.stateNode.implementation !== ee.implementation ? (k = Js(ee, $.mode, ce), k.return = $, k) : (k = d(k, ee.children || []), k.return = $, k);
    }
    function oe($, k, ee, ce, Se) {
      return k === null || k.tag !== 7 ? (k = Xr(
        ee,
        $.mode,
        ce,
        Se
      ), k.return = $, k) : (k = d(k, ee), k.return = $, k);
    }
    function fe($, k, ee) {
      if (typeof k == "string" && k !== "" || typeof k == "number" || typeof k == "bigint")
        return k = Fs(
          "" + k,
          $.mode,
          ee
        ), k.return = $, k;
      if (typeof k == "object" && k !== null) {
        switch (k.$$typeof) {
          case b:
            return ee = bl(
              k.type,
              k.key,
              k.props,
              null,
              $.mode,
              ee
            ), Xi(ee, k), ee.return = $, ee;
          case _:
            return k = Js(
              k,
              $.mode,
              ee
            ), k.return = $, k;
          case H:
            return k = $r(k), fe($, k, ee);
        }
        if (G(k) || F(k))
          return k = Xr(
            k,
            $.mode,
            ee,
            null
          ), k.return = $, k;
        if (typeof k.then == "function")
          return fe($, Tl(k), ee);
        if (k.$$typeof === S)
          return fe(
            $,
            Sl($, k),
            ee
          );
        Ml($, k);
      }
      return null;
    }
    function ne($, k, ee, ce) {
      var Se = k !== null ? k.key : null;
      if (typeof ee == "string" && ee !== "" || typeof ee == "number" || typeof ee == "bigint")
        return Se !== null ? null : q($, k, "" + ee, ce);
      if (typeof ee == "object" && ee !== null) {
        switch (ee.$$typeof) {
          case b:
            return ee.key === Se ? U($, k, ee, ce) : null;
          case _:
            return ee.key === Se ? te($, k, ee, ce) : null;
          case H:
            return ee = $r(ee), ne($, k, ee, ce);
        }
        if (G(ee) || F(ee))
          return Se !== null ? null : oe($, k, ee, ce, null);
        if (typeof ee.then == "function")
          return ne(
            $,
            k,
            Tl(ee),
            ce
          );
        if (ee.$$typeof === S)
          return ne(
            $,
            k,
            Sl($, ee),
            ce
          );
        Ml($, ee);
      }
      return null;
    }
    function ue($, k, ee, ce, Se) {
      if (typeof ce == "string" && ce !== "" || typeof ce == "number" || typeof ce == "bigint")
        return $ = $.get(ee) || null, q(k, $, "" + ce, Se);
      if (typeof ce == "object" && ce !== null) {
        switch (ce.$$typeof) {
          case b:
            return $ = $.get(
              ce.key === null ? ee : ce.key
            ) || null, U(k, $, ce, Se);
          case _:
            return $ = $.get(
              ce.key === null ? ee : ce.key
            ) || null, te(k, $, ce, Se);
          case H:
            return ce = $r(ce), ue(
              $,
              k,
              ee,
              ce,
              Se
            );
        }
        if (G(ce) || F(ce))
          return $ = $.get(ee) || null, oe(k, $, ce, Se, null);
        if (typeof ce.then == "function")
          return ue(
            $,
            k,
            ee,
            Tl(ce),
            Se
          );
        if (ce.$$typeof === S)
          return ue(
            $,
            k,
            ee,
            Sl(k, ce),
            Se
          );
        Ml(k, ce);
      }
      return null;
    }
    function ye($, k, ee, ce) {
      for (var Se = null, De = null, be = k, Me = k = 0, Ne = null; be !== null && Me < ee.length; Me++) {
        be.index > Me ? (Ne = be, be = null) : Ne = be.sibling;
        var He = ne(
          $,
          be,
          ee[Me],
          ce
        );
        if (He === null) {
          be === null && (be = Ne);
          break;
        }
        e && be && He.alternate === null && n($, be), k = y(He, k, Me), De === null ? Se = He : De.sibling = He, De = He, be = Ne;
      }
      if (Me === ee.length)
        return a($, be), Oe && Ln($, Me), Se;
      if (be === null) {
        for (; Me < ee.length; Me++)
          be = fe($, ee[Me], ce), be !== null && (k = y(
            be,
            k,
            Me
          ), De === null ? Se = be : De.sibling = be, De = be);
        return Oe && Ln($, Me), Se;
      }
      for (be = l(be); Me < ee.length; Me++)
        Ne = ue(
          be,
          $,
          Me,
          ee[Me],
          ce
        ), Ne !== null && (e && Ne.alternate !== null && be.delete(
          Ne.key === null ? Me : Ne.key
        ), k = y(
          Ne,
          k,
          Me
        ), De === null ? Se = Ne : De.sibling = Ne, De = Ne);
      return e && be.forEach(function(Rr) {
        return n($, Rr);
      }), Oe && Ln($, Me), Se;
    }
    function Ee($, k, ee, ce) {
      if (ee == null) throw Error(u(151));
      for (var Se = null, De = null, be = k, Me = k = 0, Ne = null, He = ee.next(); be !== null && !He.done; Me++, He = ee.next()) {
        be.index > Me ? (Ne = be, be = null) : Ne = be.sibling;
        var Rr = ne($, be, He.value, ce);
        if (Rr === null) {
          be === null && (be = Ne);
          break;
        }
        e && be && Rr.alternate === null && n($, be), k = y(Rr, k, Me), De === null ? Se = Rr : De.sibling = Rr, De = Rr, be = Ne;
      }
      if (He.done)
        return a($, be), Oe && Ln($, Me), Se;
      if (be === null) {
        for (; !He.done; Me++, He = ee.next())
          He = fe($, He.value, ce), He !== null && (k = y(He, k, Me), De === null ? Se = He : De.sibling = He, De = He);
        return Oe && Ln($, Me), Se;
      }
      for (be = l(be); !He.done; Me++, He = ee.next())
        He = ue(be, $, Me, He.value, ce), He !== null && (e && He.alternate !== null && be.delete(He.key === null ? Me : He.key), k = y(He, k, Me), De === null ? Se = He : De.sibling = He, De = He);
      return e && be.forEach(function(DR) {
        return n($, DR);
      }), Oe && Ln($, Me), Se;
    }
    function Ve($, k, ee, ce) {
      if (typeof ee == "object" && ee !== null && ee.type === A && ee.key === null && (ee = ee.props.children), typeof ee == "object" && ee !== null) {
        switch (ee.$$typeof) {
          case b:
            e: {
              for (var Se = ee.key; k !== null; ) {
                if (k.key === Se) {
                  if (Se = ee.type, Se === A) {
                    if (k.tag === 7) {
                      a(
                        $,
                        k.sibling
                      ), ce = d(
                        k,
                        ee.props.children
                      ), ce.return = $, $ = ce;
                      break e;
                    }
                  } else if (k.elementType === Se || typeof Se == "object" && Se !== null && Se.$$typeof === H && $r(Se) === k.type) {
                    a(
                      $,
                      k.sibling
                    ), ce = d(k, ee.props), Xi(ce, ee), ce.return = $, $ = ce;
                    break e;
                  }
                  a($, k);
                  break;
                } else n($, k);
                k = k.sibling;
              }
              ee.type === A ? (ce = Xr(
                ee.props.children,
                $.mode,
                ce,
                ee.key
              ), ce.return = $, $ = ce) : (ce = bl(
                ee.type,
                ee.key,
                ee.props,
                null,
                $.mode,
                ce
              ), Xi(ce, ee), ce.return = $, $ = ce);
            }
            return x($);
          case _:
            e: {
              for (Se = ee.key; k !== null; ) {
                if (k.key === Se)
                  if (k.tag === 4 && k.stateNode.containerInfo === ee.containerInfo && k.stateNode.implementation === ee.implementation) {
                    a(
                      $,
                      k.sibling
                    ), ce = d(k, ee.children || []), ce.return = $, $ = ce;
                    break e;
                  } else {
                    a($, k);
                    break;
                  }
                else n($, k);
                k = k.sibling;
              }
              ce = Js(ee, $.mode, ce), ce.return = $, $ = ce;
            }
            return x($);
          case H:
            return ee = $r(ee), Ve(
              $,
              k,
              ee,
              ce
            );
        }
        if (G(ee))
          return ye(
            $,
            k,
            ee,
            ce
          );
        if (F(ee)) {
          if (Se = F(ee), typeof Se != "function") throw Error(u(150));
          return ee = Se.call(ee), Ee(
            $,
            k,
            ee,
            ce
          );
        }
        if (typeof ee.then == "function")
          return Ve(
            $,
            k,
            Tl(ee),
            ce
          );
        if (ee.$$typeof === S)
          return Ve(
            $,
            k,
            Sl($, ee),
            ce
          );
        Ml($, ee);
      }
      return typeof ee == "string" && ee !== "" || typeof ee == "number" || typeof ee == "bigint" ? (ee = "" + ee, k !== null && k.tag === 6 ? (a($, k.sibling), ce = d(k, ee), ce.return = $, $ = ce) : (a($, k), ce = Fs(ee, $.mode, ce), ce.return = $, $ = ce), x($)) : a($, k);
    }
    return function($, k, ee, ce) {
      try {
        ki = 0;
        var Se = Ve(
          $,
          k,
          ee,
          ce
        );
        return Ga = null, Se;
      } catch (be) {
        if (be === Ua || be === wl) throw be;
        var De = Vt(29, be, null, $.mode);
        return De.lanes = ce, De.return = $, De;
      } finally {
      }
    };
  }
  var Jr = c0(!0), f0 = c0(!1), hr = !1;
  function sc(e) {
    e.updateQueue = {
      baseState: e.memoizedState,
      firstBaseUpdate: null,
      lastBaseUpdate: null,
      shared: { pending: null, lanes: 0, hiddenCallbacks: null },
      callbacks: null
    };
  }
  function cc(e, n) {
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
    var l = e.updateQueue;
    if (l === null) return null;
    if (l = l.shared, (Le & 2) !== 0) {
      var d = l.pending;
      return d === null ? n.next = n : (n.next = d.next, d.next = n), l.pending = n, n = ml(e), Km(e, null, a), n;
    }
    return pl(e, l, n, a), ml(e);
  }
  function Ii(e, n, a) {
    if (n = n.updateQueue, n !== null && (n = n.shared, (a & 4194048) !== 0)) {
      var l = n.lanes;
      l &= e.pendingLanes, a |= l, n.lanes = a, Ju(e, a);
    }
  }
  function fc(e, n) {
    var a = e.updateQueue, l = e.alternate;
    if (l !== null && (l = l.updateQueue, a === l)) {
      var d = null, y = null;
      if (a = a.firstBaseUpdate, a !== null) {
        do {
          var x = {
            lane: a.lane,
            tag: a.tag,
            payload: a.payload,
            callback: null,
            next: null
          };
          y === null ? d = y = x : y = y.next = x, a = a.next;
        } while (a !== null);
        y === null ? d = y = n : y = y.next = n;
      } else d = y = n;
      a = {
        baseState: l.baseState,
        firstBaseUpdate: d,
        lastBaseUpdate: y,
        shared: l.shared,
        callbacks: l.callbacks
      }, e.updateQueue = a;
      return;
    }
    e = a.lastBaseUpdate, e === null ? a.firstBaseUpdate = n : e.next = n, a.lastBaseUpdate = n;
  }
  var dc = !1;
  function Qi() {
    if (dc) {
      var e = ja;
      if (e !== null) throw e;
    }
  }
  function Zi(e, n, a, l) {
    dc = !1;
    var d = e.updateQueue;
    hr = !1;
    var y = d.firstBaseUpdate, x = d.lastBaseUpdate, q = d.shared.pending;
    if (q !== null) {
      d.shared.pending = null;
      var U = q, te = U.next;
      U.next = null, x === null ? y = te : x.next = te, x = U;
      var oe = e.alternate;
      oe !== null && (oe = oe.updateQueue, q = oe.lastBaseUpdate, q !== x && (q === null ? oe.firstBaseUpdate = te : q.next = te, oe.lastBaseUpdate = U));
    }
    if (y !== null) {
      var fe = d.baseState;
      x = 0, oe = te = U = null, q = y;
      do {
        var ne = q.lane & -536870913, ue = ne !== q.lane;
        if (ue ? (Re & ne) === ne : (l & ne) === ne) {
          ne !== 0 && ne === Ba && (dc = !0), oe !== null && (oe = oe.next = {
            lane: 0,
            tag: q.tag,
            payload: q.payload,
            callback: null,
            next: null
          });
          e: {
            var ye = e, Ee = q;
            ne = n;
            var Ve = a;
            switch (Ee.tag) {
              case 1:
                if (ye = Ee.payload, typeof ye == "function") {
                  fe = ye.call(Ve, fe, ne);
                  break e;
                }
                fe = ye;
                break e;
              case 3:
                ye.flags = ye.flags & -65537 | 128;
              case 0:
                if (ye = Ee.payload, ne = typeof ye == "function" ? ye.call(Ve, fe, ne) : ye, ne == null) break e;
                fe = p({}, fe, ne);
                break e;
              case 2:
                hr = !0;
            }
          }
          ne = q.callback, ne !== null && (e.flags |= 64, ue && (e.flags |= 8192), ue = d.callbacks, ue === null ? d.callbacks = [ne] : ue.push(ne));
        } else
          ue = {
            lane: ne,
            tag: q.tag,
            payload: q.payload,
            callback: q.callback,
            next: null
          }, oe === null ? (te = oe = ue, U = fe) : oe = oe.next = ue, x |= ne;
        if (q = q.next, q === null) {
          if (q = d.shared.pending, q === null)
            break;
          ue = q, q = ue.next, ue.next = null, d.lastBaseUpdate = ue, d.shared.pending = null;
        }
      } while (!0);
      oe === null && (U = fe), d.baseState = U, d.firstBaseUpdate = te, d.lastBaseUpdate = oe, y === null && (d.shared.lanes = 0), _r |= x, e.lanes = x, e.memoizedState = fe;
    }
  }
  function d0(e, n) {
    if (typeof e != "function")
      throw Error(u(191, e));
    e.call(n);
  }
  function h0(e, n) {
    var a = e.callbacks;
    if (a !== null)
      for (e.callbacks = null, e = 0; e < a.length; e++)
        d0(a[e], n);
  }
  var Va = z(null), ql = z(0);
  function g0(e, n) {
    e = Zn, ie(ql, e), ie(Va, n), Zn = e | n.baseLanes;
  }
  function hc() {
    ie(ql, Zn), ie(Va, Va.current);
  }
  function gc() {
    Zn = ql.current, V(Va), V(ql);
  }
  var Yt = z(null), en = null;
  function yr(e) {
    var n = e.alternate;
    ie(nt, nt.current & 1), ie(Yt, e), en === null && (n === null || Va.current !== null || n.memoizedState !== null) && (en = e);
  }
  function vc(e) {
    ie(nt, nt.current), ie(Yt, e), en === null && (en = e);
  }
  function v0(e) {
    e.tag === 22 ? (ie(nt, nt.current), ie(Yt, e), en === null && (en = e)) : pr();
  }
  function pr() {
    ie(nt, nt.current), ie(Yt, Yt.current);
  }
  function kt(e) {
    V(Yt), en === e && (en = null), V(nt);
  }
  var nt = z(0);
  function Cl(e) {
    for (var n = e; n !== null; ) {
      if (n.tag === 13) {
        var a = n.memoizedState;
        if (a !== null && (a = a.dehydrated, a === null || Sf(a) || Ef(a)))
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
  var Un = 0, Te = null, Ue = null, ut = null, Rl = !1, Ya = !1, Pr = !1, Nl = 0, Ki = 0, ka = null, wC = 0;
  function We() {
    throw Error(u(321));
  }
  function yc(e, n) {
    if (n === null) return !1;
    for (var a = 0; a < n.length && a < e.length; a++)
      if (!Gt(e[a], n[a])) return !1;
    return !0;
  }
  function pc(e, n, a, l, d, y) {
    return Un = y, Te = n, n.memoizedState = null, n.updateQueue = null, n.lanes = 0, N.H = e === null || e.memoizedState === null ? P0 : Oc, Pr = !1, y = a(l, d), Pr = !1, Ya && (y = p0(
      n,
      a,
      l,
      d
    )), y0(e), y;
  }
  function y0(e) {
    N.H = Ji;
    var n = Ue !== null && Ue.next !== null;
    if (Un = 0, ut = Ue = Te = null, Rl = !1, Ki = 0, ka = null, n) throw Error(u(300));
    e === null || lt || (e = e.dependencies, e !== null && xl(e) && (lt = !0));
  }
  function p0(e, n, a, l) {
    Te = e;
    var d = 0;
    do {
      if (Ya && (ka = null), Ki = 0, Ya = !1, 25 <= d) throw Error(u(301));
      if (d += 1, ut = Ue = null, e.updateQueue != null) {
        var y = e.updateQueue;
        y.lastEffect = null, y.events = null, y.stores = null, y.memoCache != null && (y.memoCache.index = 0);
      }
      N.H = W0, y = n(a, l);
    } while (Ya);
    return y;
  }
  function AC() {
    var e = N.H, n = e.useState()[0];
    return n = typeof n.then == "function" ? $i(n) : n, e = e.useState()[0], (Ue !== null ? Ue.memoizedState : null) !== e && (Te.flags |= 1024), n;
  }
  function mc() {
    var e = Nl !== 0;
    return Nl = 0, e;
  }
  function bc(e, n, a) {
    n.updateQueue = e.updateQueue, n.flags &= -2053, e.lanes &= ~a;
  }
  function _c(e) {
    if (Rl) {
      for (e = e.memoizedState; e !== null; ) {
        var n = e.queue;
        n !== null && (n.pending = null), e = e.next;
      }
      Rl = !1;
    }
    Un = 0, ut = Ue = Te = null, Ya = !1, Ki = Nl = 0, ka = null;
  }
  function qt() {
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
    var n = Ki;
    return Ki += 1, ka === null && (ka = []), e = l0(ka, e, n), n = Te, (ut === null ? n.memoizedState : ut.next) === null && (n = n.alternate, N.H = n === null || n.memoizedState === null ? P0 : Oc), e;
  }
  function zl(e) {
    if (e !== null && typeof e == "object") {
      if (typeof e.then == "function") return $i(e);
      if (e.$$typeof === S) return mt(e);
    }
    throw Error(u(438, String(e)));
  }
  function xc(e) {
    var n = null, a = Te.updateQueue;
    if (a !== null && (n = a.memoCache), n == null) {
      var l = Te.alternate;
      l !== null && (l = l.updateQueue, l !== null && (l = l.memoCache, l != null && (n = {
        data: l.data.map(function(d) {
          return d.slice();
        }),
        index: 0
      })));
    }
    if (n == null && (n = { data: [], index: 0 }), a === null && (a = Ol(), Te.updateQueue = a), a.memoCache = n, a = n.data[n.index], a === void 0)
      for (a = n.data[n.index] = Array(e), l = 0; l < e; l++)
        a[l] = X;
    return n.index++, a;
  }
  function Gn(e, n) {
    return typeof n == "function" ? n(e) : n;
  }
  function Dl(e) {
    var n = rt();
    return Sc(n, Ue, e);
  }
  function Sc(e, n, a) {
    var l = e.queue;
    if (l === null) throw Error(u(311));
    l.lastRenderedReducer = a;
    var d = e.baseQueue, y = l.pending;
    if (y !== null) {
      if (d !== null) {
        var x = d.next;
        d.next = y.next, y.next = x;
      }
      n.baseQueue = d = y, l.pending = null;
    }
    if (y = e.baseState, d === null) e.memoizedState = y;
    else {
      n = d.next;
      var q = x = null, U = null, te = n, oe = !1;
      do {
        var fe = te.lane & -536870913;
        if (fe !== te.lane ? (Re & fe) === fe : (Un & fe) === fe) {
          var ne = te.revertLane;
          if (ne === 0)
            U !== null && (U = U.next = {
              lane: 0,
              revertLane: 0,
              gesture: null,
              action: te.action,
              hasEagerState: te.hasEagerState,
              eagerState: te.eagerState,
              next: null
            }), fe === Ba && (oe = !0);
          else if ((Un & ne) === ne) {
            te = te.next, ne === Ba && (oe = !0);
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
            }, U === null ? (q = U = fe, x = y) : U = U.next = fe, Te.lanes |= ne, _r |= ne;
          fe = te.action, Pr && a(y, fe), y = te.hasEagerState ? te.eagerState : a(y, fe);
        } else
          ne = {
            lane: fe,
            revertLane: te.revertLane,
            gesture: te.gesture,
            action: te.action,
            hasEagerState: te.hasEagerState,
            eagerState: te.eagerState,
            next: null
          }, U === null ? (q = U = ne, x = y) : U = U.next = ne, Te.lanes |= fe, _r |= fe;
        te = te.next;
      } while (te !== null && te !== n);
      if (U === null ? x = y : U.next = q, !Gt(y, e.memoizedState) && (lt = !0, oe && (a = ja, a !== null)))
        throw a;
      e.memoizedState = y, e.baseState = x, e.baseQueue = U, l.lastRenderedState = y;
    }
    return d === null && (l.lanes = 0), [e.memoizedState, l.dispatch];
  }
  function Ec(e) {
    var n = rt(), a = n.queue;
    if (a === null) throw Error(u(311));
    a.lastRenderedReducer = e;
    var l = a.dispatch, d = a.pending, y = n.memoizedState;
    if (d !== null) {
      a.pending = null;
      var x = d = d.next;
      do
        y = e(y, x.action), x = x.next;
      while (x !== d);
      Gt(y, n.memoizedState) || (lt = !0), n.memoizedState = y, n.baseQueue === null && (n.baseState = y), a.lastRenderedState = y;
    }
    return [y, l];
  }
  function m0(e, n, a) {
    var l = Te, d = rt(), y = Oe;
    if (y) {
      if (a === void 0) throw Error(u(407));
      a = a();
    } else a = n();
    var x = !Gt(
      (Ue || d).memoizedState,
      a
    );
    if (x && (d.memoizedState = a, lt = !0), d = d.queue, Tc(x0.bind(null, l, d, e), [
      e
    ]), d.getSnapshot !== n || x || ut !== null && ut.memoizedState.tag & 1) {
      if (l.flags |= 2048, Xa(
        9,
        { destroy: void 0 },
        _0.bind(
          null,
          l,
          d,
          a,
          n
        ),
        null
      ), Ye === null) throw Error(u(349));
      y || (Un & 127) !== 0 || b0(l, n, a);
    }
    return a;
  }
  function b0(e, n, a) {
    e.flags |= 16384, e = { getSnapshot: n, value: a }, n = Te.updateQueue, n === null ? (n = Ol(), Te.updateQueue = n, n.stores = [e]) : (a = n.stores, a === null ? n.stores = [e] : a.push(e));
  }
  function _0(e, n, a, l) {
    n.value = a, n.getSnapshot = l, S0(n) && E0(e);
  }
  function x0(e, n, a) {
    return a(function() {
      S0(n) && E0(e);
    });
  }
  function S0(e) {
    var n = e.getSnapshot;
    e = e.value;
    try {
      var a = n();
      return !Gt(e, a);
    } catch {
      return !0;
    }
  }
  function E0(e) {
    var n = kr(e, 2);
    n !== null && Bt(n, e, 2);
  }
  function wc(e) {
    var n = qt();
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
  function w0(e, n, a, l) {
    return e.baseState = a, Sc(
      e,
      Ue,
      typeof l == "function" ? l : Gn
    );
  }
  function TC(e, n, a, l, d) {
    if (Bl(e)) throw Error(u(485));
    if (e = n.action, e !== null) {
      var y = {
        payload: d,
        action: e,
        next: null,
        isTransition: !0,
        status: "pending",
        value: null,
        reason: null,
        listeners: [],
        then: function(x) {
          y.listeners.push(x);
        }
      };
      N.T !== null ? a(!0) : y.isTransition = !1, l(y), a = n.pending, a === null ? (y.next = n.pending = y, A0(n, y)) : (y.next = a.next, n.pending = a.next = y);
    }
  }
  function A0(e, n) {
    var a = n.action, l = n.payload, d = e.state;
    if (n.isTransition) {
      var y = N.T, x = {};
      N.T = x;
      try {
        var q = a(d, l), U = N.S;
        U !== null && U(x, q), T0(e, n, q);
      } catch (te) {
        Ac(e, n, te);
      } finally {
        y !== null && x.types !== null && (y.types = x.types), N.T = y;
      }
    } else
      try {
        y = a(d, l), T0(e, n, y);
      } catch (te) {
        Ac(e, n, te);
      }
  }
  function T0(e, n, a) {
    a !== null && typeof a == "object" && typeof a.then == "function" ? a.then(
      function(l) {
        M0(e, n, l);
      },
      function(l) {
        return Ac(e, n, l);
      }
    ) : M0(e, n, a);
  }
  function M0(e, n, a) {
    n.status = "fulfilled", n.value = a, q0(n), e.state = a, n = e.pending, n !== null && (a = n.next, a === n ? e.pending = null : (a = a.next, n.next = a, A0(e, a)));
  }
  function Ac(e, n, a) {
    var l = e.pending;
    if (e.pending = null, l !== null) {
      l = l.next;
      do
        n.status = "rejected", n.reason = a, q0(n), n = n.next;
      while (n !== l);
    }
    e.action = null;
  }
  function q0(e) {
    e = e.listeners;
    for (var n = 0; n < e.length; n++) (0, e[n])();
  }
  function C0(e, n) {
    return n;
  }
  function R0(e, n) {
    if (Oe) {
      var a = Ye.formState;
      if (a !== null) {
        e: {
          var l = Te;
          if (Oe) {
            if (Xe) {
              t: {
                for (var d = Xe, y = Wt; d.nodeType !== 8; ) {
                  if (!y) {
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
                y = d.data, d = y === "F!" || y === "F" ? d : null;
              }
              if (d) {
                Xe = tn(
                  d.nextSibling
                ), l = d.data === "F!";
                break e;
              }
            }
            fr(l);
          }
          l = !1;
        }
        l && (n = a[0]);
      }
    }
    return a = qt(), a.memoizedState = a.baseState = n, l = {
      pending: null,
      lanes: 0,
      dispatch: null,
      lastRenderedReducer: C0,
      lastRenderedState: n
    }, a.queue = l, a = $0.bind(
      null,
      Te,
      l
    ), l.dispatch = a, l = wc(!1), y = Nc.bind(
      null,
      Te,
      !1,
      l.queue
    ), l = qt(), d = {
      state: n,
      dispatch: null,
      action: e,
      pending: null
    }, l.queue = d, a = TC.bind(
      null,
      Te,
      d,
      y,
      a
    ), d.dispatch = a, l.memoizedState = e, [n, a, !1];
  }
  function N0(e) {
    var n = rt();
    return O0(n, Ue, e);
  }
  function O0(e, n, a) {
    if (n = Sc(
      e,
      n,
      C0
    )[0], e = Dl(Gn)[0], typeof n == "object" && n !== null && typeof n.then == "function")
      try {
        var l = $i(n);
      } catch (x) {
        throw x === Ua ? wl : x;
      }
    else l = n;
    n = rt();
    var d = n.queue, y = d.dispatch;
    return a !== n.memoizedState && (Te.flags |= 2048, Xa(
      9,
      { destroy: void 0 },
      MC.bind(null, d, a),
      null
    )), [l, y, e];
  }
  function MC(e, n) {
    e.action = n;
  }
  function z0(e) {
    var n = rt(), a = Ue;
    if (a !== null)
      return O0(n, a, e);
    rt(), n = n.memoizedState, a = rt();
    var l = a.queue.dispatch;
    return a.memoizedState = e, [n, l, !1];
  }
  function Xa(e, n, a, l) {
    return e = { tag: e, create: a, deps: l, inst: n, next: null }, n = Te.updateQueue, n === null && (n = Ol(), Te.updateQueue = n), a = n.lastEffect, a === null ? n.lastEffect = e.next = e : (l = a.next, a.next = e, e.next = l, n.lastEffect = e), e;
  }
  function D0() {
    return rt().memoizedState;
  }
  function Hl(e, n, a, l) {
    var d = qt();
    Te.flags |= e, d.memoizedState = Xa(
      1 | n,
      { destroy: void 0 },
      a,
      l === void 0 ? null : l
    );
  }
  function Ll(e, n, a, l) {
    var d = rt();
    l = l === void 0 ? null : l;
    var y = d.memoizedState.inst;
    Ue !== null && l !== null && yc(l, Ue.memoizedState.deps) ? d.memoizedState = Xa(n, y, a, l) : (Te.flags |= e, d.memoizedState = Xa(
      1 | n,
      y,
      a,
      l
    ));
  }
  function H0(e, n) {
    Hl(8390656, 8, e, n);
  }
  function Tc(e, n) {
    Ll(2048, 8, e, n);
  }
  function qC(e) {
    Te.flags |= 4;
    var n = Te.updateQueue;
    if (n === null)
      n = Ol(), Te.updateQueue = n, n.events = [e];
    else {
      var a = n.events;
      a === null ? n.events = [e] : a.push(e);
    }
  }
  function L0(e) {
    var n = rt().memoizedState;
    return qC({ ref: n, nextImpl: e }), function() {
      if ((Le & 2) !== 0) throw Error(u(440));
      return n.impl.apply(void 0, arguments);
    };
  }
  function B0(e, n) {
    return Ll(4, 2, e, n);
  }
  function j0(e, n) {
    return Ll(4, 4, e, n);
  }
  function U0(e, n) {
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
  function G0(e, n, a) {
    a = a != null ? a.concat([e]) : null, Ll(4, 4, U0.bind(null, n, e), a);
  }
  function Mc() {
  }
  function V0(e, n) {
    var a = rt();
    n = n === void 0 ? null : n;
    var l = a.memoizedState;
    return n !== null && yc(n, l[1]) ? l[0] : (a.memoizedState = [e, n], e);
  }
  function Y0(e, n) {
    var a = rt();
    n = n === void 0 ? null : n;
    var l = a.memoizedState;
    if (n !== null && yc(n, l[1]))
      return l[0];
    if (l = e(), Pr) {
      mn(!0);
      try {
        e();
      } finally {
        mn(!1);
      }
    }
    return a.memoizedState = [l, n], l;
  }
  function qc(e, n, a) {
    return a === void 0 || (Un & 1073741824) !== 0 && (Re & 261930) === 0 ? e.memoizedState = n : (e.memoizedState = a, e = k1(), Te.lanes |= e, _r |= e, a);
  }
  function k0(e, n, a, l) {
    return Gt(a, n) ? a : Va.current !== null ? (e = qc(e, a, l), Gt(e, n) || (lt = !0), e) : (Un & 42) === 0 || (Un & 1073741824) !== 0 && (Re & 261930) === 0 ? (lt = !0, e.memoizedState = a) : (e = k1(), Te.lanes |= e, _r |= e, n);
  }
  function X0(e, n, a, l, d) {
    var y = j.p;
    j.p = y !== 0 && 8 > y ? y : 8;
    var x = N.T, q = {};
    N.T = q, Nc(e, !1, n, a);
    try {
      var U = d(), te = N.S;
      if (te !== null && te(q, U), U !== null && typeof U == "object" && typeof U.then == "function") {
        var oe = EC(
          U,
          l
        );
        Fi(
          e,
          n,
          oe,
          Qt(e)
        );
      } else
        Fi(
          e,
          n,
          l,
          Qt(e)
        );
    } catch (fe) {
      Fi(
        e,
        n,
        { then: function() {
        }, status: "rejected", reason: fe },
        Qt()
      );
    } finally {
      j.p = y, x !== null && q.types !== null && (x.types = q.types), N.T = x;
    }
  }
  function CC() {
  }
  function Cc(e, n, a, l) {
    if (e.tag !== 5) throw Error(u(476));
    var d = I0(e).queue;
    X0(
      e,
      d,
      n,
      Z,
      a === null ? CC : function() {
        return Q0(e), a(l);
      }
    );
  }
  function I0(e) {
    var n = e.memoizedState;
    if (n !== null) return n;
    n = {
      memoizedState: Z,
      baseState: Z,
      baseQueue: null,
      queue: {
        pending: null,
        lanes: 0,
        dispatch: null,
        lastRenderedReducer: Gn,
        lastRenderedState: Z
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
  function Q0(e) {
    var n = I0(e);
    n.next === null && (n = e.alternate.memoizedState), Fi(
      e,
      n.next.queue,
      {},
      Qt()
    );
  }
  function Rc() {
    return mt(hu);
  }
  function Z0() {
    return rt().memoizedState;
  }
  function K0() {
    return rt().memoizedState;
  }
  function RC(e) {
    for (var n = e.return; n !== null; ) {
      switch (n.tag) {
        case 24:
        case 3:
          var a = Qt();
          e = gr(a);
          var l = vr(n, e, a);
          l !== null && (Bt(l, n, a), Ii(l, n, a)), n = { cache: ic() }, e.payload = n;
          return;
      }
      n = n.return;
    }
  }
  function NC(e, n, a) {
    var l = Qt();
    a = {
      lane: l,
      revertLane: 0,
      gesture: null,
      action: a,
      hasEagerState: !1,
      eagerState: null,
      next: null
    }, Bl(e) ? F0(n, a) : (a = Ks(e, n, a, l), a !== null && (Bt(a, e, l), J0(a, n, l)));
  }
  function $0(e, n, a) {
    var l = Qt();
    Fi(e, n, a, l);
  }
  function Fi(e, n, a, l) {
    var d = {
      lane: l,
      revertLane: 0,
      gesture: null,
      action: a,
      hasEagerState: !1,
      eagerState: null,
      next: null
    };
    if (Bl(e)) F0(n, d);
    else {
      var y = e.alternate;
      if (e.lanes === 0 && (y === null || y.lanes === 0) && (y = n.lastRenderedReducer, y !== null))
        try {
          var x = n.lastRenderedState, q = y(x, a);
          if (d.hasEagerState = !0, d.eagerState = q, Gt(q, x))
            return pl(e, n, d, 0), Ye === null && yl(), !1;
        } catch {
        } finally {
        }
      if (a = Ks(e, n, d, l), a !== null)
        return Bt(a, e, l), J0(a, n, l), !0;
    }
    return !1;
  }
  function Nc(e, n, a, l) {
    if (l = {
      lane: 2,
      revertLane: cf(),
      gesture: null,
      action: l,
      hasEagerState: !1,
      eagerState: null,
      next: null
    }, Bl(e)) {
      if (n) throw Error(u(479));
    } else
      n = Ks(
        e,
        a,
        l,
        2
      ), n !== null && Bt(n, e, 2);
  }
  function Bl(e) {
    var n = e.alternate;
    return e === Te || n !== null && n === Te;
  }
  function F0(e, n) {
    Ya = Rl = !0;
    var a = e.pending;
    a === null ? n.next = n : (n.next = a.next, a.next = n), e.pending = n;
  }
  function J0(e, n, a) {
    if ((a & 4194048) !== 0) {
      var l = n.lanes;
      l &= e.pendingLanes, a |= l, n.lanes = a, Ju(e, a);
    }
  }
  var Ji = {
    readContext: mt,
    use: zl,
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
  var P0 = {
    readContext: mt,
    use: zl,
    useCallback: function(e, n) {
      return qt().memoizedState = [
        e,
        n === void 0 ? null : n
      ], e;
    },
    useContext: mt,
    useEffect: H0,
    useImperativeHandle: function(e, n, a) {
      a = a != null ? a.concat([e]) : null, Hl(
        4194308,
        4,
        U0.bind(null, n, e),
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
      var a = qt();
      n = n === void 0 ? null : n;
      var l = e();
      if (Pr) {
        mn(!0);
        try {
          e();
        } finally {
          mn(!1);
        }
      }
      return a.memoizedState = [l, n], l;
    },
    useReducer: function(e, n, a) {
      var l = qt();
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
      return l.memoizedState = l.baseState = d, e = {
        pending: null,
        lanes: 0,
        dispatch: null,
        lastRenderedReducer: e,
        lastRenderedState: d
      }, l.queue = e, e = e.dispatch = NC.bind(
        null,
        Te,
        e
      ), [l.memoizedState, e];
    },
    useRef: function(e) {
      var n = qt();
      return e = { current: e }, n.memoizedState = e;
    },
    useState: function(e) {
      e = wc(e);
      var n = e.queue, a = $0.bind(null, Te, n);
      return n.dispatch = a, [e.memoizedState, a];
    },
    useDebugValue: Mc,
    useDeferredValue: function(e, n) {
      var a = qt();
      return qc(a, e, n);
    },
    useTransition: function() {
      var e = wc(!1);
      return e = X0.bind(
        null,
        Te,
        e.queue,
        !0,
        !1
      ), qt().memoizedState = e, [!1, e];
    },
    useSyncExternalStore: function(e, n, a) {
      var l = Te, d = qt();
      if (Oe) {
        if (a === void 0)
          throw Error(u(407));
        a = a();
      } else {
        if (a = n(), Ye === null)
          throw Error(u(349));
        (Re & 127) !== 0 || b0(l, n, a);
      }
      d.memoizedState = a;
      var y = { value: a, getSnapshot: n };
      return d.queue = y, H0(x0.bind(null, l, y, e), [
        e
      ]), l.flags |= 2048, Xa(
        9,
        { destroy: void 0 },
        _0.bind(
          null,
          l,
          y,
          a,
          n
        ),
        null
      ), a;
    },
    useId: function() {
      var e = qt(), n = Ye.identifierPrefix;
      if (Oe) {
        var a = xn, l = _n;
        a = (l & ~(1 << 32 - Mt(l) - 1)).toString(32) + a, n = "_" + n + "R_" + a, a = Nl++, 0 < a && (n += "H" + a.toString(32)), n += "_";
      } else
        a = wC++, n = "_" + n + "r_" + a.toString(32) + "_";
      return e.memoizedState = n;
    },
    useHostTransitionStatus: Rc,
    useFormState: R0,
    useActionState: R0,
    useOptimistic: function(e) {
      var n = qt();
      n.memoizedState = n.baseState = e;
      var a = {
        pending: null,
        lanes: 0,
        dispatch: null,
        lastRenderedReducer: null,
        lastRenderedState: null
      };
      return n.queue = a, n = Nc.bind(
        null,
        Te,
        !0,
        a
      ), a.dispatch = n, [e, n];
    },
    useMemoCache: xc,
    useCacheRefresh: function() {
      return qt().memoizedState = RC.bind(
        null,
        Te
      );
    },
    useEffectEvent: function(e) {
      var n = qt(), a = { impl: e };
      return n.memoizedState = a, function() {
        if ((Le & 2) !== 0)
          throw Error(u(440));
        return a.impl.apply(void 0, arguments);
      };
    }
  }, Oc = {
    readContext: mt,
    use: zl,
    useCallback: V0,
    useContext: mt,
    useEffect: Tc,
    useImperativeHandle: G0,
    useInsertionEffect: B0,
    useLayoutEffect: j0,
    useMemo: Y0,
    useReducer: Dl,
    useRef: D0,
    useState: function() {
      return Dl(Gn);
    },
    useDebugValue: Mc,
    useDeferredValue: function(e, n) {
      var a = rt();
      return k0(
        a,
        Ue.memoizedState,
        e,
        n
      );
    },
    useTransition: function() {
      var e = Dl(Gn)[0], n = rt().memoizedState;
      return [
        typeof e == "boolean" ? e : $i(e),
        n
      ];
    },
    useSyncExternalStore: m0,
    useId: Z0,
    useHostTransitionStatus: Rc,
    useFormState: N0,
    useActionState: N0,
    useOptimistic: function(e, n) {
      var a = rt();
      return w0(a, Ue, e, n);
    },
    useMemoCache: xc,
    useCacheRefresh: K0
  };
  Oc.useEffectEvent = L0;
  var W0 = {
    readContext: mt,
    use: zl,
    useCallback: V0,
    useContext: mt,
    useEffect: Tc,
    useImperativeHandle: G0,
    useInsertionEffect: B0,
    useLayoutEffect: j0,
    useMemo: Y0,
    useReducer: Ec,
    useRef: D0,
    useState: function() {
      return Ec(Gn);
    },
    useDebugValue: Mc,
    useDeferredValue: function(e, n) {
      var a = rt();
      return Ue === null ? qc(a, e, n) : k0(
        a,
        Ue.memoizedState,
        e,
        n
      );
    },
    useTransition: function() {
      var e = Ec(Gn)[0], n = rt().memoizedState;
      return [
        typeof e == "boolean" ? e : $i(e),
        n
      ];
    },
    useSyncExternalStore: m0,
    useId: Z0,
    useHostTransitionStatus: Rc,
    useFormState: z0,
    useActionState: z0,
    useOptimistic: function(e, n) {
      var a = rt();
      return Ue !== null ? w0(a, Ue, e, n) : (a.baseState = e, [e, a.queue.dispatch]);
    },
    useMemoCache: xc,
    useCacheRefresh: K0
  };
  W0.useEffectEvent = L0;
  function zc(e, n, a, l) {
    n = e.memoizedState, a = a(l, n), a = a == null ? n : p({}, n, a), e.memoizedState = a, e.lanes === 0 && (e.updateQueue.baseState = a);
  }
  var Dc = {
    enqueueSetState: function(e, n, a) {
      e = e._reactInternals;
      var l = Qt(), d = gr(l);
      d.payload = n, a != null && (d.callback = a), n = vr(e, d, l), n !== null && (Bt(n, e, l), Ii(n, e, l));
    },
    enqueueReplaceState: function(e, n, a) {
      e = e._reactInternals;
      var l = Qt(), d = gr(l);
      d.tag = 1, d.payload = n, a != null && (d.callback = a), n = vr(e, d, l), n !== null && (Bt(n, e, l), Ii(n, e, l));
    },
    enqueueForceUpdate: function(e, n) {
      e = e._reactInternals;
      var a = Qt(), l = gr(a);
      l.tag = 2, n != null && (l.callback = n), n = vr(e, l, a), n !== null && (Bt(n, e, a), Ii(n, e, a));
    }
  };
  function e1(e, n, a, l, d, y, x) {
    return e = e.stateNode, typeof e.shouldComponentUpdate == "function" ? e.shouldComponentUpdate(l, y, x) : n.prototype && n.prototype.isPureReactComponent ? !Bi(a, l) || !Bi(d, y) : !0;
  }
  function t1(e, n, a, l) {
    e = n.state, typeof n.componentWillReceiveProps == "function" && n.componentWillReceiveProps(a, l), typeof n.UNSAFE_componentWillReceiveProps == "function" && n.UNSAFE_componentWillReceiveProps(a, l), n.state !== e && Dc.enqueueReplaceState(n, n.state, null);
  }
  function Wr(e, n) {
    var a = n;
    if ("ref" in n) {
      a = {};
      for (var l in n)
        l !== "ref" && (a[l] = n[l]);
    }
    if (e = e.defaultProps) {
      a === n && (a = p({}, a));
      for (var d in e)
        a[d] === void 0 && (a[d] = e[d]);
    }
    return a;
  }
  function n1(e) {
    vl(e);
  }
  function r1(e) {
    console.error(e);
  }
  function a1(e) {
    vl(e);
  }
  function jl(e, n) {
    try {
      var a = e.onUncaughtError;
      a(n.value, { componentStack: n.stack });
    } catch (l) {
      setTimeout(function() {
        throw l;
      });
    }
  }
  function i1(e, n, a) {
    try {
      var l = e.onCaughtError;
      l(a.value, {
        componentStack: a.stack,
        errorBoundary: n.tag === 1 ? n.stateNode : null
      });
    } catch (d) {
      setTimeout(function() {
        throw d;
      });
    }
  }
  function Hc(e, n, a) {
    return a = gr(a), a.tag = 3, a.payload = { element: null }, a.callback = function() {
      jl(e, n);
    }, a;
  }
  function u1(e) {
    return e = gr(e), e.tag = 3, e;
  }
  function l1(e, n, a, l) {
    var d = a.type.getDerivedStateFromError;
    if (typeof d == "function") {
      var y = l.value;
      e.payload = function() {
        return d(y);
      }, e.callback = function() {
        i1(n, a, l);
      };
    }
    var x = a.stateNode;
    x !== null && typeof x.componentDidCatch == "function" && (e.callback = function() {
      i1(n, a, l), typeof d != "function" && (xr === null ? xr = /* @__PURE__ */ new Set([this]) : xr.add(this));
      var q = l.stack;
      this.componentDidCatch(l.value, {
        componentStack: q !== null ? q : ""
      });
    });
  }
  function OC(e, n, a, l, d) {
    if (a.flags |= 32768, l !== null && typeof l == "object" && typeof l.then == "function") {
      if (n = a.alternate, n !== null && La(
        n,
        a,
        d,
        !0
      ), a = Yt.current, a !== null) {
        switch (a.tag) {
          case 31:
          case 13:
            return en === null ? Fl() : a.alternate === null && et === 0 && (et = 3), a.flags &= -257, a.flags |= 65536, a.lanes = d, l === Al ? a.flags |= 16384 : (n = a.updateQueue, n === null ? a.updateQueue = /* @__PURE__ */ new Set([l]) : n.add(l), lf(e, l, d)), !1;
          case 22:
            return a.flags |= 65536, l === Al ? a.flags |= 16384 : (n = a.updateQueue, n === null ? (n = {
              transitions: null,
              markerInstances: null,
              retryQueue: /* @__PURE__ */ new Set([l])
            }, a.updateQueue = n) : (a = n.retryQueue, a === null ? n.retryQueue = /* @__PURE__ */ new Set([l]) : a.add(l)), lf(e, l, d)), !1;
        }
        throw Error(u(435, a.tag));
      }
      return lf(e, l, d), Fl(), !1;
    }
    if (Oe)
      return n = Yt.current, n !== null ? ((n.flags & 65536) === 0 && (n.flags |= 256), n.flags |= 65536, n.lanes = d, l !== ec && (e = Error(u(422), { cause: l }), Gi(Ft(e, a)))) : (l !== ec && (n = Error(u(423), {
        cause: l
      }), Gi(
        Ft(n, a)
      )), e = e.current.alternate, e.flags |= 65536, d &= -d, e.lanes |= d, l = Ft(l, a), d = Hc(
        e.stateNode,
        l,
        d
      ), fc(e, d), et !== 4 && (et = 2)), !1;
    var y = Error(u(520), { cause: l });
    if (y = Ft(y, a), iu === null ? iu = [y] : iu.push(y), et !== 4 && (et = 2), n === null) return !0;
    l = Ft(l, a), a = n;
    do {
      switch (a.tag) {
        case 3:
          return a.flags |= 65536, e = d & -d, a.lanes |= e, e = Hc(a.stateNode, l, e), fc(a, e), !1;
        case 1:
          if (n = a.type, y = a.stateNode, (a.flags & 128) === 0 && (typeof n.getDerivedStateFromError == "function" || y !== null && typeof y.componentDidCatch == "function" && (xr === null || !xr.has(y))))
            return a.flags |= 65536, d &= -d, a.lanes |= d, d = u1(d), l1(
              d,
              e,
              a,
              l
            ), fc(a, d), !1;
      }
      a = a.return;
    } while (a !== null);
    return !1;
  }
  var Lc = Error(u(461)), lt = !1;
  function bt(e, n, a, l) {
    n.child = e === null ? f0(n, null, a, l) : Jr(
      n,
      e.child,
      a,
      l
    );
  }
  function o1(e, n, a, l, d) {
    a = a.render;
    var y = n.ref;
    if ("ref" in l) {
      var x = {};
      for (var q in l)
        q !== "ref" && (x[q] = l[q]);
    } else x = l;
    return Zr(n), l = pc(
      e,
      n,
      a,
      x,
      y,
      d
    ), q = mc(), e !== null && !lt ? (bc(e, n, d), Vn(e, n, d)) : (Oe && q && Ps(n), n.flags |= 1, bt(e, n, l, d), n.child);
  }
  function s1(e, n, a, l, d) {
    if (e === null) {
      var y = a.type;
      return typeof y == "function" && !$s(y) && y.defaultProps === void 0 && a.compare === null ? (n.tag = 15, n.type = y, c1(
        e,
        n,
        y,
        l,
        d
      )) : (e = bl(
        a.type,
        null,
        l,
        n,
        n.mode,
        d
      ), e.ref = n.ref, e.return = n, n.child = e);
    }
    if (y = e.child, !Xc(e, d)) {
      var x = y.memoizedProps;
      if (a = a.compare, a = a !== null ? a : Bi, a(x, l) && e.ref === n.ref)
        return Vn(e, n, d);
    }
    return n.flags |= 1, e = Hn(y, l), e.ref = n.ref, e.return = n, n.child = e;
  }
  function c1(e, n, a, l, d) {
    if (e !== null) {
      var y = e.memoizedProps;
      if (Bi(y, l) && e.ref === n.ref)
        if (lt = !1, n.pendingProps = l = y, Xc(e, d))
          (e.flags & 131072) !== 0 && (lt = !0);
        else
          return n.lanes = e.lanes, Vn(e, n, d);
    }
    return Bc(
      e,
      n,
      a,
      l,
      d
    );
  }
  function f1(e, n, a, l) {
    var d = l.children, y = e !== null ? e.memoizedState : null;
    if (e === null && n.stateNode === null && (n.stateNode = {
      _visibility: 1,
      _pendingMarkers: null,
      _retryCache: null,
      _transitions: null
    }), l.mode === "hidden") {
      if ((n.flags & 128) !== 0) {
        if (y = y !== null ? y.baseLanes | a : a, e !== null) {
          for (l = n.child = e.child, d = 0; l !== null; )
            d = d | l.lanes | l.childLanes, l = l.sibling;
          l = d & ~y;
        } else l = 0, n.child = null;
        return d1(
          e,
          n,
          y,
          a,
          l
        );
      }
      if ((a & 536870912) !== 0)
        n.memoizedState = { baseLanes: 0, cachePool: null }, e !== null && El(
          n,
          y !== null ? y.cachePool : null
        ), y !== null ? g0(n, y) : hc(), v0(n);
      else
        return l = n.lanes = 536870912, d1(
          e,
          n,
          y !== null ? y.baseLanes | a : a,
          a,
          l
        );
    } else
      y !== null ? (El(n, y.cachePool), g0(n, y), pr(), n.memoizedState = null) : (e !== null && El(n, null), hc(), pr());
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
  function d1(e, n, a, l, d) {
    var y = lc();
    return y = y === null ? null : { parent: it._currentValue, pool: y }, n.memoizedState = {
      baseLanes: a,
      cachePool: y
    }, e !== null && El(n, null), hc(), v0(n), e !== null && La(e, n, l, !0), n.childLanes = d, null;
  }
  function Ul(e, n) {
    return n = Vl(
      { mode: n.mode, children: n.children },
      e.mode
    ), n.ref = e.ref, e.child = n, n.return = e, n;
  }
  function h1(e, n, a) {
    return Jr(n, e.child, null, a), e = Ul(n, n.pendingProps), e.flags |= 2, kt(n), n.memoizedState = null, e;
  }
  function zC(e, n, a) {
    var l = n.pendingProps, d = (n.flags & 128) !== 0;
    if (n.flags &= -129, e === null) {
      if (Oe) {
        if (l.mode === "hidden")
          return e = Ul(n, l), n.lanes = 536870912, Pi(null, e);
        if (vc(n), (e = Xe) ? (e = Ab(
          e,
          Wt
        ), e = e !== null && e.data === "&" ? e : null, e !== null && (n.memoizedState = {
          dehydrated: e,
          treeContext: sr !== null ? { id: _n, overflow: xn } : null,
          retryLane: 536870912,
          hydrationErrors: null
        }, a = Fm(e), a.return = n, n.child = a, pt = n, Xe = null)) : e = null, e === null) throw fr(n);
        return n.lanes = 536870912, null;
      }
      return Ul(n, l);
    }
    var y = e.memoizedState;
    if (y !== null) {
      var x = y.dehydrated;
      if (vc(n), d)
        if (n.flags & 256)
          n.flags &= -257, n = h1(
            e,
            n,
            a
          );
        else if (n.memoizedState !== null)
          n.child = e.child, n.flags |= 128, n = null;
        else throw Error(u(558));
      else if (lt || La(e, n, a, !1), d = (a & e.childLanes) !== 0, lt || d) {
        if (l = Ye, l !== null && (x = Pu(l, a), x !== 0 && x !== y.retryLane))
          throw y.retryLane = x, kr(e, x), Bt(l, e, x), Lc;
        Fl(), n = h1(
          e,
          n,
          a
        );
      } else
        e = y.treeContext, Xe = tn(x.nextSibling), pt = n, Oe = !0, cr = null, Wt = !1, e !== null && Wm(n, e), n = Ul(n, l), n.flags |= 4096;
      return n;
    }
    return e = Hn(e.child, {
      mode: l.mode,
      children: l.children
    }), e.ref = n.ref, n.child = e, e.return = n, e;
  }
  function Gl(e, n) {
    var a = n.ref;
    if (a === null)
      e !== null && e.ref !== null && (n.flags |= 4194816);
    else {
      if (typeof a != "function" && typeof a != "object")
        throw Error(u(284));
      (e === null || e.ref !== a) && (n.flags |= 4194816);
    }
  }
  function Bc(e, n, a, l, d) {
    return Zr(n), a = pc(
      e,
      n,
      a,
      l,
      void 0,
      d
    ), l = mc(), e !== null && !lt ? (bc(e, n, d), Vn(e, n, d)) : (Oe && l && Ps(n), n.flags |= 1, bt(e, n, a, d), n.child);
  }
  function g1(e, n, a, l, d, y) {
    return Zr(n), n.updateQueue = null, a = p0(
      n,
      l,
      a,
      d
    ), y0(e), l = mc(), e !== null && !lt ? (bc(e, n, y), Vn(e, n, y)) : (Oe && l && Ps(n), n.flags |= 1, bt(e, n, a, y), n.child);
  }
  function v1(e, n, a, l, d) {
    if (Zr(n), n.stateNode === null) {
      var y = Oa, x = a.contextType;
      typeof x == "object" && x !== null && (y = mt(x)), y = new a(l, y), n.memoizedState = y.state !== null && y.state !== void 0 ? y.state : null, y.updater = Dc, n.stateNode = y, y._reactInternals = n, y = n.stateNode, y.props = l, y.state = n.memoizedState, y.refs = {}, sc(n), x = a.contextType, y.context = typeof x == "object" && x !== null ? mt(x) : Oa, y.state = n.memoizedState, x = a.getDerivedStateFromProps, typeof x == "function" && (zc(
        n,
        a,
        x,
        l
      ), y.state = n.memoizedState), typeof a.getDerivedStateFromProps == "function" || typeof y.getSnapshotBeforeUpdate == "function" || typeof y.UNSAFE_componentWillMount != "function" && typeof y.componentWillMount != "function" || (x = y.state, typeof y.componentWillMount == "function" && y.componentWillMount(), typeof y.UNSAFE_componentWillMount == "function" && y.UNSAFE_componentWillMount(), x !== y.state && Dc.enqueueReplaceState(y, y.state, null), Zi(n, l, y, d), Qi(), y.state = n.memoizedState), typeof y.componentDidMount == "function" && (n.flags |= 4194308), l = !0;
    } else if (e === null) {
      y = n.stateNode;
      var q = n.memoizedProps, U = Wr(a, q);
      y.props = U;
      var te = y.context, oe = a.contextType;
      x = Oa, typeof oe == "object" && oe !== null && (x = mt(oe));
      var fe = a.getDerivedStateFromProps;
      oe = typeof fe == "function" || typeof y.getSnapshotBeforeUpdate == "function", q = n.pendingProps !== q, oe || typeof y.UNSAFE_componentWillReceiveProps != "function" && typeof y.componentWillReceiveProps != "function" || (q || te !== x) && t1(
        n,
        y,
        l,
        x
      ), hr = !1;
      var ne = n.memoizedState;
      y.state = ne, Zi(n, l, y, d), Qi(), te = n.memoizedState, q || ne !== te || hr ? (typeof fe == "function" && (zc(
        n,
        a,
        fe,
        l
      ), te = n.memoizedState), (U = hr || e1(
        n,
        a,
        U,
        l,
        ne,
        te,
        x
      )) ? (oe || typeof y.UNSAFE_componentWillMount != "function" && typeof y.componentWillMount != "function" || (typeof y.componentWillMount == "function" && y.componentWillMount(), typeof y.UNSAFE_componentWillMount == "function" && y.UNSAFE_componentWillMount()), typeof y.componentDidMount == "function" && (n.flags |= 4194308)) : (typeof y.componentDidMount == "function" && (n.flags |= 4194308), n.memoizedProps = l, n.memoizedState = te), y.props = l, y.state = te, y.context = x, l = U) : (typeof y.componentDidMount == "function" && (n.flags |= 4194308), l = !1);
    } else {
      y = n.stateNode, cc(e, n), x = n.memoizedProps, oe = Wr(a, x), y.props = oe, fe = n.pendingProps, ne = y.context, te = a.contextType, U = Oa, typeof te == "object" && te !== null && (U = mt(te)), q = a.getDerivedStateFromProps, (te = typeof q == "function" || typeof y.getSnapshotBeforeUpdate == "function") || typeof y.UNSAFE_componentWillReceiveProps != "function" && typeof y.componentWillReceiveProps != "function" || (x !== fe || ne !== U) && t1(
        n,
        y,
        l,
        U
      ), hr = !1, ne = n.memoizedState, y.state = ne, Zi(n, l, y, d), Qi();
      var ue = n.memoizedState;
      x !== fe || ne !== ue || hr || e !== null && e.dependencies !== null && xl(e.dependencies) ? (typeof q == "function" && (zc(
        n,
        a,
        q,
        l
      ), ue = n.memoizedState), (oe = hr || e1(
        n,
        a,
        oe,
        l,
        ne,
        ue,
        U
      ) || e !== null && e.dependencies !== null && xl(e.dependencies)) ? (te || typeof y.UNSAFE_componentWillUpdate != "function" && typeof y.componentWillUpdate != "function" || (typeof y.componentWillUpdate == "function" && y.componentWillUpdate(l, ue, U), typeof y.UNSAFE_componentWillUpdate == "function" && y.UNSAFE_componentWillUpdate(
        l,
        ue,
        U
      )), typeof y.componentDidUpdate == "function" && (n.flags |= 4), typeof y.getSnapshotBeforeUpdate == "function" && (n.flags |= 1024)) : (typeof y.componentDidUpdate != "function" || x === e.memoizedProps && ne === e.memoizedState || (n.flags |= 4), typeof y.getSnapshotBeforeUpdate != "function" || x === e.memoizedProps && ne === e.memoizedState || (n.flags |= 1024), n.memoizedProps = l, n.memoizedState = ue), y.props = l, y.state = ue, y.context = U, l = oe) : (typeof y.componentDidUpdate != "function" || x === e.memoizedProps && ne === e.memoizedState || (n.flags |= 4), typeof y.getSnapshotBeforeUpdate != "function" || x === e.memoizedProps && ne === e.memoizedState || (n.flags |= 1024), l = !1);
    }
    return y = l, Gl(e, n), l = (n.flags & 128) !== 0, y || l ? (y = n.stateNode, a = l && typeof a.getDerivedStateFromError != "function" ? null : y.render(), n.flags |= 1, e !== null && l ? (n.child = Jr(
      n,
      e.child,
      null,
      d
    ), n.child = Jr(
      n,
      null,
      a,
      d
    )) : bt(e, n, a, d), n.memoizedState = y.state, e = n.child) : e = Vn(
      e,
      n,
      d
    ), e;
  }
  function y1(e, n, a, l) {
    return Ir(), n.flags |= 256, bt(e, n, a, l), n.child;
  }
  var jc = {
    dehydrated: null,
    treeContext: null,
    retryLane: 0,
    hydrationErrors: null
  };
  function Uc(e) {
    return { baseLanes: e, cachePool: i0() };
  }
  function Gc(e, n, a) {
    return e = e !== null ? e.childLanes & ~a : 0, n && (e |= It), e;
  }
  function p1(e, n, a) {
    var l = n.pendingProps, d = !1, y = (n.flags & 128) !== 0, x;
    if ((x = y) || (x = e !== null && e.memoizedState === null ? !1 : (nt.current & 2) !== 0), x && (d = !0, n.flags &= -129), x = (n.flags & 32) !== 0, n.flags &= -33, e === null) {
      if (Oe) {
        if (d ? yr(n) : pr(), (e = Xe) ? (e = Ab(
          e,
          Wt
        ), e = e !== null && e.data !== "&" ? e : null, e !== null && (n.memoizedState = {
          dehydrated: e,
          treeContext: sr !== null ? { id: _n, overflow: xn } : null,
          retryLane: 536870912,
          hydrationErrors: null
        }, a = Fm(e), a.return = n, n.child = a, pt = n, Xe = null)) : e = null, e === null) throw fr(n);
        return Ef(e) ? n.lanes = 32 : n.lanes = 536870912, null;
      }
      var q = l.children;
      return l = l.fallback, d ? (pr(), d = n.mode, q = Vl(
        { mode: "hidden", children: q },
        d
      ), l = Xr(
        l,
        d,
        a,
        null
      ), q.return = n, l.return = n, q.sibling = l, n.child = q, l = n.child, l.memoizedState = Uc(a), l.childLanes = Gc(
        e,
        x,
        a
      ), n.memoizedState = jc, Pi(null, l)) : (yr(n), Vc(n, q));
    }
    var U = e.memoizedState;
    if (U !== null && (q = U.dehydrated, q !== null)) {
      if (y)
        n.flags & 256 ? (yr(n), n.flags &= -257, n = Yc(
          e,
          n,
          a
        )) : n.memoizedState !== null ? (pr(), n.child = e.child, n.flags |= 128, n = null) : (pr(), q = l.fallback, d = n.mode, l = Vl(
          { mode: "visible", children: l.children },
          d
        ), q = Xr(
          q,
          d,
          a,
          null
        ), q.flags |= 2, l.return = n, q.return = n, l.sibling = q, n.child = l, Jr(
          n,
          e.child,
          null,
          a
        ), l = n.child, l.memoizedState = Uc(a), l.childLanes = Gc(
          e,
          x,
          a
        ), n.memoizedState = jc, n = Pi(null, l));
      else if (yr(n), Ef(q)) {
        if (x = q.nextSibling && q.nextSibling.dataset, x) var te = x.dgst;
        x = te, l = Error(u(419)), l.stack = "", l.digest = x, Gi({ value: l, source: null, stack: null }), n = Yc(
          e,
          n,
          a
        );
      } else if (lt || La(e, n, a, !1), x = (a & e.childLanes) !== 0, lt || x) {
        if (x = Ye, x !== null && (l = Pu(x, a), l !== 0 && l !== U.retryLane))
          throw U.retryLane = l, kr(e, l), Bt(x, e, l), Lc;
        Sf(q) || Fl(), n = Yc(
          e,
          n,
          a
        );
      } else
        Sf(q) ? (n.flags |= 192, n.child = e.child, n = null) : (e = U.treeContext, Xe = tn(
          q.nextSibling
        ), pt = n, Oe = !0, cr = null, Wt = !1, e !== null && Wm(n, e), n = Vc(
          n,
          l.children
        ), n.flags |= 4096);
      return n;
    }
    return d ? (pr(), q = l.fallback, d = n.mode, U = e.child, te = U.sibling, l = Hn(U, {
      mode: "hidden",
      children: l.children
    }), l.subtreeFlags = U.subtreeFlags & 65011712, te !== null ? q = Hn(
      te,
      q
    ) : (q = Xr(
      q,
      d,
      a,
      null
    ), q.flags |= 2), q.return = n, l.return = n, l.sibling = q, n.child = l, Pi(null, l), l = n.child, q = e.child.memoizedState, q === null ? q = Uc(a) : (d = q.cachePool, d !== null ? (U = it._currentValue, d = d.parent !== U ? { parent: U, pool: U } : d) : d = i0(), q = {
      baseLanes: q.baseLanes | a,
      cachePool: d
    }), l.memoizedState = q, l.childLanes = Gc(
      e,
      x,
      a
    ), n.memoizedState = jc, Pi(e.child, l)) : (yr(n), a = e.child, e = a.sibling, a = Hn(a, {
      mode: "visible",
      children: l.children
    }), a.return = n, a.sibling = null, e !== null && (x = n.deletions, x === null ? (n.deletions = [e], n.flags |= 16) : x.push(e)), n.child = a, n.memoizedState = null, a);
  }
  function Vc(e, n) {
    return n = Vl(
      { mode: "visible", children: n },
      e.mode
    ), n.return = e, e.child = n;
  }
  function Vl(e, n) {
    return e = Vt(22, e, null, n), e.lanes = 0, e;
  }
  function Yc(e, n, a) {
    return Jr(n, e.child, null, a), e = Vc(
      n,
      n.pendingProps.children
    ), e.flags |= 2, n.memoizedState = null, e;
  }
  function m1(e, n, a) {
    e.lanes |= n;
    var l = e.alternate;
    l !== null && (l.lanes |= n), rc(e.return, n, a);
  }
  function kc(e, n, a, l, d, y) {
    var x = e.memoizedState;
    x === null ? e.memoizedState = {
      isBackwards: n,
      rendering: null,
      renderingStartTime: 0,
      last: l,
      tail: a,
      tailMode: d,
      treeForkCount: y
    } : (x.isBackwards = n, x.rendering = null, x.renderingStartTime = 0, x.last = l, x.tail = a, x.tailMode = d, x.treeForkCount = y);
  }
  function b1(e, n, a) {
    var l = n.pendingProps, d = l.revealOrder, y = l.tail;
    l = l.children;
    var x = nt.current, q = (x & 2) !== 0;
    if (q ? (x = x & 1 | 2, n.flags |= 128) : x &= 1, ie(nt, x), bt(e, n, l, a), l = Oe ? Ui : 0, !q && e !== null && (e.flags & 128) !== 0)
      e: for (e = n.child; e !== null; ) {
        if (e.tag === 13)
          e.memoizedState !== null && m1(e, a, n);
        else if (e.tag === 19)
          m1(e, a, n);
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
          e = a.alternate, e !== null && Cl(e) === null && (d = a), a = a.sibling;
        a = d, a === null ? (d = n.child, n.child = null) : (d = a.sibling, a.sibling = null), kc(
          n,
          !1,
          d,
          a,
          y,
          l
        );
        break;
      case "backwards":
      case "unstable_legacy-backwards":
        for (a = null, d = n.child, n.child = null; d !== null; ) {
          if (e = d.alternate, e !== null && Cl(e) === null) {
            n.child = d;
            break;
          }
          e = d.sibling, d.sibling = a, a = d, d = e;
        }
        kc(
          n,
          !0,
          a,
          null,
          y,
          l
        );
        break;
      case "together":
        kc(
          n,
          !1,
          null,
          null,
          void 0,
          l
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
        if (La(
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
      for (e = n.child, a = Hn(e, e.pendingProps), n.child = a, a.return = n; e.sibling !== null; )
        e = e.sibling, a = a.sibling = Hn(e, e.pendingProps), a.return = n;
      a.sibling = null;
    }
    return n.child;
  }
  function Xc(e, n) {
    return (e.lanes & n) !== 0 ? !0 : (e = e.dependencies, !!(e !== null && xl(e)));
  }
  function DC(e, n, a) {
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
          return n.flags |= 128, vc(n), null;
        break;
      case 13:
        var l = n.memoizedState;
        if (l !== null)
          return l.dehydrated !== null ? (yr(n), n.flags |= 128, null) : (a & n.child.childLanes) !== 0 ? p1(e, n, a) : (yr(n), e = Vn(
            e,
            n,
            a
          ), e !== null ? e.sibling : null);
        yr(n);
        break;
      case 19:
        var d = (e.flags & 128) !== 0;
        if (l = (a & n.childLanes) !== 0, l || (La(
          e,
          n,
          a,
          !1
        ), l = (a & n.childLanes) !== 0), d) {
          if (l)
            return b1(
              e,
              n,
              a
            );
          n.flags |= 128;
        }
        if (d = n.memoizedState, d !== null && (d.rendering = null, d.tail = null, d.lastEffect = null), ie(nt, nt.current), l) break;
        return null;
      case 22:
        return n.lanes = 0, f1(
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
  function _1(e, n, a) {
    if (e !== null)
      if (e.memoizedProps !== n.pendingProps)
        lt = !0;
      else {
        if (!Xc(e, a) && (n.flags & 128) === 0)
          return lt = !1, DC(
            e,
            n,
            a
          );
        lt = (e.flags & 131072) !== 0;
      }
    else
      lt = !1, Oe && (n.flags & 1048576) !== 0 && Pm(n, Ui, n.index);
    switch (n.lanes = 0, n.tag) {
      case 16:
        e: {
          var l = n.pendingProps;
          if (e = $r(n.elementType), n.type = e, typeof e == "function")
            $s(e) ? (l = Wr(e, l), n.tag = 1, n = v1(
              null,
              n,
              e,
              l,
              a
            )) : (n.tag = 0, n = Bc(
              null,
              n,
              e,
              l,
              a
            ));
          else {
            if (e != null) {
              var d = e.$$typeof;
              if (d === T) {
                n.tag = 11, n = o1(
                  null,
                  n,
                  e,
                  l,
                  a
                );
                break e;
              } else if (d === R) {
                n.tag = 14, n = s1(
                  null,
                  n,
                  e,
                  l,
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
        return Bc(
          e,
          n,
          n.type,
          n.pendingProps,
          a
        );
      case 1:
        return l = n.type, d = Wr(
          l,
          n.pendingProps
        ), v1(
          e,
          n,
          l,
          d,
          a
        );
      case 3:
        e: {
          if (W(
            n,
            n.stateNode.containerInfo
          ), e === null) throw Error(u(387));
          l = n.pendingProps;
          var y = n.memoizedState;
          d = y.element, cc(e, n), Zi(n, l, null, a);
          var x = n.memoizedState;
          if (l = x.cache, dr(n, it, l), l !== y.cache && ac(
            n,
            [it],
            a,
            !0
          ), Qi(), l = x.element, y.isDehydrated)
            if (y = {
              element: l,
              isDehydrated: !1,
              cache: x.cache
            }, n.updateQueue.baseState = y, n.memoizedState = y, n.flags & 256) {
              n = y1(
                e,
                n,
                l,
                a
              );
              break e;
            } else if (l !== d) {
              d = Ft(
                Error(u(424)),
                n
              ), Gi(d), n = y1(
                e,
                n,
                l,
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
              for (Xe = tn(e.firstChild), pt = n, Oe = !0, cr = null, Wt = !0, a = f0(
                n,
                null,
                l,
                a
              ), n.child = a; a; )
                a.flags = a.flags & -3 | 4096, a = a.sibling;
            }
          else {
            if (Ir(), l === d) {
              n = Vn(
                e,
                n,
                a
              );
              break e;
            }
            bt(e, n, l, a);
          }
          n = n.child;
        }
        return n;
      case 26:
        return Gl(e, n), e === null ? (a = Nb(
          n.type,
          null,
          n.pendingProps,
          null
        )) ? n.memoizedState = a : Oe || (a = n.type, e = n.pendingProps, l = ro(
          P.current
        ).createElement(a), l[dt] = n, l[wt] = e, _t(l, a, e), at(l), n.stateNode = l) : n.memoizedState = Nb(
          n.type,
          e.memoizedProps,
          n.pendingProps,
          e.memoizedState
        ), null;
      case 27:
        return de(n), e === null && Oe && (l = n.stateNode = qb(
          n.type,
          n.pendingProps,
          P.current
        ), pt = n, Wt = !0, d = Xe, Ar(n.type) ? (wf = d, Xe = tn(l.firstChild)) : Xe = d), bt(
          e,
          n,
          n.pendingProps.children,
          a
        ), Gl(e, n), e === null && (n.flags |= 4194304), n.child;
      case 5:
        return e === null && Oe && ((d = l = Xe) && (l = cR(
          l,
          n.type,
          n.pendingProps,
          Wt
        ), l !== null ? (n.stateNode = l, pt = n, Xe = tn(l.firstChild), Wt = !1, d = !0) : d = !1), d || fr(n)), de(n), d = n.type, y = n.pendingProps, x = e !== null ? e.memoizedProps : null, l = y.children, bf(d, y) ? l = null : x !== null && bf(d, x) && (n.flags |= 32), n.memoizedState !== null && (d = pc(
          e,
          n,
          AC,
          null,
          null,
          a
        ), hu._currentValue = d), Gl(e, n), bt(e, n, l, a), n.child;
      case 6:
        return e === null && Oe && ((e = a = Xe) && (a = fR(
          a,
          n.pendingProps,
          Wt
        ), a !== null ? (n.stateNode = a, pt = n, Xe = null, e = !0) : e = !1), e || fr(n)), null;
      case 13:
        return p1(e, n, a);
      case 4:
        return W(
          n,
          n.stateNode.containerInfo
        ), l = n.pendingProps, e === null ? n.child = Jr(
          n,
          null,
          l,
          a
        ) : bt(e, n, l, a), n.child;
      case 11:
        return o1(
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
        return l = n.pendingProps, dr(n, n.type, l.value), bt(e, n, l.children, a), n.child;
      case 9:
        return d = n.type._context, l = n.pendingProps.children, Zr(n), d = mt(d), l = l(d), n.flags |= 1, bt(e, n, l, a), n.child;
      case 14:
        return s1(
          e,
          n,
          n.type,
          n.pendingProps,
          a
        );
      case 15:
        return c1(
          e,
          n,
          n.type,
          n.pendingProps,
          a
        );
      case 19:
        return b1(e, n, a);
      case 31:
        return zC(e, n, a);
      case 22:
        return f1(
          e,
          n,
          a,
          n.pendingProps
        );
      case 24:
        return Zr(n), l = mt(it), e === null ? (d = lc(), d === null && (d = Ye, y = ic(), d.pooledCache = y, y.refCount++, y !== null && (d.pooledCacheLanes |= a), d = y), n.memoizedState = { parent: l, cache: d }, sc(n), dr(n, it, d)) : ((e.lanes & a) !== 0 && (cc(e, n), Zi(n, null, null, a), Qi()), d = e.memoizedState, y = n.memoizedState, d.parent !== l ? (d = { parent: l, cache: l }, n.memoizedState = d, n.lanes === 0 && (n.memoizedState = n.updateQueue.baseState = d), dr(n, it, l)) : (l = y.cache, dr(n, it, l), l !== d.cache && ac(
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
  function Ic(e, n, a, l, d) {
    if ((n = (e.mode & 32) !== 0) && (n = !1), n) {
      if (e.flags |= 16777216, (d & 335544128) === d)
        if (e.stateNode.complete) e.flags |= 8192;
        else if (Z1()) e.flags |= 8192;
        else
          throw Fr = Al, oc;
    } else e.flags &= -16777217;
  }
  function x1(e, n) {
    if (n.type !== "stylesheet" || (n.state.loading & 4) !== 0)
      e.flags &= -16777217;
    else if (e.flags |= 16777216, !Lb(n))
      if (Z1()) e.flags |= 8192;
      else
        throw Fr = Al, oc;
  }
  function Yl(e, n) {
    n !== null && (e.flags |= 4), e.flags & 16384 && (n = e.tag !== 22 ? $u() : 536870912, e.lanes |= n, Ka |= n);
  }
  function Wi(e, n) {
    if (!Oe)
      switch (e.tailMode) {
        case "hidden":
          n = e.tail;
          for (var a = null; n !== null; )
            n.alternate !== null && (a = n), n = n.sibling;
          a === null ? e.tail = null : a.sibling = null;
          break;
        case "collapsed":
          a = e.tail;
          for (var l = null; a !== null; )
            a.alternate !== null && (l = a), a = a.sibling;
          l === null ? n || e.tail === null ? e.tail = null : e.tail.sibling = null : l.sibling = null;
      }
  }
  function Ie(e) {
    var n = e.alternate !== null && e.alternate.child === e.child, a = 0, l = 0;
    if (n)
      for (var d = e.child; d !== null; )
        a |= d.lanes | d.childLanes, l |= d.subtreeFlags & 65011712, l |= d.flags & 65011712, d.return = e, d = d.sibling;
    else
      for (d = e.child; d !== null; )
        a |= d.lanes | d.childLanes, l |= d.subtreeFlags, l |= d.flags, d.return = e, d = d.sibling;
    return e.subtreeFlags |= l, e.childLanes = a, n;
  }
  function HC(e, n, a) {
    var l = n.pendingProps;
    switch (Ws(n), n.tag) {
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
        return a = n.stateNode, l = null, e !== null && (l = e.memoizedState.cache), n.memoizedState.cache !== l && (n.flags |= 2048), jn(it), se(), a.pendingContext && (a.context = a.pendingContext, a.pendingContext = null), (e === null || e.child === null) && (Ha(n) ? Yn(n) : e === null || e.memoizedState.isDehydrated && (n.flags & 256) === 0 || (n.flags |= 1024, tc())), Ie(n), null;
      case 26:
        var d = n.type, y = n.memoizedState;
        return e === null ? (Yn(n), y !== null ? (Ie(n), x1(n, y)) : (Ie(n), Ic(
          n,
          d,
          null,
          l,
          a
        ))) : y ? y !== e.memoizedState ? (Yn(n), Ie(n), x1(n, y)) : (Ie(n), n.flags &= -16777217) : (e = e.memoizedProps, e !== l && Yn(n), Ie(n), Ic(
          n,
          d,
          e,
          l,
          a
        )), null;
      case 27:
        if (ve(n), a = P.current, d = n.type, e !== null && n.stateNode != null)
          e.memoizedProps !== l && Yn(n);
        else {
          if (!l) {
            if (n.stateNode === null)
              throw Error(u(166));
            return Ie(n), null;
          }
          e = L.current, Ha(n) ? e0(n) : (e = qb(d, l, a), n.stateNode = e, Yn(n));
        }
        return Ie(n), null;
      case 5:
        if (ve(n), d = n.type, e !== null && n.stateNode != null)
          e.memoizedProps !== l && Yn(n);
        else {
          if (!l) {
            if (n.stateNode === null)
              throw Error(u(166));
            return Ie(n), null;
          }
          if (y = L.current, Ha(n))
            e0(n);
          else {
            var x = ro(
              P.current
            );
            switch (y) {
              case 1:
                y = x.createElementNS(
                  "http://www.w3.org/2000/svg",
                  d
                );
                break;
              case 2:
                y = x.createElementNS(
                  "http://www.w3.org/1998/Math/MathML",
                  d
                );
                break;
              default:
                switch (d) {
                  case "svg":
                    y = x.createElementNS(
                      "http://www.w3.org/2000/svg",
                      d
                    );
                    break;
                  case "math":
                    y = x.createElementNS(
                      "http://www.w3.org/1998/Math/MathML",
                      d
                    );
                    break;
                  case "script":
                    y = x.createElement("div"), y.innerHTML = "<script><\/script>", y = y.removeChild(
                      y.firstChild
                    );
                    break;
                  case "select":
                    y = typeof l.is == "string" ? x.createElement("select", {
                      is: l.is
                    }) : x.createElement("select"), l.multiple ? y.multiple = !0 : l.size && (y.size = l.size);
                    break;
                  default:
                    y = typeof l.is == "string" ? x.createElement(d, { is: l.is }) : x.createElement(d);
                }
            }
            y[dt] = n, y[wt] = l;
            e: for (x = n.child; x !== null; ) {
              if (x.tag === 5 || x.tag === 6)
                y.appendChild(x.stateNode);
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
            n.stateNode = y;
            e: switch (_t(y, d, l), d) {
              case "button":
              case "input":
              case "select":
              case "textarea":
                l = !!l.autoFocus;
                break e;
              case "img":
                l = !0;
                break e;
              default:
                l = !1;
            }
            l && Yn(n);
          }
        }
        return Ie(n), Ic(
          n,
          n.type,
          e === null ? null : e.memoizedProps,
          n.pendingProps,
          a
        ), null;
      case 6:
        if (e && n.stateNode != null)
          e.memoizedProps !== l && Yn(n);
        else {
          if (typeof l != "string" && n.stateNode === null)
            throw Error(u(166));
          if (e = P.current, Ha(n)) {
            if (e = n.stateNode, a = n.memoizedProps, l = null, d = pt, d !== null)
              switch (d.tag) {
                case 27:
                case 5:
                  l = d.memoizedProps;
              }
            e[dt] = n, e = !!(e.nodeValue === a || l !== null && l.suppressHydrationWarning === !0 || pb(e.nodeValue, a)), e || fr(n, !0);
          } else
            e = ro(e).createTextNode(
              l
            ), e[dt] = n, n.stateNode = e;
        }
        return Ie(n), null;
      case 31:
        if (a = n.memoizedState, e === null || e.memoizedState !== null) {
          if (l = Ha(n), a !== null) {
            if (e === null) {
              if (!l) throw Error(u(318));
              if (e = n.memoizedState, e = e !== null ? e.dehydrated : null, !e) throw Error(u(557));
              e[dt] = n;
            } else
              Ir(), (n.flags & 128) === 0 && (n.memoizedState = null), n.flags |= 4;
            Ie(n), e = !1;
          } else
            a = tc(), e !== null && e.memoizedState !== null && (e.memoizedState.hydrationErrors = a), e = !0;
          if (!e)
            return n.flags & 256 ? (kt(n), n) : (kt(n), null);
          if ((n.flags & 128) !== 0)
            throw Error(u(558));
        }
        return Ie(n), null;
      case 13:
        if (l = n.memoizedState, e === null || e.memoizedState !== null && e.memoizedState.dehydrated !== null) {
          if (d = Ha(n), l !== null && l.dehydrated !== null) {
            if (e === null) {
              if (!d) throw Error(u(318));
              if (d = n.memoizedState, d = d !== null ? d.dehydrated : null, !d) throw Error(u(317));
              d[dt] = n;
            } else
              Ir(), (n.flags & 128) === 0 && (n.memoizedState = null), n.flags |= 4;
            Ie(n), d = !1;
          } else
            d = tc(), e !== null && e.memoizedState !== null && (e.memoizedState.hydrationErrors = d), d = !0;
          if (!d)
            return n.flags & 256 ? (kt(n), n) : (kt(n), null);
        }
        return kt(n), (n.flags & 128) !== 0 ? (n.lanes = a, n) : (a = l !== null, e = e !== null && e.memoizedState !== null, a && (l = n.child, d = null, l.alternate !== null && l.alternate.memoizedState !== null && l.alternate.memoizedState.cachePool !== null && (d = l.alternate.memoizedState.cachePool.pool), y = null, l.memoizedState !== null && l.memoizedState.cachePool !== null && (y = l.memoizedState.cachePool.pool), y !== d && (l.flags |= 2048)), a !== e && a && (n.child.flags |= 8192), Yl(n, n.updateQueue), Ie(n), null);
      case 4:
        return se(), e === null && gf(n.stateNode.containerInfo), Ie(n), null;
      case 10:
        return jn(n.type), Ie(n), null;
      case 19:
        if (V(nt), l = n.memoizedState, l === null) return Ie(n), null;
        if (d = (n.flags & 128) !== 0, y = l.rendering, y === null)
          if (d) Wi(l, !1);
          else {
            if (et !== 0 || e !== null && (e.flags & 128) !== 0)
              for (e = n.child; e !== null; ) {
                if (y = Cl(e), y !== null) {
                  for (n.flags |= 128, Wi(l, !1), e = y.updateQueue, n.updateQueue = e, Yl(n, e), n.subtreeFlags = 0, e = a, a = n.child; a !== null; )
                    $m(a, e), a = a.sibling;
                  return ie(
                    nt,
                    nt.current & 1 | 2
                  ), Oe && Ln(n, l.treeForkCount), n.child;
                }
                e = e.sibling;
              }
            l.tail !== null && ke() > Zl && (n.flags |= 128, d = !0, Wi(l, !1), n.lanes = 4194304);
          }
        else {
          if (!d)
            if (e = Cl(y), e !== null) {
              if (n.flags |= 128, d = !0, e = e.updateQueue, n.updateQueue = e, Yl(n, e), Wi(l, !0), l.tail === null && l.tailMode === "hidden" && !y.alternate && !Oe)
                return Ie(n), null;
            } else
              2 * ke() - l.renderingStartTime > Zl && a !== 536870912 && (n.flags |= 128, d = !0, Wi(l, !1), n.lanes = 4194304);
          l.isBackwards ? (y.sibling = n.child, n.child = y) : (e = l.last, e !== null ? e.sibling = y : n.child = y, l.last = y);
        }
        return l.tail !== null ? (e = l.tail, l.rendering = e, l.tail = e.sibling, l.renderingStartTime = ke(), e.sibling = null, a = nt.current, ie(
          nt,
          d ? a & 1 | 2 : a & 1
        ), Oe && Ln(n, l.treeForkCount), e) : (Ie(n), null);
      case 22:
      case 23:
        return kt(n), gc(), l = n.memoizedState !== null, e !== null ? e.memoizedState !== null !== l && (n.flags |= 8192) : l && (n.flags |= 8192), l ? (a & 536870912) !== 0 && (n.flags & 128) === 0 && (Ie(n), n.subtreeFlags & 6 && (n.flags |= 8192)) : Ie(n), a = n.updateQueue, a !== null && Yl(n, a.retryQueue), a = null, e !== null && e.memoizedState !== null && e.memoizedState.cachePool !== null && (a = e.memoizedState.cachePool.pool), l = null, n.memoizedState !== null && n.memoizedState.cachePool !== null && (l = n.memoizedState.cachePool.pool), l !== a && (n.flags |= 2048), e !== null && V(Kr), null;
      case 24:
        return a = null, e !== null && (a = e.memoizedState.cache), n.memoizedState.cache !== a && (n.flags |= 2048), jn(it), Ie(n), null;
      case 25:
        return null;
      case 30:
        return null;
    }
    throw Error(u(156, n.tag));
  }
  function LC(e, n) {
    switch (Ws(n), n.tag) {
      case 1:
        return e = n.flags, e & 65536 ? (n.flags = e & -65537 | 128, n) : null;
      case 3:
        return jn(it), se(), e = n.flags, (e & 65536) !== 0 && (e & 128) === 0 ? (n.flags = e & -65537 | 128, n) : null;
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
        return V(nt), null;
      case 4:
        return se(), null;
      case 10:
        return jn(n.type), null;
      case 22:
      case 23:
        return kt(n), gc(), e !== null && V(Kr), e = n.flags, e & 65536 ? (n.flags = e & -65537 | 128, n) : null;
      case 24:
        return jn(it), null;
      case 25:
        return null;
      default:
        return null;
    }
  }
  function S1(e, n) {
    switch (Ws(n), n.tag) {
      case 3:
        jn(it), se();
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
        V(nt);
        break;
      case 10:
        jn(n.type);
        break;
      case 22:
      case 23:
        kt(n), gc(), e !== null && V(Kr);
        break;
      case 24:
        jn(it);
    }
  }
  function eu(e, n) {
    try {
      var a = n.updateQueue, l = a !== null ? a.lastEffect : null;
      if (l !== null) {
        var d = l.next;
        a = d;
        do {
          if ((a.tag & e) === e) {
            l = void 0;
            var y = a.create, x = a.inst;
            l = y(), x.destroy = l;
          }
          a = a.next;
        } while (a !== d);
      }
    } catch (q) {
      je(n, n.return, q);
    }
  }
  function mr(e, n, a) {
    try {
      var l = n.updateQueue, d = l !== null ? l.lastEffect : null;
      if (d !== null) {
        var y = d.next;
        l = y;
        do {
          if ((l.tag & e) === e) {
            var x = l.inst, q = x.destroy;
            if (q !== void 0) {
              x.destroy = void 0, d = n;
              var U = a, te = q;
              try {
                te();
              } catch (oe) {
                je(
                  d,
                  U,
                  oe
                );
              }
            }
          }
          l = l.next;
        } while (l !== y);
      }
    } catch (oe) {
      je(n, n.return, oe);
    }
  }
  function E1(e) {
    var n = e.updateQueue;
    if (n !== null) {
      var a = e.stateNode;
      try {
        h0(n, a);
      } catch (l) {
        je(e, e.return, l);
      }
    }
  }
  function w1(e, n, a) {
    a.props = Wr(
      e.type,
      e.memoizedProps
    ), a.state = e.memoizedState;
    try {
      a.componentWillUnmount();
    } catch (l) {
      je(e, n, l);
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
            var l = e.stateNode;
            break;
          case 30:
            l = e.stateNode;
            break;
          default:
            l = e.stateNode;
        }
        typeof a == "function" ? e.refCleanup = a(l) : a.current = l;
      }
    } catch (d) {
      je(e, n, d);
    }
  }
  function Sn(e, n) {
    var a = e.ref, l = e.refCleanup;
    if (a !== null)
      if (typeof l == "function")
        try {
          l();
        } catch (d) {
          je(e, n, d);
        } finally {
          e.refCleanup = null, e = e.alternate, e != null && (e.refCleanup = null);
        }
      else if (typeof a == "function")
        try {
          a(null);
        } catch (d) {
          je(e, n, d);
        }
      else a.current = null;
  }
  function A1(e) {
    var n = e.type, a = e.memoizedProps, l = e.stateNode;
    try {
      e: switch (n) {
        case "button":
        case "input":
        case "select":
        case "textarea":
          a.autoFocus && l.focus();
          break e;
        case "img":
          a.src ? l.src = a.src : a.srcSet && (l.srcset = a.srcSet);
      }
    } catch (d) {
      je(e, e.return, d);
    }
  }
  function Qc(e, n, a) {
    try {
      var l = e.stateNode;
      aR(l, e.type, a, n), l[wt] = n;
    } catch (d) {
      je(e, e.return, d);
    }
  }
  function T1(e) {
    return e.tag === 5 || e.tag === 3 || e.tag === 26 || e.tag === 27 && Ar(e.type) || e.tag === 4;
  }
  function Zc(e) {
    e: for (; ; ) {
      for (; e.sibling === null; ) {
        if (e.return === null || T1(e.return)) return null;
        e = e.return;
      }
      for (e.sibling.return = e.return, e = e.sibling; e.tag !== 5 && e.tag !== 6 && e.tag !== 18; ) {
        if (e.tag === 27 && Ar(e.type) || e.flags & 2 || e.child === null || e.tag === 4) continue e;
        e.child.return = e, e = e.child;
      }
      if (!(e.flags & 2)) return e.stateNode;
    }
  }
  function Kc(e, n, a) {
    var l = e.tag;
    if (l === 5 || l === 6)
      e = e.stateNode, n ? (a.nodeType === 9 ? a.body : a.nodeName === "HTML" ? a.ownerDocument.body : a).insertBefore(e, n) : (n = a.nodeType === 9 ? a.body : a.nodeName === "HTML" ? a.ownerDocument.body : a, n.appendChild(e), a = a._reactRootContainer, a != null || n.onclick !== null || (n.onclick = zn));
    else if (l !== 4 && (l === 27 && Ar(e.type) && (a = e.stateNode, n = null), e = e.child, e !== null))
      for (Kc(e, n, a), e = e.sibling; e !== null; )
        Kc(e, n, a), e = e.sibling;
  }
  function kl(e, n, a) {
    var l = e.tag;
    if (l === 5 || l === 6)
      e = e.stateNode, n ? a.insertBefore(e, n) : a.appendChild(e);
    else if (l !== 4 && (l === 27 && Ar(e.type) && (a = e.stateNode), e = e.child, e !== null))
      for (kl(e, n, a), e = e.sibling; e !== null; )
        kl(e, n, a), e = e.sibling;
  }
  function M1(e) {
    var n = e.stateNode, a = e.memoizedProps;
    try {
      for (var l = e.type, d = n.attributes; d.length; )
        n.removeAttributeNode(d[0]);
      _t(n, l, a), n[dt] = e, n[wt] = a;
    } catch (y) {
      je(e, e.return, y);
    }
  }
  var kn = !1, ot = !1, $c = !1, q1 = typeof WeakSet == "function" ? WeakSet : Set, yt = null;
  function BC(e, n) {
    if (e = e.containerInfo, pf = co, e = Gm(e), Ys(e)) {
      if ("selectionStart" in e)
        var a = {
          start: e.selectionStart,
          end: e.selectionEnd
        };
      else
        e: {
          a = (a = e.ownerDocument) && a.defaultView || window;
          var l = a.getSelection && a.getSelection();
          if (l && l.rangeCount !== 0) {
            a = l.anchorNode;
            var d = l.anchorOffset, y = l.focusNode;
            l = l.focusOffset;
            try {
              a.nodeType, y.nodeType;
            } catch {
              a = null;
              break e;
            }
            var x = 0, q = -1, U = -1, te = 0, oe = 0, fe = e, ne = null;
            t: for (; ; ) {
              for (var ue; fe !== a || d !== 0 && fe.nodeType !== 3 || (q = x + d), fe !== y || l !== 0 && fe.nodeType !== 3 || (U = x + l), fe.nodeType === 3 && (x += fe.nodeValue.length), (ue = fe.firstChild) !== null; )
                ne = fe, fe = ue;
              for (; ; ) {
                if (fe === e) break t;
                if (ne === a && ++te === d && (q = x), ne === y && ++oe === l && (U = x), (ue = fe.nextSibling) !== null) break;
                fe = ne, ne = fe.parentNode;
              }
              fe = ue;
            }
            a = q === -1 || U === -1 ? null : { start: q, end: U };
          } else a = null;
        }
      a = a || { start: 0, end: 0 };
    } else a = null;
    for (mf = { focusedElem: e, selectionRange: a }, co = !1, yt = n; yt !== null; )
      if (n = yt, e = n.child, (n.subtreeFlags & 1028) !== 0 && e !== null)
        e.return = n, yt = e;
      else
        for (; yt !== null; ) {
          switch (n = yt, y = n.alternate, e = n.flags, n.tag) {
            case 0:
              if ((e & 4) !== 0 && (e = n.updateQueue, e = e !== null ? e.events : null, e !== null))
                for (a = 0; a < e.length; a++)
                  d = e[a], d.ref.impl = d.nextImpl;
              break;
            case 11:
            case 15:
              break;
            case 1:
              if ((e & 1024) !== 0 && y !== null) {
                e = void 0, a = n, d = y.memoizedProps, y = y.memoizedState, l = a.stateNode;
                try {
                  var ye = Wr(
                    a.type,
                    d
                  );
                  e = l.getSnapshotBeforeUpdate(
                    ye,
                    y
                  ), l.__reactInternalSnapshotBeforeUpdate = e;
                } catch (Ee) {
                  je(
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
                  xf(e);
                else if (a === 1)
                  switch (e.nodeName) {
                    case "HEAD":
                    case "HTML":
                    case "BODY":
                      xf(e);
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
            e.return = n.return, yt = e;
            break;
          }
          yt = n.return;
        }
  }
  function C1(e, n, a) {
    var l = a.flags;
    switch (a.tag) {
      case 0:
      case 11:
      case 15:
        In(e, a), l & 4 && eu(5, a);
        break;
      case 1:
        if (In(e, a), l & 4)
          if (e = a.stateNode, n === null)
            try {
              e.componentDidMount();
            } catch (x) {
              je(a, a.return, x);
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
              je(
                a,
                a.return,
                x
              );
            }
          }
        l & 64 && E1(a), l & 512 && tu(a, a.return);
        break;
      case 3:
        if (In(e, a), l & 64 && (e = a.updateQueue, e !== null)) {
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
            h0(e, n);
          } catch (x) {
            je(a, a.return, x);
          }
        }
        break;
      case 27:
        n === null && l & 4 && M1(a);
      case 26:
      case 5:
        In(e, a), n === null && l & 4 && A1(a), l & 512 && tu(a, a.return);
        break;
      case 12:
        In(e, a);
        break;
      case 31:
        In(e, a), l & 4 && O1(e, a);
        break;
      case 13:
        In(e, a), l & 4 && z1(e, a), l & 64 && (e = a.memoizedState, e !== null && (e = e.dehydrated, e !== null && (a = QC.bind(
          null,
          a
        ), dR(e, a))));
        break;
      case 22:
        if (l = a.memoizedState !== null || kn, !l) {
          n = n !== null && n.memoizedState !== null || ot, d = kn;
          var y = ot;
          kn = l, (ot = n) && !y ? Qn(
            e,
            a,
            (a.subtreeFlags & 8772) !== 0
          ) : In(e, a), kn = d, ot = y;
        }
        break;
      case 30:
        break;
      default:
        In(e, a);
    }
  }
  function R1(e) {
    var n = e.alternate;
    n !== null && (e.alternate = null, R1(n)), e.child = null, e.deletions = null, e.sibling = null, e.tag === 5 && (n = e.stateNode, n !== null && wi(n)), e.stateNode = null, e.return = null, e.dependencies = null, e.memoizedProps = null, e.memoizedState = null, e.pendingProps = null, e.stateNode = null, e.updateQueue = null;
  }
  var Ke = null, zt = !1;
  function Xn(e, n, a) {
    for (a = a.child; a !== null; )
      N1(e, n, a), a = a.sibling;
  }
  function N1(e, n, a) {
    if (Fe && typeof Fe.onCommitFiberUnmount == "function")
      try {
        Fe.onCommitFiberUnmount(Et, a);
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
        var l = Ke, d = zt;
        Ar(a.type) && (Ke = a.stateNode, zt = !1), Xn(
          e,
          n,
          a
        ), cu(a.stateNode), Ke = l, zt = d;
        break;
      case 5:
        ot || Sn(a, n);
      case 6:
        if (l = Ke, d = zt, Ke = null, Xn(
          e,
          n,
          a
        ), Ke = l, zt = d, Ke !== null)
          if (zt)
            try {
              (Ke.nodeType === 9 ? Ke.body : Ke.nodeName === "HTML" ? Ke.ownerDocument.body : Ke).removeChild(a.stateNode);
            } catch (y) {
              je(
                a,
                n,
                y
              );
            }
          else
            try {
              Ke.removeChild(a.stateNode);
            } catch (y) {
              je(
                a,
                n,
                y
              );
            }
        break;
      case 18:
        Ke !== null && (zt ? (e = Ke, Eb(
          e.nodeType === 9 ? e.body : e.nodeName === "HTML" ? e.ownerDocument.body : e,
          a.stateNode
        ), ni(e)) : Eb(Ke, a.stateNode));
        break;
      case 4:
        l = Ke, d = zt, Ke = a.stateNode.containerInfo, zt = !0, Xn(
          e,
          n,
          a
        ), Ke = l, zt = d;
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
        ot || (Sn(a, n), l = a.stateNode, typeof l.componentWillUnmount == "function" && w1(
          a,
          n,
          l
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
        ot = (l = ot) || a.memoizedState !== null, Xn(
          e,
          n,
          a
        ), ot = l;
        break;
      default:
        Xn(
          e,
          n,
          a
        );
    }
  }
  function O1(e, n) {
    if (n.memoizedState === null && (e = n.alternate, e !== null && (e = e.memoizedState, e !== null))) {
      e = e.dehydrated;
      try {
        ni(e);
      } catch (a) {
        je(n, n.return, a);
      }
    }
  }
  function z1(e, n) {
    if (n.memoizedState === null && (e = n.alternate, e !== null && (e = e.memoizedState, e !== null && (e = e.dehydrated, e !== null))))
      try {
        ni(e);
      } catch (a) {
        je(n, n.return, a);
      }
  }
  function jC(e) {
    switch (e.tag) {
      case 31:
      case 13:
      case 19:
        var n = e.stateNode;
        return n === null && (n = e.stateNode = new q1()), n;
      case 22:
        return e = e.stateNode, n = e._retryCache, n === null && (n = e._retryCache = new q1()), n;
      default:
        throw Error(u(435, e.tag));
    }
  }
  function Xl(e, n) {
    var a = jC(e);
    n.forEach(function(l) {
      if (!a.has(l)) {
        a.add(l);
        var d = ZC.bind(null, e, l);
        l.then(d, d);
      }
    });
  }
  function Dt(e, n) {
    var a = n.deletions;
    if (a !== null)
      for (var l = 0; l < a.length; l++) {
        var d = a[l], y = e, x = n, q = x;
        e: for (; q !== null; ) {
          switch (q.tag) {
            case 27:
              if (Ar(q.type)) {
                Ke = q.stateNode, zt = !1;
                break e;
              }
              break;
            case 5:
              Ke = q.stateNode, zt = !1;
              break e;
            case 3:
            case 4:
              Ke = q.stateNode.containerInfo, zt = !0;
              break e;
          }
          q = q.return;
        }
        if (Ke === null) throw Error(u(160));
        N1(y, x, d), Ke = null, zt = !1, y = d.alternate, y !== null && (y.return = null), d.return = null;
      }
    if (n.subtreeFlags & 13886)
      for (n = n.child; n !== null; )
        D1(n, e), n = n.sibling;
  }
  var on = null;
  function D1(e, n) {
    var a = e.alternate, l = e.flags;
    switch (e.tag) {
      case 0:
      case 11:
      case 14:
      case 15:
        Dt(n, e), Ht(e), l & 4 && (mr(3, e, e.return), eu(3, e), mr(5, e, e.return));
        break;
      case 1:
        Dt(n, e), Ht(e), l & 512 && (ot || a === null || Sn(a, a.return)), l & 64 && kn && (e = e.updateQueue, e !== null && (l = e.callbacks, l !== null && (a = e.shared.hiddenCallbacks, e.shared.hiddenCallbacks = a === null ? l : a.concat(l))));
        break;
      case 26:
        var d = on;
        if (Dt(n, e), Ht(e), l & 512 && (ot || a === null || Sn(a, a.return)), l & 4) {
          var y = a !== null ? a.memoizedState : null;
          if (l = e.memoizedState, a === null)
            if (l === null)
              if (e.stateNode === null) {
                e: {
                  l = e.type, a = e.memoizedProps, d = d.ownerDocument || d;
                  t: switch (l) {
                    case "title":
                      y = d.getElementsByTagName("title")[0], (!y || y[jr] || y[dt] || y.namespaceURI === "http://www.w3.org/2000/svg" || y.hasAttribute("itemprop")) && (y = d.createElement(l), d.head.insertBefore(
                        y,
                        d.querySelector("head > title")
                      )), _t(y, l, a), y[dt] = e, at(y), l = y;
                      break e;
                    case "link":
                      var x = Db(
                        "link",
                        "href",
                        d
                      ).get(l + (a.href || ""));
                      if (x) {
                        for (var q = 0; q < x.length; q++)
                          if (y = x[q], y.getAttribute("href") === (a.href == null || a.href === "" ? null : a.href) && y.getAttribute("rel") === (a.rel == null ? null : a.rel) && y.getAttribute("title") === (a.title == null ? null : a.title) && y.getAttribute("crossorigin") === (a.crossOrigin == null ? null : a.crossOrigin)) {
                            x.splice(q, 1);
                            break t;
                          }
                      }
                      y = d.createElement(l), _t(y, l, a), d.head.appendChild(y);
                      break;
                    case "meta":
                      if (x = Db(
                        "meta",
                        "content",
                        d
                      ).get(l + (a.content || ""))) {
                        for (q = 0; q < x.length; q++)
                          if (y = x[q], y.getAttribute("content") === (a.content == null ? null : "" + a.content) && y.getAttribute("name") === (a.name == null ? null : a.name) && y.getAttribute("property") === (a.property == null ? null : a.property) && y.getAttribute("http-equiv") === (a.httpEquiv == null ? null : a.httpEquiv) && y.getAttribute("charset") === (a.charSet == null ? null : a.charSet)) {
                            x.splice(q, 1);
                            break t;
                          }
                      }
                      y = d.createElement(l), _t(y, l, a), d.head.appendChild(y);
                      break;
                    default:
                      throw Error(u(468, l));
                  }
                  y[dt] = e, at(y), l = y;
                }
                e.stateNode = l;
              } else
                Hb(
                  d,
                  e.type,
                  e.stateNode
                );
            else
              e.stateNode = zb(
                d,
                l,
                e.memoizedProps
              );
          else
            y !== l ? (y === null ? a.stateNode !== null && (a = a.stateNode, a.parentNode.removeChild(a)) : y.count--, l === null ? Hb(
              d,
              e.type,
              e.stateNode
            ) : zb(
              d,
              l,
              e.memoizedProps
            )) : l === null && e.stateNode !== null && Qc(
              e,
              e.memoizedProps,
              a.memoizedProps
            );
        }
        break;
      case 27:
        Dt(n, e), Ht(e), l & 512 && (ot || a === null || Sn(a, a.return)), a !== null && l & 4 && Qc(
          e,
          e.memoizedProps,
          a.memoizedProps
        );
        break;
      case 5:
        if (Dt(n, e), Ht(e), l & 512 && (ot || a === null || Sn(a, a.return)), e.flags & 32) {
          d = e.stateNode;
          try {
            Aa(d, "");
          } catch (ye) {
            je(e, e.return, ye);
          }
        }
        l & 4 && e.stateNode != null && (d = e.memoizedProps, Qc(
          e,
          d,
          a !== null ? a.memoizedProps : d
        )), l & 1024 && ($c = !0);
        break;
      case 6:
        if (Dt(n, e), Ht(e), l & 4) {
          if (e.stateNode === null)
            throw Error(u(162));
          l = e.memoizedProps, a = e.stateNode;
          try {
            a.nodeValue = l;
          } catch (ye) {
            je(e, e.return, ye);
          }
        }
        break;
      case 3:
        if (uo = null, d = on, on = ao(n.containerInfo), Dt(n, e), on = d, Ht(e), l & 4 && a !== null && a.memoizedState.isDehydrated)
          try {
            ni(n.containerInfo);
          } catch (ye) {
            je(e, e.return, ye);
          }
        $c && ($c = !1, H1(e));
        break;
      case 4:
        l = on, on = ao(
          e.stateNode.containerInfo
        ), Dt(n, e), Ht(e), on = l;
        break;
      case 12:
        Dt(n, e), Ht(e);
        break;
      case 31:
        Dt(n, e), Ht(e), l & 4 && (l = e.updateQueue, l !== null && (e.updateQueue = null, Xl(e, l)));
        break;
      case 13:
        Dt(n, e), Ht(e), e.child.flags & 8192 && e.memoizedState !== null != (a !== null && a.memoizedState !== null) && (Ql = ke()), l & 4 && (l = e.updateQueue, l !== null && (e.updateQueue = null, Xl(e, l)));
        break;
      case 22:
        d = e.memoizedState !== null;
        var U = a !== null && a.memoizedState !== null, te = kn, oe = ot;
        if (kn = te || d, ot = oe || U, Dt(n, e), ot = oe, kn = te, Ht(e), l & 8192)
          e: for (n = e.stateNode, n._visibility = d ? n._visibility & -2 : n._visibility | 1, d && (a === null || U || kn || ot || ea(e)), a = null, n = e; ; ) {
            if (n.tag === 5 || n.tag === 26) {
              if (a === null) {
                U = a = n;
                try {
                  if (y = U.stateNode, d)
                    x = y.style, typeof x.setProperty == "function" ? x.setProperty("display", "none", "important") : x.display = "none";
                  else {
                    q = U.stateNode;
                    var fe = U.memoizedProps.style, ne = fe != null && fe.hasOwnProperty("display") ? fe.display : null;
                    q.style.display = ne == null || typeof ne == "boolean" ? "" : ("" + ne).trim();
                  }
                } catch (ye) {
                  je(U, U.return, ye);
                }
              }
            } else if (n.tag === 6) {
              if (a === null) {
                U = n;
                try {
                  U.stateNode.nodeValue = d ? "" : U.memoizedProps;
                } catch (ye) {
                  je(U, U.return, ye);
                }
              }
            } else if (n.tag === 18) {
              if (a === null) {
                U = n;
                try {
                  var ue = U.stateNode;
                  d ? wb(ue, !0) : wb(U.stateNode, !1);
                } catch (ye) {
                  je(U, U.return, ye);
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
        l & 4 && (l = e.updateQueue, l !== null && (a = l.retryQueue, a !== null && (l.retryQueue = null, Xl(e, a))));
        break;
      case 19:
        Dt(n, e), Ht(e), l & 4 && (l = e.updateQueue, l !== null && (e.updateQueue = null, Xl(e, l)));
        break;
      case 30:
        break;
      case 21:
        break;
      default:
        Dt(n, e), Ht(e);
    }
  }
  function Ht(e) {
    var n = e.flags;
    if (n & 2) {
      try {
        for (var a, l = e.return; l !== null; ) {
          if (T1(l)) {
            a = l;
            break;
          }
          l = l.return;
        }
        if (a == null) throw Error(u(160));
        switch (a.tag) {
          case 27:
            var d = a.stateNode, y = Zc(e);
            kl(e, y, d);
            break;
          case 5:
            var x = a.stateNode;
            a.flags & 32 && (Aa(x, ""), a.flags &= -33);
            var q = Zc(e);
            kl(e, q, x);
            break;
          case 3:
          case 4:
            var U = a.stateNode.containerInfo, te = Zc(e);
            Kc(
              e,
              te,
              U
            );
            break;
          default:
            throw Error(u(161));
        }
      } catch (oe) {
        je(e, e.return, oe);
      }
      e.flags &= -3;
    }
    n & 4096 && (e.flags &= -4097);
  }
  function H1(e) {
    if (e.subtreeFlags & 1024)
      for (e = e.child; e !== null; ) {
        var n = e;
        H1(n), n.tag === 5 && n.flags & 1024 && n.stateNode.reset(), e = e.sibling;
      }
  }
  function In(e, n) {
    if (n.subtreeFlags & 8772)
      for (n = n.child; n !== null; )
        C1(e, n.alternate, n), n = n.sibling;
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
          typeof a.componentWillUnmount == "function" && w1(
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
  function Qn(e, n, a) {
    for (a = a && (n.subtreeFlags & 8772) !== 0, n = n.child; n !== null; ) {
      var l = n.alternate, d = e, y = n, x = y.flags;
      switch (y.tag) {
        case 0:
        case 11:
        case 15:
          Qn(
            d,
            y,
            a
          ), eu(4, y);
          break;
        case 1:
          if (Qn(
            d,
            y,
            a
          ), l = y, d = l.stateNode, typeof d.componentDidMount == "function")
            try {
              d.componentDidMount();
            } catch (te) {
              je(l, l.return, te);
            }
          if (l = y, d = l.updateQueue, d !== null) {
            var q = l.stateNode;
            try {
              var U = d.shared.hiddenCallbacks;
              if (U !== null)
                for (d.shared.hiddenCallbacks = null, d = 0; d < U.length; d++)
                  d0(U[d], q);
            } catch (te) {
              je(l, l.return, te);
            }
          }
          a && x & 64 && E1(y), tu(y, y.return);
          break;
        case 27:
          M1(y);
        case 26:
        case 5:
          Qn(
            d,
            y,
            a
          ), a && l === null && x & 4 && A1(y), tu(y, y.return);
          break;
        case 12:
          Qn(
            d,
            y,
            a
          );
          break;
        case 31:
          Qn(
            d,
            y,
            a
          ), a && x & 4 && O1(d, y);
          break;
        case 13:
          Qn(
            d,
            y,
            a
          ), a && x & 4 && z1(d, y);
          break;
        case 22:
          y.memoizedState === null && Qn(
            d,
            y,
            a
          ), tu(y, y.return);
          break;
        case 30:
          break;
        default:
          Qn(
            d,
            y,
            a
          );
      }
      n = n.sibling;
    }
  }
  function Fc(e, n) {
    var a = null;
    e !== null && e.memoizedState !== null && e.memoizedState.cachePool !== null && (a = e.memoizedState.cachePool.pool), e = null, n.memoizedState !== null && n.memoizedState.cachePool !== null && (e = n.memoizedState.cachePool.pool), e !== a && (e != null && e.refCount++, a != null && Vi(a));
  }
  function Jc(e, n) {
    e = null, n.alternate !== null && (e = n.alternate.memoizedState.cache), n = n.memoizedState.cache, n !== e && (n.refCount++, e != null && Vi(e));
  }
  function sn(e, n, a, l) {
    if (n.subtreeFlags & 10256)
      for (n = n.child; n !== null; )
        L1(
          e,
          n,
          a,
          l
        ), n = n.sibling;
  }
  function L1(e, n, a, l) {
    var d = n.flags;
    switch (n.tag) {
      case 0:
      case 11:
      case 15:
        sn(
          e,
          n,
          a,
          l
        ), d & 2048 && eu(9, n);
        break;
      case 1:
        sn(
          e,
          n,
          a,
          l
        );
        break;
      case 3:
        sn(
          e,
          n,
          a,
          l
        ), d & 2048 && (e = null, n.alternate !== null && (e = n.alternate.memoizedState.cache), n = n.memoizedState.cache, n !== e && (n.refCount++, e != null && Vi(e)));
        break;
      case 12:
        if (d & 2048) {
          sn(
            e,
            n,
            a,
            l
          ), e = n.stateNode;
          try {
            var y = n.memoizedProps, x = y.id, q = y.onPostCommit;
            typeof q == "function" && q(
              x,
              n.alternate === null ? "mount" : "update",
              e.passiveEffectDuration,
              -0
            );
          } catch (U) {
            je(n, n.return, U);
          }
        } else
          sn(
            e,
            n,
            a,
            l
          );
        break;
      case 31:
        sn(
          e,
          n,
          a,
          l
        );
        break;
      case 13:
        sn(
          e,
          n,
          a,
          l
        );
        break;
      case 23:
        break;
      case 22:
        y = n.stateNode, x = n.alternate, n.memoizedState !== null ? y._visibility & 2 ? sn(
          e,
          n,
          a,
          l
        ) : nu(e, n) : y._visibility & 2 ? sn(
          e,
          n,
          a,
          l
        ) : (y._visibility |= 2, Ia(
          e,
          n,
          a,
          l,
          (n.subtreeFlags & 10256) !== 0 || !1
        )), d & 2048 && Fc(x, n);
        break;
      case 24:
        sn(
          e,
          n,
          a,
          l
        ), d & 2048 && Jc(n.alternate, n);
        break;
      default:
        sn(
          e,
          n,
          a,
          l
        );
    }
  }
  function Ia(e, n, a, l, d) {
    for (d = d && ((n.subtreeFlags & 10256) !== 0 || !1), n = n.child; n !== null; ) {
      var y = e, x = n, q = a, U = l, te = x.flags;
      switch (x.tag) {
        case 0:
        case 11:
        case 15:
          Ia(
            y,
            x,
            q,
            U,
            d
          ), eu(8, x);
          break;
        case 23:
          break;
        case 22:
          var oe = x.stateNode;
          x.memoizedState !== null ? oe._visibility & 2 ? Ia(
            y,
            x,
            q,
            U,
            d
          ) : nu(
            y,
            x
          ) : (oe._visibility |= 2, Ia(
            y,
            x,
            q,
            U,
            d
          )), d && te & 2048 && Fc(
            x.alternate,
            x
          );
          break;
        case 24:
          Ia(
            y,
            x,
            q,
            U,
            d
          ), d && te & 2048 && Jc(x.alternate, x);
          break;
        default:
          Ia(
            y,
            x,
            q,
            U,
            d
          );
      }
      n = n.sibling;
    }
  }
  function nu(e, n) {
    if (n.subtreeFlags & 10256)
      for (n = n.child; n !== null; ) {
        var a = e, l = n, d = l.flags;
        switch (l.tag) {
          case 22:
            nu(a, l), d & 2048 && Fc(
              l.alternate,
              l
            );
            break;
          case 24:
            nu(a, l), d & 2048 && Jc(l.alternate, l);
            break;
          default:
            nu(a, l);
        }
        n = n.sibling;
      }
  }
  var ru = 8192;
  function Qa(e, n, a) {
    if (e.subtreeFlags & ru)
      for (e = e.child; e !== null; )
        B1(
          e,
          n,
          a
        ), e = e.sibling;
  }
  function B1(e, n, a) {
    switch (e.tag) {
      case 26:
        Qa(
          e,
          n,
          a
        ), e.flags & ru && e.memoizedState !== null && wR(
          a,
          on,
          e.memoizedState,
          e.memoizedProps
        );
        break;
      case 5:
        Qa(
          e,
          n,
          a
        );
        break;
      case 3:
      case 4:
        var l = on;
        on = ao(e.stateNode.containerInfo), Qa(
          e,
          n,
          a
        ), on = l;
        break;
      case 22:
        e.memoizedState === null && (l = e.alternate, l !== null && l.memoizedState !== null ? (l = ru, ru = 16777216, Qa(
          e,
          n,
          a
        ), ru = l) : Qa(
          e,
          n,
          a
        ));
        break;
      default:
        Qa(
          e,
          n,
          a
        );
    }
  }
  function j1(e) {
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
          var l = n[a];
          yt = l, G1(
            l,
            e
          );
        }
      j1(e);
    }
    if (e.subtreeFlags & 10256)
      for (e = e.child; e !== null; )
        U1(e), e = e.sibling;
  }
  function U1(e) {
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
        e.memoizedState !== null && n._visibility & 2 && (e.return === null || e.return.tag !== 13) ? (n._visibility &= -3, Il(e)) : au(e);
        break;
      default:
        au(e);
    }
  }
  function Il(e) {
    var n = e.deletions;
    if ((e.flags & 16) !== 0) {
      if (n !== null)
        for (var a = 0; a < n.length; a++) {
          var l = n[a];
          yt = l, G1(
            l,
            e
          );
        }
      j1(e);
    }
    for (e = e.child; e !== null; ) {
      switch (n = e, n.tag) {
        case 0:
        case 11:
        case 15:
          mr(8, n, n.return), Il(n);
          break;
        case 22:
          a = n.stateNode, a._visibility & 2 && (a._visibility &= -3, Il(n));
          break;
        default:
          Il(n);
      }
      e = e.sibling;
    }
  }
  function G1(e, n) {
    for (; yt !== null; ) {
      var a = yt;
      switch (a.tag) {
        case 0:
        case 11:
        case 15:
          mr(8, a, n);
          break;
        case 23:
        case 22:
          if (a.memoizedState !== null && a.memoizedState.cachePool !== null) {
            var l = a.memoizedState.cachePool.pool;
            l != null && l.refCount++;
          }
          break;
        case 24:
          Vi(a.memoizedState.cache);
      }
      if (l = a.child, l !== null) l.return = a, yt = l;
      else
        e: for (a = e; yt !== null; ) {
          l = yt;
          var d = l.sibling, y = l.return;
          if (R1(l), l === a) {
            yt = null;
            break e;
          }
          if (d !== null) {
            d.return = y, yt = d;
            break e;
          }
          yt = y;
        }
    }
  }
  var UC = {
    getCacheForType: function(e) {
      var n = mt(it), a = n.data.get(e);
      return a === void 0 && (a = e(), n.data.set(e, a)), a;
    },
    cacheSignal: function() {
      return mt(it).controller.signal;
    }
  }, GC = typeof WeakMap == "function" ? WeakMap : Map, Le = 0, Ye = null, qe = null, Re = 0, Be = 0, Xt = null, br = !1, Za = !1, Pc = !1, Zn = 0, et = 0, _r = 0, ta = 0, Wc = 0, It = 0, Ka = 0, iu = null, Lt = null, ef = !1, Ql = 0, V1 = 0, Zl = 1 / 0, Kl = null, xr = null, ht = 0, Sr = null, $a = null, Kn = 0, tf = 0, nf = null, Y1 = null, uu = 0, rf = null;
  function Qt() {
    return (Le & 2) !== 0 && Re !== 0 ? Re & -Re : N.T !== null ? cf() : Wu();
  }
  function k1() {
    if (It === 0)
      if ((Re & 536870912) === 0 || Oe) {
        var e = pa;
        pa <<= 1, (pa & 3932160) === 0 && (pa = 262144), It = e;
      } else It = 536870912;
    return e = Yt.current, e !== null && (e.flags |= 32), It;
  }
  function Bt(e, n, a) {
    (e === Ye && (Be === 2 || Be === 9) || e.cancelPendingCommit !== null) && (Fa(e, 0), Er(
      e,
      Re,
      It,
      !1
    )), Br(e, a), ((Le & 2) === 0 || e !== Ye) && (e === Ye && ((Le & 2) === 0 && (ta |= a), et === 4 && Er(
      e,
      Re,
      It,
      !1
    )), En(e));
  }
  function X1(e, n, a) {
    if ((Le & 6) !== 0) throw Error(u(327));
    var l = !a && (n & 127) === 0 && (n & e.expiredLanes) === 0 || Lr(e, n), d = l ? kC(e, n) : uf(e, n, !0), y = l;
    do {
      if (d === 0) {
        Za && !l && Er(e, n, 0, !1);
        break;
      } else {
        if (a = e.current.alternate, y && !VC(a)) {
          d = uf(e, n, !1), y = !1;
          continue;
        }
        if (d === 2) {
          if (y = n, e.errorRecoveryDisabledLanes & y)
            var x = 0;
          else
            x = e.pendingLanes & -536870913, x = x !== 0 ? x : x & 536870912 ? 536870912 : 0;
          if (x !== 0) {
            n = x;
            e: {
              var q = e;
              d = iu;
              var U = q.current.memoizedState.isDehydrated;
              if (U && (Fa(q, x).flags |= 256), x = uf(
                q,
                x,
                !1
              ), x !== 2) {
                if (Pc && !U) {
                  q.errorRecoveryDisabledLanes |= y, ta |= y, d = 4;
                  break e;
                }
                y = Lt, Lt = d, y !== null && (Lt === null ? Lt = y : Lt.push.apply(
                  Lt,
                  y
                ));
              }
              d = x;
            }
            if (y = !1, d !== 2) continue;
          }
        }
        if (d === 1) {
          Fa(e, 0), Er(e, n, 0, !0);
          break;
        }
        e: {
          switch (l = e, y = d, y) {
            case 0:
            case 1:
              throw Error(u(345));
            case 4:
              if ((n & 4194048) !== n) break;
            case 6:
              Er(
                l,
                n,
                It,
                !br
              );
              break e;
            case 2:
              Lt = null;
              break;
            case 3:
            case 5:
              break;
            default:
              throw Error(u(329));
          }
          if ((n & 62914560) === n && (d = Ql + 300 - ke(), 10 < d)) {
            if (Er(
              l,
              n,
              It,
              !br
            ), ba(l, 0, !0) !== 0) break e;
            Kn = n, l.timeoutHandle = xb(
              I1.bind(
                null,
                l,
                a,
                Lt,
                Kl,
                ef,
                n,
                It,
                ta,
                Ka,
                br,
                y,
                "Throttled",
                -0,
                0
              ),
              d
            );
            break e;
          }
          I1(
            l,
            a,
            Lt,
            Kl,
            ef,
            n,
            It,
            ta,
            Ka,
            br,
            y,
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
  function I1(e, n, a, l, d, y, x, q, U, te, oe, fe, ne, ue) {
    if (e.timeoutHandle = -1, fe = n.subtreeFlags, fe & 8192 || (fe & 16785408) === 16785408) {
      fe = {
        stylesheets: null,
        count: 0,
        imgCount: 0,
        imgBytes: 0,
        suspenseyImages: [],
        waitingForImages: !0,
        waitingForViewTransition: !1,
        unsuspend: zn
      }, B1(
        n,
        y,
        fe
      );
      var ye = (y & 62914560) === y ? Ql - ke() : (y & 4194048) === y ? V1 - ke() : 0;
      if (ye = AR(
        fe,
        ye
      ), ye !== null) {
        Kn = y, e.cancelPendingCommit = ye(
          W1.bind(
            null,
            e,
            n,
            y,
            a,
            l,
            d,
            x,
            q,
            U,
            oe,
            fe,
            null,
            ne,
            ue
          )
        ), Er(e, y, x, !te);
        return;
      }
    }
    W1(
      e,
      n,
      y,
      a,
      l,
      d,
      x,
      q,
      U
    );
  }
  function VC(e) {
    for (var n = e; ; ) {
      var a = n.tag;
      if ((a === 0 || a === 11 || a === 15) && n.flags & 16384 && (a = n.updateQueue, a !== null && (a = a.stores, a !== null)))
        for (var l = 0; l < a.length; l++) {
          var d = a[l], y = d.getSnapshot;
          d = d.value;
          try {
            if (!Gt(y(), d)) return !1;
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
  function Er(e, n, a, l) {
    n &= ~Wc, n &= ~ta, e.suspendedLanes |= n, e.pingedLanes &= ~n, l && (e.warmLanes |= n), l = e.expirationTimes;
    for (var d = n; 0 < d; ) {
      var y = 31 - Mt(d), x = 1 << y;
      l[y] = -1, d &= ~x;
    }
    a !== 0 && Fu(e, a, n);
  }
  function $l() {
    return (Le & 6) === 0 ? (lu(0), !1) : !0;
  }
  function af() {
    if (qe !== null) {
      if (Be === 0)
        var e = qe.return;
      else
        e = qe, Bn = Qr = null, _c(e), Ga = null, ki = 0, e = qe;
      for (; e !== null; )
        S1(e.alternate, e), e = e.return;
      qe = null;
    }
  }
  function Fa(e, n) {
    var a = e.timeoutHandle;
    a !== -1 && (e.timeoutHandle = -1, lR(a)), a = e.cancelPendingCommit, a !== null && (e.cancelPendingCommit = null, a()), Kn = 0, af(), Ye = e, qe = a = Hn(e.current, null), Re = n, Be = 0, Xt = null, br = !1, Za = Lr(e, n), Pc = !1, Ka = It = Wc = ta = _r = et = 0, Lt = iu = null, ef = !1, (n & 8) !== 0 && (n |= n & 32);
    var l = e.entangledLanes;
    if (l !== 0)
      for (e = e.entanglements, l &= n; 0 < l; ) {
        var d = 31 - Mt(l), y = 1 << d;
        n |= e[d], l &= ~y;
      }
    return Zn = n, yl(), a;
  }
  function Q1(e, n) {
    Te = null, N.H = Ji, n === Ua || n === wl ? (n = o0(), Be = 3) : n === oc ? (n = o0(), Be = 4) : Be = n === Lc ? 8 : n !== null && typeof n == "object" && typeof n.then == "function" ? 6 : 1, Xt = n, qe === null && (et = 1, jl(
      e,
      Ft(n, e.current)
    ));
  }
  function Z1() {
    var e = Yt.current;
    return e === null ? !0 : (Re & 4194048) === Re ? en === null : (Re & 62914560) === Re || (Re & 536870912) !== 0 ? e === en : !1;
  }
  function K1() {
    var e = N.H;
    return N.H = Ji, e === null ? Ji : e;
  }
  function $1() {
    var e = N.A;
    return N.A = UC, e;
  }
  function Fl() {
    et = 4, br || (Re & 4194048) !== Re && Yt.current !== null || (Za = !0), (_r & 134217727) === 0 && (ta & 134217727) === 0 || Ye === null || Er(
      Ye,
      Re,
      It,
      !1
    );
  }
  function uf(e, n, a) {
    var l = Le;
    Le |= 2;
    var d = K1(), y = $1();
    (Ye !== e || Re !== n) && (Kl = null, Fa(e, n)), n = !1;
    var x = et;
    e: do
      try {
        if (Be !== 0 && qe !== null) {
          var q = qe, U = Xt;
          switch (Be) {
            case 8:
              af(), x = 6;
              break e;
            case 3:
            case 2:
            case 9:
            case 6:
              Yt.current === null && (n = !0);
              var te = Be;
              if (Be = 0, Xt = null, Ja(e, q, U, te), a && Za) {
                x = 0;
                break e;
              }
              break;
            default:
              te = Be, Be = 0, Xt = null, Ja(e, q, U, te);
          }
        }
        YC(), x = et;
        break;
      } catch (oe) {
        Q1(e, oe);
      }
    while (!0);
    return n && e.shellSuspendCounter++, Bn = Qr = null, Le = l, N.H = d, N.A = y, qe === null && (Ye = null, Re = 0, yl()), x;
  }
  function YC() {
    for (; qe !== null; ) F1(qe);
  }
  function kC(e, n) {
    var a = Le;
    Le |= 2;
    var l = K1(), d = $1();
    Ye !== e || Re !== n ? (Kl = null, Zl = ke() + 500, Fa(e, n)) : Za = Lr(
      e,
      n
    );
    e: do
      try {
        if (Be !== 0 && qe !== null) {
          n = qe;
          var y = Xt;
          t: switch (Be) {
            case 1:
              Be = 0, Xt = null, Ja(e, n, y, 1);
              break;
            case 2:
            case 9:
              if (u0(y)) {
                Be = 0, Xt = null, J1(n);
                break;
              }
              n = function() {
                Be !== 2 && Be !== 9 || Ye !== e || (Be = 7), En(e);
              }, y.then(n, n);
              break e;
            case 3:
              Be = 7;
              break e;
            case 4:
              Be = 5;
              break e;
            case 7:
              u0(y) ? (Be = 0, Xt = null, J1(n)) : (Be = 0, Xt = null, Ja(e, n, y, 7));
              break;
            case 5:
              var x = null;
              switch (qe.tag) {
                case 26:
                  x = qe.memoizedState;
                case 5:
                case 27:
                  var q = qe;
                  if (x ? Lb(x) : q.stateNode.complete) {
                    Be = 0, Xt = null;
                    var U = q.sibling;
                    if (U !== null) qe = U;
                    else {
                      var te = q.return;
                      te !== null ? (qe = te, Jl(te)) : qe = null;
                    }
                    break t;
                  }
              }
              Be = 0, Xt = null, Ja(e, n, y, 5);
              break;
            case 6:
              Be = 0, Xt = null, Ja(e, n, y, 6);
              break;
            case 8:
              af(), et = 6;
              break e;
            default:
              throw Error(u(462));
          }
        }
        XC();
        break;
      } catch (oe) {
        Q1(e, oe);
      }
    while (!0);
    return Bn = Qr = null, N.H = l, N.A = d, Le = a, qe !== null ? 0 : (Ye = null, Re = 0, yl(), et);
  }
  function XC() {
    for (; qe !== null && !St(); )
      F1(qe);
  }
  function F1(e) {
    var n = _1(e.alternate, e, Zn);
    e.memoizedProps = e.pendingProps, n === null ? Jl(e) : qe = n;
  }
  function J1(e) {
    var n = e, a = n.alternate;
    switch (n.tag) {
      case 15:
      case 0:
        n = g1(
          a,
          n,
          n.pendingProps,
          n.type,
          void 0,
          Re
        );
        break;
      case 11:
        n = g1(
          a,
          n,
          n.pendingProps,
          n.type.render,
          n.ref,
          Re
        );
        break;
      case 5:
        _c(n);
      default:
        S1(a, n), n = qe = $m(n, Zn), n = _1(a, n, Zn);
    }
    e.memoizedProps = e.pendingProps, n === null ? Jl(e) : qe = n;
  }
  function Ja(e, n, a, l) {
    Bn = Qr = null, _c(n), Ga = null, ki = 0;
    var d = n.return;
    try {
      if (OC(
        e,
        d,
        n,
        a,
        Re
      )) {
        et = 1, jl(
          e,
          Ft(a, e.current)
        ), qe = null;
        return;
      }
    } catch (y) {
      if (d !== null) throw qe = d, y;
      et = 1, jl(
        e,
        Ft(a, e.current)
      ), qe = null;
      return;
    }
    n.flags & 32768 ? (Oe || l === 1 ? e = !0 : Za || (Re & 536870912) !== 0 ? e = !1 : (br = e = !0, (l === 2 || l === 9 || l === 3 || l === 6) && (l = Yt.current, l !== null && l.tag === 13 && (l.flags |= 16384))), P1(n, e)) : Jl(n);
  }
  function Jl(e) {
    var n = e;
    do {
      if ((n.flags & 32768) !== 0) {
        P1(
          n,
          br
        );
        return;
      }
      e = n.return;
      var a = HC(
        n.alternate,
        n,
        Zn
      );
      if (a !== null) {
        qe = a;
        return;
      }
      if (n = n.sibling, n !== null) {
        qe = n;
        return;
      }
      qe = n = e;
    } while (n !== null);
    et === 0 && (et = 5);
  }
  function P1(e, n) {
    do {
      var a = LC(e.alternate, e);
      if (a !== null) {
        a.flags &= 32767, qe = a;
        return;
      }
      if (a = e.return, a !== null && (a.flags |= 32768, a.subtreeFlags = 0, a.deletions = null), !n && (e = e.sibling, e !== null)) {
        qe = e;
        return;
      }
      qe = e = a;
    } while (e !== null);
    et = 6, qe = null;
  }
  function W1(e, n, a, l, d, y, x, q, U) {
    e.cancelPendingCommit = null;
    do
      Pl();
    while (ht !== 0);
    if ((Le & 6) !== 0) throw Error(u(327));
    if (n !== null) {
      if (n === e.current) throw Error(u(177));
      if (y = n.lanes | n.childLanes, y |= Zs, ws(
        e,
        a,
        y,
        x,
        q,
        U
      ), e === Ye && (qe = Ye = null, Re = 0), $a = n, Sr = e, Kn = a, tf = y, nf = d, Y1 = l, (n.subtreeFlags & 10256) !== 0 || (n.flags & 10256) !== 0 ? (e.callbackNode = null, e.callbackPriority = 0, KC(ft, function() {
        return ab(), null;
      })) : (e.callbackNode = null, e.callbackPriority = 0), l = (n.flags & 13878) !== 0, (n.subtreeFlags & 13878) !== 0 || l) {
        l = N.T, N.T = null, d = j.p, j.p = 2, x = Le, Le |= 4;
        try {
          BC(e, n, a);
        } finally {
          Le = x, j.p = d, N.T = l;
        }
      }
      ht = 1, eb(), tb(), nb();
    }
  }
  function eb() {
    if (ht === 1) {
      ht = 0;
      var e = Sr, n = $a, a = (n.flags & 13878) !== 0;
      if ((n.subtreeFlags & 13878) !== 0 || a) {
        a = N.T, N.T = null;
        var l = j.p;
        j.p = 2;
        var d = Le;
        Le |= 4;
        try {
          D1(n, e);
          var y = mf, x = Gm(e.containerInfo), q = y.focusedElem, U = y.selectionRange;
          if (x !== q && q && q.ownerDocument && Um(
            q.ownerDocument.documentElement,
            q
          )) {
            if (U !== null && Ys(q)) {
              var te = U.start, oe = U.end;
              if (oe === void 0 && (oe = te), "selectionStart" in q)
                q.selectionStart = te, q.selectionEnd = Math.min(
                  oe,
                  q.value.length
                );
              else {
                var fe = q.ownerDocument || document, ne = fe && fe.defaultView || window;
                if (ne.getSelection) {
                  var ue = ne.getSelection(), ye = q.textContent.length, Ee = Math.min(U.start, ye), Ve = U.end === void 0 ? Ee : Math.min(U.end, ye);
                  !ue.extend && Ee > Ve && (x = Ve, Ve = Ee, Ee = x);
                  var $ = jm(
                    q,
                    Ee
                  ), k = jm(
                    q,
                    Ve
                  );
                  if ($ && k && (ue.rangeCount !== 1 || ue.anchorNode !== $.node || ue.anchorOffset !== $.offset || ue.focusNode !== k.node || ue.focusOffset !== k.offset)) {
                    var ee = fe.createRange();
                    ee.setStart($.node, $.offset), ue.removeAllRanges(), Ee > Ve ? (ue.addRange(ee), ue.extend(k.node, k.offset)) : (ee.setEnd(k.node, k.offset), ue.addRange(ee));
                  }
                }
              }
            }
            for (fe = [], ue = q; ue = ue.parentNode; )
              ue.nodeType === 1 && fe.push({
                element: ue,
                left: ue.scrollLeft,
                top: ue.scrollTop
              });
            for (typeof q.focus == "function" && q.focus(), q = 0; q < fe.length; q++) {
              var ce = fe[q];
              ce.element.scrollLeft = ce.left, ce.element.scrollTop = ce.top;
            }
          }
          co = !!pf, mf = pf = null;
        } finally {
          Le = d, j.p = l, N.T = a;
        }
      }
      e.current = n, ht = 2;
    }
  }
  function tb() {
    if (ht === 2) {
      ht = 0;
      var e = Sr, n = $a, a = (n.flags & 8772) !== 0;
      if ((n.subtreeFlags & 8772) !== 0 || a) {
        a = N.T, N.T = null;
        var l = j.p;
        j.p = 2;
        var d = Le;
        Le |= 4;
        try {
          C1(e, n.alternate, n);
        } finally {
          Le = d, j.p = l, N.T = a;
        }
      }
      ht = 3;
    }
  }
  function nb() {
    if (ht === 4 || ht === 3) {
      ht = 0, Ze();
      var e = Sr, n = $a, a = Kn, l = Y1;
      (n.subtreeFlags & 10256) !== 0 || (n.flags & 10256) !== 0 ? ht = 5 : (ht = 0, $a = Sr = null, rb(e, e.pendingLanes));
      var d = e.pendingLanes;
      if (d === 0 && (xr = null), Ei(a), n = n.stateNode, Fe && typeof Fe.onCommitFiberRoot == "function")
        try {
          Fe.onCommitFiberRoot(
            Et,
            n,
            void 0,
            (n.current.flags & 128) === 128
          );
        } catch {
        }
      if (l !== null) {
        n = N.T, d = j.p, j.p = 2, N.T = null;
        try {
          for (var y = e.onRecoverableError, x = 0; x < l.length; x++) {
            var q = l[x];
            y(q.value, {
              componentStack: q.stack
            });
          }
        } finally {
          N.T = n, j.p = d;
        }
      }
      (Kn & 3) !== 0 && Pl(), En(e), d = e.pendingLanes, (a & 261930) !== 0 && (d & 42) !== 0 ? e === rf ? uu++ : (uu = 0, rf = e) : uu = 0, lu(0);
    }
  }
  function rb(e, n) {
    (e.pooledCacheLanes &= n) === 0 && (n = e.pooledCache, n != null && (e.pooledCache = null, Vi(n)));
  }
  function Pl() {
    return eb(), tb(), nb(), ab();
  }
  function ab() {
    if (ht !== 5) return !1;
    var e = Sr, n = tf;
    tf = 0;
    var a = Ei(Kn), l = N.T, d = j.p;
    try {
      j.p = 32 > a ? 32 : a, N.T = null, a = nf, nf = null;
      var y = Sr, x = Kn;
      if (ht = 0, $a = Sr = null, Kn = 0, (Le & 6) !== 0) throw Error(u(331));
      var q = Le;
      if (Le |= 4, U1(y.current), L1(
        y,
        y.current,
        x,
        a
      ), Le = q, lu(0, !1), Fe && typeof Fe.onPostCommitFiberRoot == "function")
        try {
          Fe.onPostCommitFiberRoot(Et, y);
        } catch {
        }
      return !0;
    } finally {
      j.p = d, N.T = l, rb(e, n);
    }
  }
  function ib(e, n, a) {
    n = Ft(a, n), n = Hc(e.stateNode, n, 2), e = vr(e, n, 2), e !== null && (Br(e, 2), En(e));
  }
  function je(e, n, a) {
    if (e.tag === 3)
      ib(e, e, a);
    else
      for (; n !== null; ) {
        if (n.tag === 3) {
          ib(
            n,
            e,
            a
          );
          break;
        } else if (n.tag === 1) {
          var l = n.stateNode;
          if (typeof n.type.getDerivedStateFromError == "function" || typeof l.componentDidCatch == "function" && (xr === null || !xr.has(l))) {
            e = Ft(a, e), a = u1(2), l = vr(n, a, 2), l !== null && (l1(
              a,
              l,
              n,
              e
            ), Br(l, 2), En(l));
            break;
          }
        }
        n = n.return;
      }
  }
  function lf(e, n, a) {
    var l = e.pingCache;
    if (l === null) {
      l = e.pingCache = new GC();
      var d = /* @__PURE__ */ new Set();
      l.set(n, d);
    } else
      d = l.get(n), d === void 0 && (d = /* @__PURE__ */ new Set(), l.set(n, d));
    d.has(a) || (Pc = !0, d.add(a), e = IC.bind(null, e, n, a), n.then(e, e));
  }
  function IC(e, n, a) {
    var l = e.pingCache;
    l !== null && l.delete(n), e.pingedLanes |= e.suspendedLanes & a, e.warmLanes &= ~a, Ye === e && (Re & a) === a && (et === 4 || et === 3 && (Re & 62914560) === Re && 300 > ke() - Ql ? (Le & 2) === 0 && Fa(e, 0) : Wc |= a, Ka === Re && (Ka = 0)), En(e);
  }
  function ub(e, n) {
    n === 0 && (n = $u()), e = kr(e, n), e !== null && (Br(e, n), En(e));
  }
  function QC(e) {
    var n = e.memoizedState, a = 0;
    n !== null && (a = n.retryLane), ub(e, a);
  }
  function ZC(e, n) {
    var a = 0;
    switch (e.tag) {
      case 31:
      case 13:
        var l = e.stateNode, d = e.memoizedState;
        d !== null && (a = d.retryLane);
        break;
      case 19:
        l = e.stateNode;
        break;
      case 22:
        l = e.stateNode._retryCache;
        break;
      default:
        throw Error(u(314));
    }
    l !== null && l.delete(n), ub(e, a);
  }
  function KC(e, n) {
    return xt(e, n);
  }
  var Wl = null, Pa = null, of = !1, eo = !1, sf = !1, wr = 0;
  function En(e) {
    e !== Pa && e.next === null && (Pa === null ? Wl = Pa = e : Pa = Pa.next = e), eo = !0, of || (of = !0, FC());
  }
  function lu(e, n) {
    if (!sf && eo) {
      sf = !0;
      do
        for (var a = !1, l = Wl; l !== null; ) {
          if (e !== 0) {
            var d = l.pendingLanes;
            if (d === 0) var y = 0;
            else {
              var x = l.suspendedLanes, q = l.pingedLanes;
              y = (1 << 31 - Mt(42 | e) + 1) - 1, y &= d & ~(x & ~q), y = y & 201326741 ? y & 201326741 | 1 : y ? y | 2 : 0;
            }
            y !== 0 && (a = !0, cb(l, y));
          } else
            y = Re, y = ba(
              l,
              l === Ye ? y : 0,
              l.cancelPendingCommit !== null || l.timeoutHandle !== -1
            ), (y & 3) === 0 || Lr(l, y) || (a = !0, cb(l, y));
          l = l.next;
        }
      while (a);
      sf = !1;
    }
  }
  function $C() {
    lb();
  }
  function lb() {
    eo = of = !1;
    var e = 0;
    wr !== 0 && uR() && (e = wr);
    for (var n = ke(), a = null, l = Wl; l !== null; ) {
      var d = l.next, y = ob(l, n);
      y === 0 ? (l.next = null, a === null ? Wl = d : a.next = d, d === null && (Pa = a)) : (a = l, (e !== 0 || (y & 3) !== 0) && (eo = !0)), l = d;
    }
    ht !== 0 && ht !== 5 || lu(e), wr !== 0 && (wr = 0);
  }
  function ob(e, n) {
    for (var a = e.suspendedLanes, l = e.pingedLanes, d = e.expirationTimes, y = e.pendingLanes & -62914561; 0 < y; ) {
      var x = 31 - Mt(y), q = 1 << x, U = d[x];
      U === -1 ? ((q & a) === 0 || (q & l) !== 0) && (d[x] = Es(q, n)) : U <= n && (e.expiredLanes |= q), y &= ~q;
    }
    if (n = Ye, a = Re, a = ba(
      e,
      e === n ? a : 0,
      e.cancelPendingCommit !== null || e.timeoutHandle !== -1
    ), l = e.callbackNode, a === 0 || e === n && (Be === 2 || Be === 9) || e.cancelPendingCommit !== null)
      return l !== null && l !== null && gt(l), e.callbackNode = null, e.callbackPriority = 0;
    if ((a & 3) === 0 || Lr(e, a)) {
      if (n = a & -a, n === e.callbackPriority) return n;
      switch (l !== null && gt(l), Ei(a)) {
        case 2:
        case 8:
          a = Tt;
          break;
        case 32:
          a = ft;
          break;
        case 268435456:
          a = Cn;
          break;
        default:
          a = ft;
      }
      return l = sb.bind(null, e), a = xt(a, l), e.callbackPriority = n, e.callbackNode = a, n;
    }
    return l !== null && l !== null && gt(l), e.callbackPriority = 2, e.callbackNode = null, 2;
  }
  function sb(e, n) {
    if (ht !== 0 && ht !== 5)
      return e.callbackNode = null, e.callbackPriority = 0, null;
    var a = e.callbackNode;
    if (Pl() && e.callbackNode !== a)
      return null;
    var l = Re;
    return l = ba(
      e,
      e === Ye ? l : 0,
      e.cancelPendingCommit !== null || e.timeoutHandle !== -1
    ), l === 0 ? null : (X1(e, l, n), ob(e, ke()), e.callbackNode != null && e.callbackNode === a ? sb.bind(null, e) : null);
  }
  function cb(e, n) {
    if (Pl()) return null;
    X1(e, n, !0);
  }
  function FC() {
    oR(function() {
      (Le & 6) !== 0 ? xt(
        Ct,
        $C
      ) : lb();
    });
  }
  function cf() {
    if (wr === 0) {
      var e = Ba;
      e === 0 && (e = ya, ya <<= 1, (ya & 261888) === 0 && (ya = 256)), wr = e;
    }
    return wr;
  }
  function fb(e) {
    return e == null || typeof e == "symbol" || typeof e == "boolean" ? null : typeof e == "function" ? e : ol("" + e);
  }
  function db(e, n) {
    var a = n.ownerDocument.createElement("input");
    return a.name = n.name, a.value = n.value, e.id && a.setAttribute("form", e.id), n.parentNode.insertBefore(a, n), e = new FormData(e), a.parentNode.removeChild(a), e;
  }
  function JC(e, n, a, l, d) {
    if (n === "submit" && a && a.stateNode === d) {
      var y = fb(
        (d[wt] || null).action
      ), x = l.submitter;
      x && (n = (n = x[wt] || null) ? fb(n.formAction) : x.getAttribute("formAction"), n !== null && (y = n, x = null));
      var q = new dl(
        "action",
        "action",
        null,
        l,
        d
      );
      e.push({
        event: q,
        listeners: [
          {
            instance: null,
            listener: function() {
              if (l.defaultPrevented) {
                if (wr !== 0) {
                  var U = x ? db(d, x) : new FormData(d);
                  Cc(
                    a,
                    {
                      pending: !0,
                      data: U,
                      method: d.method,
                      action: y
                    },
                    null,
                    U
                  );
                }
              } else
                typeof y == "function" && (q.preventDefault(), U = x ? db(d, x) : new FormData(d), Cc(
                  a,
                  {
                    pending: !0,
                    data: U,
                    method: d.method,
                    action: y
                  },
                  y,
                  U
                ));
            },
            currentTarget: d
          }
        ]
      });
    }
  }
  for (var ff = 0; ff < Qs.length; ff++) {
    var df = Qs[ff], PC = df.toLowerCase(), WC = df[0].toUpperCase() + df.slice(1);
    ln(
      PC,
      "on" + WC
    );
  }
  ln(km, "onAnimationEnd"), ln(Xm, "onAnimationIteration"), ln(Im, "onAnimationStart"), ln("dblclick", "onDoubleClick"), ln("focusin", "onFocus"), ln("focusout", "onBlur"), ln(vC, "onTransitionRun"), ln(yC, "onTransitionStart"), ln(pC, "onTransitionCancel"), ln(Qm, "onTransitionEnd"), ur("onMouseEnter", ["mouseout", "mouseover"]), ur("onMouseLeave", ["mouseout", "mouseover"]), ur("onPointerEnter", ["pointerout", "pointerover"]), ur("onPointerLeave", ["pointerout", "pointerover"]), On(
    "onChange",
    "change click focusin focusout input keydown keyup selectionchange".split(" ")
  ), On(
    "onSelect",
    "focusout contextmenu dragend focusin keydown keyup mousedown mouseup selectionchange".split(
      " "
    )
  ), On("onBeforeInput", [
    "compositionend",
    "keypress",
    "textInput",
    "paste"
  ]), On(
    "onCompositionEnd",
    "compositionend focusout keydown keypress keyup mousedown".split(" ")
  ), On(
    "onCompositionStart",
    "compositionstart focusout keydown keypress keyup mousedown".split(" ")
  ), On(
    "onCompositionUpdate",
    "compositionupdate focusout keydown keypress keyup mousedown".split(" ")
  );
  var ou = "abort canplay canplaythrough durationchange emptied encrypted ended error loadeddata loadedmetadata loadstart pause play playing progress ratechange resize seeked seeking stalled suspend timeupdate volumechange waiting".split(
    " "
  ), eR = new Set(
    "beforetoggle cancel close invalid load scroll scrollend toggle".split(" ").concat(ou)
  );
  function hb(e, n) {
    n = (n & 4) !== 0;
    for (var a = 0; a < e.length; a++) {
      var l = e[a], d = l.event;
      l = l.listeners;
      e: {
        var y = void 0;
        if (n)
          for (var x = l.length - 1; 0 <= x; x--) {
            var q = l[x], U = q.instance, te = q.currentTarget;
            if (q = q.listener, U !== y && d.isPropagationStopped())
              break e;
            y = q, d.currentTarget = te;
            try {
              y(d);
            } catch (oe) {
              vl(oe);
            }
            d.currentTarget = null, y = U;
          }
        else
          for (x = 0; x < l.length; x++) {
            if (q = l[x], U = q.instance, te = q.currentTarget, q = q.listener, U !== y && d.isPropagationStopped())
              break e;
            y = q, d.currentTarget = te;
            try {
              y(d);
            } catch (oe) {
              vl(oe);
            }
            d.currentTarget = null, y = U;
          }
      }
    }
  }
  function Ce(e, n) {
    var a = n[_a];
    a === void 0 && (a = n[_a] = /* @__PURE__ */ new Set());
    var l = e + "__bubble";
    a.has(l) || (gb(n, e, 2, !1), a.add(l));
  }
  function hf(e, n, a) {
    var l = 0;
    n && (l |= 4), gb(
      a,
      e,
      l,
      n
    );
  }
  var to = "_reactListening" + Math.random().toString(36).slice(2);
  function gf(e) {
    if (!e[to]) {
      e[to] = !0, rl.forEach(function(a) {
        a !== "selectionchange" && (eR.has(a) || hf(a, !1, e), hf(a, !0, e));
      });
      var n = e.nodeType === 9 ? e : e.ownerDocument;
      n === null || n[to] || (n[to] = !0, hf("selectionchange", !1, n));
    }
  }
  function gb(e, n, a, l) {
    switch (kb(n)) {
      case 2:
        var d = qR;
        break;
      case 8:
        d = CR;
        break;
      default:
        d = Cf;
    }
    a = d.bind(
      null,
      n,
      a,
      e
    ), d = void 0, !zs || n !== "touchstart" && n !== "touchmove" && n !== "wheel" || (d = !0), l ? d !== void 0 ? e.addEventListener(n, a, {
      capture: !0,
      passive: d
    }) : e.addEventListener(n, a, !0) : d !== void 0 ? e.addEventListener(n, a, {
      passive: d
    }) : e.addEventListener(n, a, !1);
  }
  function vf(e, n, a, l, d) {
    var y = l;
    if ((n & 1) === 0 && (n & 2) === 0 && l !== null)
      e: for (; ; ) {
        if (l === null) return;
        var x = l.tag;
        if (x === 3 || x === 4) {
          var q = l.stateNode.containerInfo;
          if (q === d) break;
          if (x === 4)
            for (x = l.return; x !== null; ) {
              var U = x.tag;
              if ((U === 3 || U === 4) && x.stateNode.containerInfo === d)
                return;
              x = x.return;
            }
          for (; q !== null; ) {
            if (x = nr(q), x === null) return;
            if (U = x.tag, U === 5 || U === 6 || U === 26 || U === 27) {
              l = y = x;
              continue e;
            }
            q = q.parentNode;
          }
        }
        l = l.return;
      }
    bm(function() {
      var te = y, oe = Ns(a), fe = [];
      e: {
        var ne = Zm.get(e);
        if (ne !== void 0) {
          var ue = dl, ye = e;
          switch (e) {
            case "keypress":
              if (cl(a) === 0) break e;
            case "keydown":
            case "keyup":
              ue = Zq;
              break;
            case "focusin":
              ye = "focus", ue = Bs;
              break;
            case "focusout":
              ye = "blur", ue = Bs;
              break;
            case "beforeblur":
            case "afterblur":
              ue = Bs;
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
              ue = Sm;
              break;
            case "drag":
            case "dragend":
            case "dragenter":
            case "dragexit":
            case "dragleave":
            case "dragover":
            case "dragstart":
            case "drop":
              ue = Hq;
              break;
            case "touchcancel":
            case "touchend":
            case "touchmove":
            case "touchstart":
              ue = Fq;
              break;
            case km:
            case Xm:
            case Im:
              ue = jq;
              break;
            case Qm:
              ue = Pq;
              break;
            case "scroll":
            case "scrollend":
              ue = zq;
              break;
            case "wheel":
              ue = eC;
              break;
            case "copy":
            case "cut":
            case "paste":
              ue = Gq;
              break;
            case "gotpointercapture":
            case "lostpointercapture":
            case "pointercancel":
            case "pointerdown":
            case "pointermove":
            case "pointerout":
            case "pointerover":
            case "pointerup":
              ue = wm;
              break;
            case "toggle":
            case "beforetoggle":
              ue = nC;
          }
          var Ee = (n & 4) !== 0, Ve = !Ee && (e === "scroll" || e === "scrollend"), $ = Ee ? ne !== null ? ne + "Capture" : null : ne;
          Ee = [];
          for (var k = te, ee; k !== null; ) {
            var ce = k;
            if (ee = ce.stateNode, ce = ce.tag, ce !== 5 && ce !== 26 && ce !== 27 || ee === null || $ === null || (ce = Ri(k, $), ce != null && Ee.push(
              su(k, ce, ee)
            )), Ve) break;
            k = k.return;
          }
          0 < Ee.length && (ne = new ue(
            ne,
            ye,
            null,
            a,
            oe
          ), fe.push({ event: ne, listeners: Ee }));
        }
      }
      if ((n & 7) === 0) {
        e: {
          if (ne = e === "mouseover" || e === "pointerover", ue = e === "mouseout" || e === "pointerout", ne && a !== Rs && (ye = a.relatedTarget || a.fromElement) && (nr(ye) || ye[Nn]))
            break e;
          if ((ue || ne) && (ne = oe.window === oe ? oe : (ne = oe.ownerDocument) ? ne.defaultView || ne.parentWindow : window, ue ? (ye = a.relatedTarget || a.toElement, ue = te, ye = ye ? nr(ye) : null, ye !== null && (Ve = s(ye), Ee = ye.tag, ye !== Ve || Ee !== 5 && Ee !== 27 && Ee !== 6) && (ye = null)) : (ue = null, ye = te), ue !== ye)) {
            if (Ee = Sm, ce = "onMouseLeave", $ = "onMouseEnter", k = "mouse", (e === "pointerout" || e === "pointerover") && (Ee = wm, ce = "onPointerLeave", $ = "onPointerEnter", k = "pointer"), Ve = ue == null ? ne : ar(ue), ee = ye == null ? ne : ar(ye), ne = new Ee(
              ce,
              k + "leave",
              ue,
              a,
              oe
            ), ne.target = Ve, ne.relatedTarget = ee, ce = null, nr(oe) === te && (Ee = new Ee(
              $,
              k + "enter",
              ye,
              a,
              oe
            ), Ee.target = ee, Ee.relatedTarget = Ve, ce = Ee), Ve = ce, ue && ye)
              t: {
                for (Ee = tR, $ = ue, k = ye, ee = 0, ce = $; ce; ce = Ee(ce))
                  ee++;
                ce = 0;
                for (var Se = k; Se; Se = Ee(Se))
                  ce++;
                for (; 0 < ee - ce; )
                  $ = Ee($), ee--;
                for (; 0 < ce - ee; )
                  k = Ee(k), ce--;
                for (; ee--; ) {
                  if ($ === k || k !== null && $ === k.alternate) {
                    Ee = $;
                    break t;
                  }
                  $ = Ee($), k = Ee(k);
                }
                Ee = null;
              }
            else Ee = null;
            ue !== null && vb(
              fe,
              ne,
              ue,
              Ee,
              !1
            ), ye !== null && Ve !== null && vb(
              fe,
              Ve,
              ye,
              Ee,
              !0
            );
          }
        }
        e: {
          if (ne = te ? ar(te) : window, ue = ne.nodeName && ne.nodeName.toLowerCase(), ue === "select" || ue === "input" && ne.type === "file")
            var De = Om;
          else if (Rm(ne))
            if (zm)
              De = dC;
            else {
              De = cC;
              var be = sC;
            }
          else
            ue = ne.nodeName, !ue || ue.toLowerCase() !== "input" || ne.type !== "checkbox" && ne.type !== "radio" ? te && Cs(te.elementType) && (De = Om) : De = fC;
          if (De && (De = De(e, te))) {
            Nm(
              fe,
              De,
              a,
              oe
            );
            break e;
          }
          be && be(e, ne, te), e === "focusout" && te && ne.type === "number" && te.memoizedProps.value != null && qi(ne, "number", ne.value);
        }
        switch (be = te ? ar(te) : window, e) {
          case "focusin":
            (Rm(be) || be.contentEditable === "true") && (Ca = be, ks = te, ji = null);
            break;
          case "focusout":
            ji = ks = Ca = null;
            break;
          case "mousedown":
            Xs = !0;
            break;
          case "contextmenu":
          case "mouseup":
          case "dragend":
            Xs = !1, Vm(fe, a, oe);
            break;
          case "selectionchange":
            if (gC) break;
          case "keydown":
          case "keyup":
            Vm(fe, a, oe);
        }
        var Me;
        if (Us)
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
          qa ? qm(e, a) && (Ne = "onCompositionEnd") : e === "keydown" && a.keyCode === 229 && (Ne = "onCompositionStart");
        Ne && (Am && a.locale !== "ko" && (qa || Ne !== "onCompositionStart" ? Ne === "onCompositionEnd" && qa && (Me = _m()) : (or = oe, Ds = "value" in or ? or.value : or.textContent, qa = !0)), be = no(te, Ne), 0 < be.length && (Ne = new Em(
          Ne,
          e,
          null,
          a,
          oe
        ), fe.push({ event: Ne, listeners: be }), Me ? Ne.data = Me : (Me = Cm(a), Me !== null && (Ne.data = Me)))), (Me = aC ? iC(e, a) : uC(e, a)) && (Ne = no(te, "onBeforeInput"), 0 < Ne.length && (be = new Em(
          "onBeforeInput",
          "beforeinput",
          null,
          a,
          oe
        ), fe.push({
          event: be,
          listeners: Ne
        }), be.data = Me)), JC(
          fe,
          e,
          te,
          a,
          oe
        );
      }
      hb(fe, n);
    });
  }
  function su(e, n, a) {
    return {
      instance: e,
      listener: n,
      currentTarget: a
    };
  }
  function no(e, n) {
    for (var a = n + "Capture", l = []; e !== null; ) {
      var d = e, y = d.stateNode;
      if (d = d.tag, d !== 5 && d !== 26 && d !== 27 || y === null || (d = Ri(e, a), d != null && l.unshift(
        su(e, d, y)
      ), d = Ri(e, n), d != null && l.push(
        su(e, d, y)
      )), e.tag === 3) return l;
      e = e.return;
    }
    return [];
  }
  function tR(e) {
    if (e === null) return null;
    do
      e = e.return;
    while (e && e.tag !== 5 && e.tag !== 27);
    return e || null;
  }
  function vb(e, n, a, l, d) {
    for (var y = n._reactName, x = []; a !== null && a !== l; ) {
      var q = a, U = q.alternate, te = q.stateNode;
      if (q = q.tag, U !== null && U === l) break;
      q !== 5 && q !== 26 && q !== 27 || te === null || (U = te, d ? (te = Ri(a, y), te != null && x.unshift(
        su(a, te, U)
      )) : d || (te = Ri(a, y), te != null && x.push(
        su(a, te, U)
      ))), a = a.return;
    }
    x.length !== 0 && e.push({ event: n, listeners: x });
  }
  var nR = /\r\n?/g, rR = /\u0000|\uFFFD/g;
  function yb(e) {
    return (typeof e == "string" ? e : "" + e).replace(nR, `
`).replace(rR, "");
  }
  function pb(e, n) {
    return n = yb(n), yb(e) === n;
  }
  function Ge(e, n, a, l, d, y) {
    switch (a) {
      case "children":
        typeof l == "string" ? n === "body" || n === "textarea" && l === "" || Aa(e, l) : (typeof l == "number" || typeof l == "bigint") && n !== "body" && Aa(e, "" + l);
        break;
      case "className":
        Sa(e, "class", l);
        break;
      case "tabIndex":
        Sa(e, "tabindex", l);
        break;
      case "dir":
      case "role":
      case "viewBox":
      case "width":
      case "height":
        Sa(e, a, l);
        break;
      case "style":
        pm(e, l, y);
        break;
      case "data":
        if (n !== "object") {
          Sa(e, "data", l);
          break;
        }
      case "src":
      case "href":
        if (l === "" && (n !== "a" || a !== "href")) {
          e.removeAttribute(a);
          break;
        }
        if (l == null || typeof l == "function" || typeof l == "symbol" || typeof l == "boolean") {
          e.removeAttribute(a);
          break;
        }
        l = ol("" + l), e.setAttribute(a, l);
        break;
      case "action":
      case "formAction":
        if (typeof l == "function") {
          e.setAttribute(
            a,
            "javascript:throw new Error('A React form was unexpectedly submitted. If you called form.submit() manually, consider using form.requestSubmit() instead. If you\\'re trying to use event.stopPropagation() in a submit event handler, consider also calling event.preventDefault().')"
          );
          break;
        } else
          typeof y == "function" && (a === "formAction" ? (n !== "input" && Ge(e, n, "name", d.name, d, null), Ge(
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
        if (l == null || typeof l == "symbol" || typeof l == "boolean") {
          e.removeAttribute(a);
          break;
        }
        l = ol("" + l), e.setAttribute(a, l);
        break;
      case "onClick":
        l != null && (e.onclick = zn);
        break;
      case "onScroll":
        l != null && Ce("scroll", e);
        break;
      case "onScrollEnd":
        l != null && Ce("scrollend", e);
        break;
      case "dangerouslySetInnerHTML":
        if (l != null) {
          if (typeof l != "object" || !("__html" in l))
            throw Error(u(61));
          if (a = l.__html, a != null) {
            if (d.children != null) throw Error(u(60));
            e.innerHTML = a;
          }
        }
        break;
      case "multiple":
        e.multiple = l && typeof l != "function" && typeof l != "symbol";
        break;
      case "muted":
        e.muted = l && typeof l != "function" && typeof l != "symbol";
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
        if (l == null || typeof l == "function" || typeof l == "boolean" || typeof l == "symbol") {
          e.removeAttribute("xlink:href");
          break;
        }
        a = ol("" + l), e.setAttributeNS(
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
        l != null && typeof l != "function" && typeof l != "symbol" ? e.setAttribute(a, "" + l) : e.removeAttribute(a);
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
        l && typeof l != "function" && typeof l != "symbol" ? e.setAttribute(a, "") : e.removeAttribute(a);
        break;
      case "capture":
      case "download":
        l === !0 ? e.setAttribute(a, "") : l !== !1 && l != null && typeof l != "function" && typeof l != "symbol" ? e.setAttribute(a, l) : e.removeAttribute(a);
        break;
      case "cols":
      case "rows":
      case "size":
      case "span":
        l != null && typeof l != "function" && typeof l != "symbol" && !isNaN(l) && 1 <= l ? e.setAttribute(a, l) : e.removeAttribute(a);
        break;
      case "rowSpan":
      case "start":
        l == null || typeof l == "function" || typeof l == "symbol" || isNaN(l) ? e.removeAttribute(a) : e.setAttribute(a, l);
        break;
      case "popover":
        Ce("beforetoggle", e), Ce("toggle", e), xa(e, "popover", l);
        break;
      case "xlinkActuate":
        un(
          e,
          "http://www.w3.org/1999/xlink",
          "xlink:actuate",
          l
        );
        break;
      case "xlinkArcrole":
        un(
          e,
          "http://www.w3.org/1999/xlink",
          "xlink:arcrole",
          l
        );
        break;
      case "xlinkRole":
        un(
          e,
          "http://www.w3.org/1999/xlink",
          "xlink:role",
          l
        );
        break;
      case "xlinkShow":
        un(
          e,
          "http://www.w3.org/1999/xlink",
          "xlink:show",
          l
        );
        break;
      case "xlinkTitle":
        un(
          e,
          "http://www.w3.org/1999/xlink",
          "xlink:title",
          l
        );
        break;
      case "xlinkType":
        un(
          e,
          "http://www.w3.org/1999/xlink",
          "xlink:type",
          l
        );
        break;
      case "xmlBase":
        un(
          e,
          "http://www.w3.org/XML/1998/namespace",
          "xml:base",
          l
        );
        break;
      case "xmlLang":
        un(
          e,
          "http://www.w3.org/XML/1998/namespace",
          "xml:lang",
          l
        );
        break;
      case "xmlSpace":
        un(
          e,
          "http://www.w3.org/XML/1998/namespace",
          "xml:space",
          l
        );
        break;
      case "is":
        xa(e, "is", l);
        break;
      case "innerText":
      case "textContent":
        break;
      default:
        (!(2 < a.length) || a[0] !== "o" && a[0] !== "O" || a[1] !== "n" && a[1] !== "N") && (a = Nq.get(a) || a, xa(e, a, l));
    }
  }
  function yf(e, n, a, l, d, y) {
    switch (a) {
      case "style":
        pm(e, l, y);
        break;
      case "dangerouslySetInnerHTML":
        if (l != null) {
          if (typeof l != "object" || !("__html" in l))
            throw Error(u(61));
          if (a = l.__html, a != null) {
            if (d.children != null) throw Error(u(60));
            e.innerHTML = a;
          }
        }
        break;
      case "children":
        typeof l == "string" ? Aa(e, l) : (typeof l == "number" || typeof l == "bigint") && Aa(e, "" + l);
        break;
      case "onScroll":
        l != null && Ce("scroll", e);
        break;
      case "onScrollEnd":
        l != null && Ce("scrollend", e);
        break;
      case "onClick":
        l != null && (e.onclick = zn);
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
        if (!al.hasOwnProperty(a))
          e: {
            if (a[0] === "o" && a[1] === "n" && (d = a.endsWith("Capture"), n = a.slice(2, d ? a.length - 7 : void 0), y = e[wt] || null, y = y != null ? y[a] : null, typeof y == "function" && e.removeEventListener(n, y, d), typeof l == "function")) {
              typeof y != "function" && y !== null && (a in e ? e[a] = null : e.hasAttribute(a) && e.removeAttribute(a)), e.addEventListener(n, l, d);
              break e;
            }
            a in e ? e[a] = l : l === !0 ? e.setAttribute(a, "") : xa(e, a, l);
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
        Ce("error", e), Ce("load", e);
        var l = !1, d = !1, y;
        for (y in a)
          if (a.hasOwnProperty(y)) {
            var x = a[y];
            if (x != null)
              switch (y) {
                case "src":
                  l = !0;
                  break;
                case "srcSet":
                  d = !0;
                  break;
                case "children":
                case "dangerouslySetInnerHTML":
                  throw Error(u(137, n));
                default:
                  Ge(e, n, y, x, a, null);
              }
          }
        d && Ge(e, n, "srcSet", a.srcSet, a, null), l && Ge(e, n, "src", a.src, a, null);
        return;
      case "input":
        Ce("invalid", e);
        var q = y = x = d = null, U = null, te = null;
        for (l in a)
          if (a.hasOwnProperty(l)) {
            var oe = a[l];
            if (oe != null)
              switch (l) {
                case "name":
                  d = oe;
                  break;
                case "type":
                  x = oe;
                  break;
                case "checked":
                  U = oe;
                  break;
                case "defaultChecked":
                  te = oe;
                  break;
                case "value":
                  y = oe;
                  break;
                case "defaultValue":
                  q = oe;
                  break;
                case "children":
                case "dangerouslySetInnerHTML":
                  if (oe != null)
                    throw Error(u(137, n));
                  break;
                default:
                  Ge(e, n, l, oe, a, null);
              }
          }
        wa(
          e,
          y,
          q,
          U,
          te,
          x,
          d,
          !1
        );
        return;
      case "select":
        Ce("invalid", e), l = x = y = null;
        for (d in a)
          if (a.hasOwnProperty(d) && (q = a[d], q != null))
            switch (d) {
              case "value":
                y = q;
                break;
              case "defaultValue":
                x = q;
                break;
              case "multiple":
                l = q;
              default:
                Ge(e, n, d, q, a, null);
            }
        n = y, a = x, e.multiple = !!l, n != null ? lr(e, !!l, n, !1) : a != null && lr(e, !!l, a, !0);
        return;
      case "textarea":
        Ce("invalid", e), y = d = l = null;
        for (x in a)
          if (a.hasOwnProperty(x) && (q = a[x], q != null))
            switch (x) {
              case "value":
                l = q;
                break;
              case "defaultValue":
                d = q;
                break;
              case "children":
                y = q;
                break;
              case "dangerouslySetInnerHTML":
                if (q != null) throw Error(u(91));
                break;
              default:
                Ge(e, n, x, q, a, null);
            }
        vm(e, l, d, y);
        return;
      case "option":
        for (U in a)
          if (a.hasOwnProperty(U) && (l = a[U], l != null))
            switch (U) {
              case "selected":
                e.selected = l && typeof l != "function" && typeof l != "symbol";
                break;
              default:
                Ge(e, n, U, l, a, null);
            }
        return;
      case "dialog":
        Ce("beforetoggle", e), Ce("toggle", e), Ce("cancel", e), Ce("close", e);
        break;
      case "iframe":
      case "object":
        Ce("load", e);
        break;
      case "video":
      case "audio":
        for (l = 0; l < ou.length; l++)
          Ce(ou[l], e);
        break;
      case "image":
        Ce("error", e), Ce("load", e);
        break;
      case "details":
        Ce("toggle", e);
        break;
      case "embed":
      case "source":
      case "link":
        Ce("error", e), Ce("load", e);
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
          if (a.hasOwnProperty(te) && (l = a[te], l != null))
            switch (te) {
              case "children":
              case "dangerouslySetInnerHTML":
                throw Error(u(137, n));
              default:
                Ge(e, n, te, l, a, null);
            }
        return;
      default:
        if (Cs(n)) {
          for (oe in a)
            a.hasOwnProperty(oe) && (l = a[oe], l !== void 0 && yf(
              e,
              n,
              oe,
              l,
              a,
              void 0
            ));
          return;
        }
    }
    for (q in a)
      a.hasOwnProperty(q) && (l = a[q], l != null && Ge(e, n, q, l, a, null));
  }
  function aR(e, n, a, l) {
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
        var d = null, y = null, x = null, q = null, U = null, te = null, oe = null;
        for (ue in a) {
          var fe = a[ue];
          if (a.hasOwnProperty(ue) && fe != null)
            switch (ue) {
              case "checked":
                break;
              case "value":
                break;
              case "defaultValue":
                U = fe;
              default:
                l.hasOwnProperty(ue) || Ge(e, n, ue, null, l, fe);
            }
        }
        for (var ne in l) {
          var ue = l[ne];
          if (fe = a[ne], l.hasOwnProperty(ne) && (ue != null || fe != null))
            switch (ne) {
              case "type":
                y = ue;
                break;
              case "name":
                d = ue;
                break;
              case "checked":
                te = ue;
                break;
              case "defaultChecked":
                oe = ue;
                break;
              case "value":
                x = ue;
                break;
              case "defaultValue":
                q = ue;
                break;
              case "children":
              case "dangerouslySetInnerHTML":
                if (ue != null)
                  throw Error(u(137, n));
                break;
              default:
                ue !== fe && Ge(
                  e,
                  n,
                  ne,
                  ue,
                  l,
                  fe
                );
            }
        }
        Mi(
          e,
          x,
          q,
          U,
          te,
          oe,
          y,
          d
        );
        return;
      case "select":
        ue = x = q = ne = null;
        for (y in a)
          if (U = a[y], a.hasOwnProperty(y) && U != null)
            switch (y) {
              case "value":
                break;
              case "multiple":
                ue = U;
              default:
                l.hasOwnProperty(y) || Ge(
                  e,
                  n,
                  y,
                  null,
                  l,
                  U
                );
            }
        for (d in l)
          if (y = l[d], U = a[d], l.hasOwnProperty(d) && (y != null || U != null))
            switch (d) {
              case "value":
                ne = y;
                break;
              case "defaultValue":
                q = y;
                break;
              case "multiple":
                x = y;
              default:
                y !== U && Ge(
                  e,
                  n,
                  d,
                  y,
                  l,
                  U
                );
            }
        n = q, a = x, l = ue, ne != null ? lr(e, !!a, ne, !1) : !!l != !!a && (n != null ? lr(e, !!a, n, !0) : lr(e, !!a, a ? [] : "", !1));
        return;
      case "textarea":
        ue = ne = null;
        for (q in a)
          if (d = a[q], a.hasOwnProperty(q) && d != null && !l.hasOwnProperty(q))
            switch (q) {
              case "value":
                break;
              case "children":
                break;
              default:
                Ge(e, n, q, null, l, d);
            }
        for (x in l)
          if (d = l[x], y = a[x], l.hasOwnProperty(x) && (d != null || y != null))
            switch (x) {
              case "value":
                ne = d;
                break;
              case "defaultValue":
                ue = d;
                break;
              case "children":
                break;
              case "dangerouslySetInnerHTML":
                if (d != null) throw Error(u(91));
                break;
              default:
                d !== y && Ge(e, n, x, d, l, y);
            }
        Ci(e, ne, ue);
        return;
      case "option":
        for (var ye in a)
          if (ne = a[ye], a.hasOwnProperty(ye) && ne != null && !l.hasOwnProperty(ye))
            switch (ye) {
              case "selected":
                e.selected = !1;
                break;
              default:
                Ge(
                  e,
                  n,
                  ye,
                  null,
                  l,
                  ne
                );
            }
        for (U in l)
          if (ne = l[U], ue = a[U], l.hasOwnProperty(U) && ne !== ue && (ne != null || ue != null))
            switch (U) {
              case "selected":
                e.selected = ne && typeof ne != "function" && typeof ne != "symbol";
                break;
              default:
                Ge(
                  e,
                  n,
                  U,
                  ne,
                  l,
                  ue
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
          ne = a[Ee], a.hasOwnProperty(Ee) && ne != null && !l.hasOwnProperty(Ee) && Ge(e, n, Ee, null, l, ne);
        for (te in l)
          if (ne = l[te], ue = a[te], l.hasOwnProperty(te) && ne !== ue && (ne != null || ue != null))
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
                  l,
                  ue
                );
            }
        return;
      default:
        if (Cs(n)) {
          for (var Ve in a)
            ne = a[Ve], a.hasOwnProperty(Ve) && ne !== void 0 && !l.hasOwnProperty(Ve) && yf(
              e,
              n,
              Ve,
              void 0,
              l,
              ne
            );
          for (oe in l)
            ne = l[oe], ue = a[oe], !l.hasOwnProperty(oe) || ne === ue || ne === void 0 && ue === void 0 || yf(
              e,
              n,
              oe,
              ne,
              l,
              ue
            );
          return;
        }
    }
    for (var $ in a)
      ne = a[$], a.hasOwnProperty($) && ne != null && !l.hasOwnProperty($) && Ge(e, n, $, null, l, ne);
    for (fe in l)
      ne = l[fe], ue = a[fe], !l.hasOwnProperty(fe) || ne === ue || ne == null && ue == null || Ge(e, n, fe, ne, l, ue);
  }
  function mb(e) {
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
  function iR() {
    if (typeof performance.getEntriesByType == "function") {
      for (var e = 0, n = 0, a = performance.getEntriesByType("resource"), l = 0; l < a.length; l++) {
        var d = a[l], y = d.transferSize, x = d.initiatorType, q = d.duration;
        if (y && q && mb(x)) {
          for (x = 0, q = d.responseEnd, l += 1; l < a.length; l++) {
            var U = a[l], te = U.startTime;
            if (te > q) break;
            var oe = U.transferSize, fe = U.initiatorType;
            oe && mb(fe) && (U = U.responseEnd, x += oe * (U < q ? 1 : (q - te) / (U - te)));
          }
          if (--l, n += 8 * (y + x) / (d.duration / 1e3), e++, 10 < e) break;
        }
      }
      if (0 < e) return n / e / 1e6;
    }
    return navigator.connection && (e = navigator.connection.downlink, typeof e == "number") ? e : 5;
  }
  var pf = null, mf = null;
  function ro(e) {
    return e.nodeType === 9 ? e : e.ownerDocument;
  }
  function bb(e) {
    switch (e) {
      case "http://www.w3.org/2000/svg":
        return 1;
      case "http://www.w3.org/1998/Math/MathML":
        return 2;
      default:
        return 0;
    }
  }
  function _b(e, n) {
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
  function bf(e, n) {
    return e === "textarea" || e === "noscript" || typeof n.children == "string" || typeof n.children == "number" || typeof n.children == "bigint" || typeof n.dangerouslySetInnerHTML == "object" && n.dangerouslySetInnerHTML !== null && n.dangerouslySetInnerHTML.__html != null;
  }
  var _f = null;
  function uR() {
    var e = window.event;
    return e && e.type === "popstate" ? e === _f ? !1 : (_f = e, !0) : (_f = null, !1);
  }
  var xb = typeof setTimeout == "function" ? setTimeout : void 0, lR = typeof clearTimeout == "function" ? clearTimeout : void 0, Sb = typeof Promise == "function" ? Promise : void 0, oR = typeof queueMicrotask == "function" ? queueMicrotask : typeof Sb < "u" ? function(e) {
    return Sb.resolve(null).then(e).catch(sR);
  } : xb;
  function sR(e) {
    setTimeout(function() {
      throw e;
    });
  }
  function Ar(e) {
    return e === "head";
  }
  function Eb(e, n) {
    var a = n, l = 0;
    do {
      var d = a.nextSibling;
      if (e.removeChild(a), d && d.nodeType === 8)
        if (a = d.data, a === "/$" || a === "/&") {
          if (l === 0) {
            e.removeChild(d), ni(n);
            return;
          }
          l--;
        } else if (a === "$" || a === "$?" || a === "$~" || a === "$!" || a === "&")
          l++;
        else if (a === "html")
          cu(e.ownerDocument.documentElement);
        else if (a === "head") {
          a = e.ownerDocument.head, cu(a);
          for (var y = a.firstChild; y; ) {
            var x = y.nextSibling, q = y.nodeName;
            y[jr] || q === "SCRIPT" || q === "STYLE" || q === "LINK" && y.rel.toLowerCase() === "stylesheet" || a.removeChild(y), y = x;
          }
        } else
          a === "body" && cu(e.ownerDocument.body);
      a = d;
    } while (a);
    ni(n);
  }
  function wb(e, n) {
    var a = e;
    e = 0;
    do {
      var l = a.nextSibling;
      if (a.nodeType === 1 ? n ? (a._stashedDisplay = a.style.display, a.style.display = "none") : (a.style.display = a._stashedDisplay || "", a.getAttribute("style") === "" && a.removeAttribute("style")) : a.nodeType === 3 && (n ? (a._stashedText = a.nodeValue, a.nodeValue = "") : a.nodeValue = a._stashedText || ""), l && l.nodeType === 8)
        if (a = l.data, a === "/$") {
          if (e === 0) break;
          e--;
        } else
          a !== "$" && a !== "$?" && a !== "$~" && a !== "$!" || e++;
      a = l;
    } while (a);
  }
  function xf(e) {
    var n = e.firstChild;
    for (n && n.nodeType === 10 && (n = n.nextSibling); n; ) {
      var a = n;
      switch (n = n.nextSibling, a.nodeName) {
        case "HTML":
        case "HEAD":
        case "BODY":
          xf(a), wi(a);
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
  function cR(e, n, a, l) {
    for (; e.nodeType === 1; ) {
      var d = a;
      if (e.nodeName.toLowerCase() !== n.toLowerCase()) {
        if (!l && (e.nodeName !== "INPUT" || e.type !== "hidden"))
          break;
      } else if (l) {
        if (!e[jr])
          switch (n) {
            case "meta":
              if (!e.hasAttribute("itemprop")) break;
              return e;
            case "link":
              if (y = e.getAttribute("rel"), y === "stylesheet" && e.hasAttribute("data-precedence"))
                break;
              if (y !== d.rel || e.getAttribute("href") !== (d.href == null || d.href === "" ? null : d.href) || e.getAttribute("crossorigin") !== (d.crossOrigin == null ? null : d.crossOrigin) || e.getAttribute("title") !== (d.title == null ? null : d.title))
                break;
              return e;
            case "style":
              if (e.hasAttribute("data-precedence")) break;
              return e;
            case "script":
              if (y = e.getAttribute("src"), (y !== (d.src == null ? null : d.src) || e.getAttribute("type") !== (d.type == null ? null : d.type) || e.getAttribute("crossorigin") !== (d.crossOrigin == null ? null : d.crossOrigin)) && y && e.hasAttribute("async") && !e.hasAttribute("itemprop"))
                break;
              return e;
            default:
              return e;
          }
      } else if (n === "input" && e.type === "hidden") {
        var y = d.name == null ? null : "" + d.name;
        if (d.type === "hidden" && e.getAttribute("name") === y)
          return e;
      } else return e;
      if (e = tn(e.nextSibling), e === null) break;
    }
    return null;
  }
  function fR(e, n, a) {
    if (n === "") return null;
    for (; e.nodeType !== 3; )
      if ((e.nodeType !== 1 || e.nodeName !== "INPUT" || e.type !== "hidden") && !a || (e = tn(e.nextSibling), e === null)) return null;
    return e;
  }
  function Ab(e, n) {
    for (; e.nodeType !== 8; )
      if ((e.nodeType !== 1 || e.nodeName !== "INPUT" || e.type !== "hidden") && !n || (e = tn(e.nextSibling), e === null)) return null;
    return e;
  }
  function Sf(e) {
    return e.data === "$?" || e.data === "$~";
  }
  function Ef(e) {
    return e.data === "$!" || e.data === "$?" && e.ownerDocument.readyState !== "loading";
  }
  function dR(e, n) {
    var a = e.ownerDocument;
    if (e.data === "$~") e._reactRetry = n;
    else if (e.data !== "$?" || a.readyState !== "loading")
      n();
    else {
      var l = function() {
        n(), a.removeEventListener("DOMContentLoaded", l);
      };
      a.addEventListener("DOMContentLoaded", l), e._reactRetry = l;
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
  var wf = null;
  function Tb(e) {
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
  function Mb(e) {
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
  function qb(e, n, a) {
    switch (n = ro(a), e) {
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
  var nn = /* @__PURE__ */ new Map(), Cb = /* @__PURE__ */ new Set();
  function ao(e) {
    return typeof e.getRootNode == "function" ? e.getRootNode() : e.nodeType === 9 ? e : e.ownerDocument;
  }
  var $n = j.d;
  j.d = {
    f: hR,
    r: gR,
    D: vR,
    C: yR,
    L: pR,
    m: mR,
    X: _R,
    S: bR,
    M: xR
  };
  function hR() {
    var e = $n.f(), n = $l();
    return e || n;
  }
  function gR(e) {
    var n = rr(e);
    n !== null && n.tag === 5 && n.type === "form" ? Q0(n) : $n.r(e);
  }
  var Wa = typeof document > "u" ? null : document;
  function Rb(e, n, a) {
    var l = Wa;
    if (l && typeof n == "string" && n) {
      var d = Nt(n);
      d = 'link[rel="' + e + '"][href="' + d + '"]', typeof a == "string" && (d += '[crossorigin="' + a + '"]'), Cb.has(d) || (Cb.add(d), e = { rel: e, crossOrigin: a, href: n }, l.querySelector(d) === null && (n = l.createElement("link"), _t(n, "link", e), at(n), l.head.appendChild(n)));
    }
  }
  function vR(e) {
    $n.D(e), Rb("dns-prefetch", e, null);
  }
  function yR(e, n) {
    $n.C(e, n), Rb("preconnect", e, n);
  }
  function pR(e, n, a) {
    $n.L(e, n, a);
    var l = Wa;
    if (l && e && n) {
      var d = 'link[rel="preload"][as="' + Nt(n) + '"]';
      n === "image" && a && a.imageSrcSet ? (d += '[imagesrcset="' + Nt(
        a.imageSrcSet
      ) + '"]', typeof a.imageSizes == "string" && (d += '[imagesizes="' + Nt(
        a.imageSizes
      ) + '"]')) : d += '[href="' + Nt(e) + '"]';
      var y = d;
      switch (n) {
        case "style":
          y = ei(e);
          break;
        case "script":
          y = ti(e);
      }
      nn.has(y) || (e = p(
        {
          rel: "preload",
          href: n === "image" && a && a.imageSrcSet ? void 0 : e,
          as: n
        },
        a
      ), nn.set(y, e), l.querySelector(d) !== null || n === "style" && l.querySelector(fu(y)) || n === "script" && l.querySelector(du(y)) || (n = l.createElement("link"), _t(n, "link", e), at(n), l.head.appendChild(n)));
    }
  }
  function mR(e, n) {
    $n.m(e, n);
    var a = Wa;
    if (a && e) {
      var l = n && typeof n.as == "string" ? n.as : "script", d = 'link[rel="modulepreload"][as="' + Nt(l) + '"][href="' + Nt(e) + '"]', y = d;
      switch (l) {
        case "audioworklet":
        case "paintworklet":
        case "serviceworker":
        case "sharedworker":
        case "worker":
        case "script":
          y = ti(e);
      }
      if (!nn.has(y) && (e = p({ rel: "modulepreload", href: e }, n), nn.set(y, e), a.querySelector(d) === null)) {
        switch (l) {
          case "audioworklet":
          case "paintworklet":
          case "serviceworker":
          case "sharedworker":
          case "worker":
          case "script":
            if (a.querySelector(du(y)))
              return;
        }
        l = a.createElement("link"), _t(l, "link", e), at(l), a.head.appendChild(l);
      }
    }
  }
  function bR(e, n, a) {
    $n.S(e, n, a);
    var l = Wa;
    if (l && e) {
      var d = ir(l).hoistableStyles, y = ei(e);
      n = n || "default";
      var x = d.get(y);
      if (!x) {
        var q = { loading: 0, preload: null };
        if (x = l.querySelector(
          fu(y)
        ))
          q.loading = 5;
        else {
          e = p(
            { rel: "stylesheet", href: e, "data-precedence": n },
            a
          ), (a = nn.get(y)) && Af(e, a);
          var U = x = l.createElement("link");
          at(U), _t(U, "link", e), U._p = new Promise(function(te, oe) {
            U.onload = te, U.onerror = oe;
          }), U.addEventListener("load", function() {
            q.loading |= 1;
          }), U.addEventListener("error", function() {
            q.loading |= 2;
          }), q.loading |= 4, io(x, n, l);
        }
        x = {
          type: "stylesheet",
          instance: x,
          count: 1,
          state: q
        }, d.set(y, x);
      }
    }
  }
  function _R(e, n) {
    $n.X(e, n);
    var a = Wa;
    if (a && e) {
      var l = ir(a).hoistableScripts, d = ti(e), y = l.get(d);
      y || (y = a.querySelector(du(d)), y || (e = p({ src: e, async: !0 }, n), (n = nn.get(d)) && Tf(e, n), y = a.createElement("script"), at(y), _t(y, "link", e), a.head.appendChild(y)), y = {
        type: "script",
        instance: y,
        count: 1,
        state: null
      }, l.set(d, y));
    }
  }
  function xR(e, n) {
    $n.M(e, n);
    var a = Wa;
    if (a && e) {
      var l = ir(a).hoistableScripts, d = ti(e), y = l.get(d);
      y || (y = a.querySelector(du(d)), y || (e = p({ src: e, async: !0, type: "module" }, n), (n = nn.get(d)) && Tf(e, n), y = a.createElement("script"), at(y), _t(y, "link", e), a.head.appendChild(y)), y = {
        type: "script",
        instance: y,
        count: 1,
        state: null
      }, l.set(d, y));
    }
  }
  function Nb(e, n, a, l) {
    var d = (d = P.current) ? ao(d) : null;
    if (!d) throw Error(u(446));
    switch (e) {
      case "meta":
      case "title":
        return null;
      case "style":
        return typeof a.precedence == "string" && typeof a.href == "string" ? (n = ei(a.href), a = ir(
          d
        ).hoistableStyles, l = a.get(n), l || (l = {
          type: "style",
          instance: null,
          count: 0,
          state: null
        }, a.set(n, l)), l) : { type: "void", instance: null, count: 0, state: null };
      case "link":
        if (a.rel === "stylesheet" && typeof a.href == "string" && typeof a.precedence == "string") {
          e = ei(a.href);
          var y = ir(
            d
          ).hoistableStyles, x = y.get(e);
          if (x || (d = d.ownerDocument || d, x = {
            type: "stylesheet",
            instance: null,
            count: 0,
            state: { loading: 0, preload: null }
          }, y.set(e, x), (y = d.querySelector(
            fu(e)
          )) && !y._p && (x.instance = y, x.state.loading = 5), nn.has(e) || (a = {
            rel: "preload",
            as: "style",
            href: a.href,
            crossOrigin: a.crossOrigin,
            integrity: a.integrity,
            media: a.media,
            hrefLang: a.hrefLang,
            referrerPolicy: a.referrerPolicy
          }, nn.set(e, a), y || SR(
            d,
            e,
            a,
            x.state
          ))), n && l === null)
            throw Error(u(528, ""));
          return x;
        }
        if (n && l !== null)
          throw Error(u(529, ""));
        return null;
      case "script":
        return n = a.async, a = a.src, typeof a == "string" && n && typeof n != "function" && typeof n != "symbol" ? (n = ti(a), a = ir(
          d
        ).hoistableScripts, l = a.get(n), l || (l = {
          type: "script",
          instance: null,
          count: 0,
          state: null
        }, a.set(n, l)), l) : { type: "void", instance: null, count: 0, state: null };
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
  function Ob(e) {
    return p({}, e, {
      "data-precedence": e.precedence,
      precedence: null
    });
  }
  function SR(e, n, a, l) {
    e.querySelector('link[rel="preload"][as="style"][' + n + "]") ? l.loading = 1 : (n = e.createElement("link"), l.preload = n, n.addEventListener("load", function() {
      return l.loading |= 1;
    }), n.addEventListener("error", function() {
      return l.loading |= 2;
    }), _t(n, "link", a), at(n), e.head.appendChild(n));
  }
  function ti(e) {
    return '[src="' + Nt(e) + '"]';
  }
  function du(e) {
    return "script[async]" + e;
  }
  function zb(e, n, a) {
    if (n.count++, n.instance === null)
      switch (n.type) {
        case "style":
          var l = e.querySelector(
            'style[data-href~="' + Nt(a.href) + '"]'
          );
          if (l)
            return n.instance = l, at(l), l;
          var d = p({}, a, {
            "data-href": a.href,
            "data-precedence": a.precedence,
            href: null,
            precedence: null
          });
          return l = (e.ownerDocument || e).createElement(
            "style"
          ), at(l), _t(l, "style", d), io(l, a.precedence, e), n.instance = l;
        case "stylesheet":
          d = ei(a.href);
          var y = e.querySelector(
            fu(d)
          );
          if (y)
            return n.state.loading |= 4, n.instance = y, at(y), y;
          l = Ob(a), (d = nn.get(d)) && Af(l, d), y = (e.ownerDocument || e).createElement("link"), at(y);
          var x = y;
          return x._p = new Promise(function(q, U) {
            x.onload = q, x.onerror = U;
          }), _t(y, "link", l), n.state.loading |= 4, io(y, a.precedence, e), n.instance = y;
        case "script":
          return y = ti(a.src), (d = e.querySelector(
            du(y)
          )) ? (n.instance = d, at(d), d) : (l = a, (d = nn.get(y)) && (l = p({}, a), Tf(l, d)), e = e.ownerDocument || e, d = e.createElement("script"), at(d), _t(d, "link", l), e.head.appendChild(d), n.instance = d);
        case "void":
          return null;
        default:
          throw Error(u(443, n.type));
      }
    else
      n.type === "stylesheet" && (n.state.loading & 4) === 0 && (l = n.instance, n.state.loading |= 4, io(l, a.precedence, e));
    return n.instance;
  }
  function io(e, n, a) {
    for (var l = a.querySelectorAll(
      'link[rel="stylesheet"][data-precedence],style[data-precedence]'
    ), d = l.length ? l[l.length - 1] : null, y = d, x = 0; x < l.length; x++) {
      var q = l[x];
      if (q.dataset.precedence === n) y = q;
      else if (y !== d) break;
    }
    y ? y.parentNode.insertBefore(e, y.nextSibling) : (n = a.nodeType === 9 ? a.head : a, n.insertBefore(e, n.firstChild));
  }
  function Af(e, n) {
    e.crossOrigin == null && (e.crossOrigin = n.crossOrigin), e.referrerPolicy == null && (e.referrerPolicy = n.referrerPolicy), e.title == null && (e.title = n.title);
  }
  function Tf(e, n) {
    e.crossOrigin == null && (e.crossOrigin = n.crossOrigin), e.referrerPolicy == null && (e.referrerPolicy = n.referrerPolicy), e.integrity == null && (e.integrity = n.integrity);
  }
  var uo = null;
  function Db(e, n, a) {
    if (uo === null) {
      var l = /* @__PURE__ */ new Map(), d = uo = /* @__PURE__ */ new Map();
      d.set(a, l);
    } else
      d = uo, l = d.get(a), l || (l = /* @__PURE__ */ new Map(), d.set(a, l));
    if (l.has(e)) return l;
    for (l.set(e, null), a = a.getElementsByTagName(e), d = 0; d < a.length; d++) {
      var y = a[d];
      if (!(y[jr] || y[dt] || e === "link" && y.getAttribute("rel") === "stylesheet") && y.namespaceURI !== "http://www.w3.org/2000/svg") {
        var x = y.getAttribute(n) || "";
        x = e + x;
        var q = l.get(x);
        q ? q.push(y) : l.set(x, [y]);
      }
    }
    return l;
  }
  function Hb(e, n, a) {
    e = e.ownerDocument || e, e.head.insertBefore(
      a,
      n === "title" ? e.querySelector("head > title") : null
    );
  }
  function ER(e, n, a) {
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
  function Lb(e) {
    return !(e.type === "stylesheet" && (e.state.loading & 3) === 0);
  }
  function wR(e, n, a, l) {
    if (a.type === "stylesheet" && (typeof l.media != "string" || matchMedia(l.media).matches !== !1) && (a.state.loading & 4) === 0) {
      if (a.instance === null) {
        var d = ei(l.href), y = n.querySelector(
          fu(d)
        );
        if (y) {
          n = y._p, n !== null && typeof n == "object" && typeof n.then == "function" && (e.count++, e = lo.bind(e), n.then(e, e)), a.state.loading |= 4, a.instance = y, at(y);
          return;
        }
        y = n.ownerDocument || n, l = Ob(l), (d = nn.get(d)) && Af(l, d), y = y.createElement("link"), at(y);
        var x = y;
        x._p = new Promise(function(q, U) {
          x.onload = q, x.onerror = U;
        }), _t(y, "link", l), a.instance = y;
      }
      e.stylesheets === null && (e.stylesheets = /* @__PURE__ */ new Map()), e.stylesheets.set(a, n), (n = a.state.preload) && (a.state.loading & 3) === 0 && (e.count++, a = lo.bind(e), n.addEventListener("load", a), n.addEventListener("error", a));
    }
  }
  var Mf = 0;
  function AR(e, n) {
    return e.stylesheets && e.count === 0 && so(e, e.stylesheets), 0 < e.count || 0 < e.imgCount ? function(a) {
      var l = setTimeout(function() {
        if (e.stylesheets && so(e, e.stylesheets), e.unsuspend) {
          var y = e.unsuspend;
          e.unsuspend = null, y();
        }
      }, 6e4 + n);
      0 < e.imgBytes && Mf === 0 && (Mf = 62500 * iR());
      var d = setTimeout(
        function() {
          if (e.waitingForImages = !1, e.count === 0 && (e.stylesheets && so(e, e.stylesheets), e.unsuspend)) {
            var y = e.unsuspend;
            e.unsuspend = null, y();
          }
        },
        (e.imgBytes > Mf ? 50 : 800) + n
      );
      return e.unsuspend = a, function() {
        e.unsuspend = null, clearTimeout(l), clearTimeout(d);
      };
    } : null;
  }
  function lo() {
    if (this.count--, this.count === 0 && (this.imgCount === 0 || !this.waitingForImages)) {
      if (this.stylesheets) so(this, this.stylesheets);
      else if (this.unsuspend) {
        var e = this.unsuspend;
        this.unsuspend = null, e();
      }
    }
  }
  var oo = null;
  function so(e, n) {
    e.stylesheets = null, e.unsuspend !== null && (e.count++, oo = /* @__PURE__ */ new Map(), n.forEach(TR, e), oo = null, lo.call(e));
  }
  function TR(e, n) {
    if (!(n.state.loading & 4)) {
      var a = oo.get(e);
      if (a) var l = a.get(null);
      else {
        a = /* @__PURE__ */ new Map(), oo.set(e, a);
        for (var d = e.querySelectorAll(
          "link[data-precedence],style[data-precedence]"
        ), y = 0; y < d.length; y++) {
          var x = d[y];
          (x.nodeName === "LINK" || x.getAttribute("media") !== "not all") && (a.set(x.dataset.precedence, x), l = x);
        }
        l && a.set(null, l);
      }
      d = n.instance, x = d.getAttribute("data-precedence"), y = a.get(x) || l, y === l && a.set(null, d), a.set(x, d), this.count++, l = lo.bind(this), d.addEventListener("load", l), d.addEventListener("error", l), y ? y.parentNode.insertBefore(d, y.nextSibling) : (e = e.nodeType === 9 ? e.head : e, e.insertBefore(d, e.firstChild)), n.state.loading |= 4;
    }
  }
  var hu = {
    $$typeof: S,
    Provider: null,
    Consumer: null,
    _currentValue: Z,
    _currentValue2: Z,
    _threadCount: 0
  };
  function MR(e, n, a, l, d, y, x, q, U) {
    this.tag = 1, this.containerInfo = e, this.pingCache = this.current = this.pendingChildren = null, this.timeoutHandle = -1, this.callbackNode = this.next = this.pendingContext = this.context = this.cancelPendingCommit = null, this.callbackPriority = 0, this.expirationTimes = xi(-1), this.entangledLanes = this.shellSuspendCounter = this.errorRecoveryDisabledLanes = this.expiredLanes = this.warmLanes = this.pingedLanes = this.suspendedLanes = this.pendingLanes = 0, this.entanglements = xi(0), this.hiddenUpdates = xi(null), this.identifierPrefix = l, this.onUncaughtError = d, this.onCaughtError = y, this.onRecoverableError = x, this.pooledCache = null, this.pooledCacheLanes = 0, this.formState = U, this.incompleteTransitions = /* @__PURE__ */ new Map();
  }
  function Bb(e, n, a, l, d, y, x, q, U, te, oe, fe) {
    return e = new MR(
      e,
      n,
      a,
      x,
      U,
      te,
      oe,
      fe,
      q
    ), n = 1, y === !0 && (n |= 24), y = Vt(3, null, null, n), e.current = y, y.stateNode = e, n = ic(), n.refCount++, e.pooledCache = n, n.refCount++, y.memoizedState = {
      element: l,
      isDehydrated: a,
      cache: n
    }, sc(y), e;
  }
  function jb(e) {
    return e ? (e = Oa, e) : Oa;
  }
  function Ub(e, n, a, l, d, y) {
    d = jb(d), l.context === null ? l.context = d : l.pendingContext = d, l = gr(n), l.payload = { element: a }, y = y === void 0 ? null : y, y !== null && (l.callback = y), a = vr(e, l, n), a !== null && (Bt(a, e, n), Ii(a, e, n));
  }
  function Gb(e, n) {
    if (e = e.memoizedState, e !== null && e.dehydrated !== null) {
      var a = e.retryLane;
      e.retryLane = a !== 0 && a < n ? a : n;
    }
  }
  function qf(e, n) {
    Gb(e, n), (e = e.alternate) && Gb(e, n);
  }
  function Vb(e) {
    if (e.tag === 13 || e.tag === 31) {
      var n = kr(e, 67108864);
      n !== null && Bt(n, e, 67108864), qf(e, 67108864);
    }
  }
  function Yb(e) {
    if (e.tag === 13 || e.tag === 31) {
      var n = Qt();
      n = Si(n);
      var a = kr(e, n);
      a !== null && Bt(a, e, n), qf(e, n);
    }
  }
  var co = !0;
  function qR(e, n, a, l) {
    var d = N.T;
    N.T = null;
    var y = j.p;
    try {
      j.p = 2, Cf(e, n, a, l);
    } finally {
      j.p = y, N.T = d;
    }
  }
  function CR(e, n, a, l) {
    var d = N.T;
    N.T = null;
    var y = j.p;
    try {
      j.p = 8, Cf(e, n, a, l);
    } finally {
      j.p = y, N.T = d;
    }
  }
  function Cf(e, n, a, l) {
    if (co) {
      var d = Rf(l);
      if (d === null)
        vf(
          e,
          n,
          l,
          fo,
          a
        ), Xb(e, l);
      else if (NR(
        d,
        e,
        n,
        a,
        l
      ))
        l.stopPropagation();
      else if (Xb(e, l), n & 4 && -1 < RR.indexOf(e)) {
        for (; d !== null; ) {
          var y = rr(d);
          if (y !== null)
            switch (y.tag) {
              case 3:
                if (y = y.stateNode, y.current.memoizedState.isDehydrated) {
                  var x = Rn(y.pendingLanes);
                  if (x !== 0) {
                    var q = y;
                    for (q.pendingLanes |= 2, q.entangledLanes |= 2; x; ) {
                      var U = 1 << 31 - Mt(x);
                      q.entanglements[1] |= U, x &= ~U;
                    }
                    En(y), (Le & 6) === 0 && (Zl = ke() + 500, lu(0));
                  }
                }
                break;
              case 31:
              case 13:
                q = kr(y, 2), q !== null && Bt(q, y, 2), $l(), qf(y, 2);
            }
          if (y = Rf(l), y === null && vf(
            e,
            n,
            l,
            fo,
            a
          ), y === d) break;
          d = y;
        }
        d !== null && l.stopPropagation();
      } else
        vf(
          e,
          n,
          l,
          null,
          a
        );
    }
  }
  function Rf(e) {
    return e = Ns(e), Nf(e);
  }
  var fo = null;
  function Nf(e) {
    if (fo = null, e = nr(e), e !== null) {
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
    return fo = e, null;
  }
  function kb(e) {
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
          case Ct:
            return 2;
          case Tt:
            return 8;
          case ft:
          case tr:
            return 32;
          case Cn:
            return 268435456;
          default:
            return 32;
        }
      default:
        return 32;
    }
  }
  var Of = !1, Tr = null, Mr = null, qr = null, gu = /* @__PURE__ */ new Map(), vu = /* @__PURE__ */ new Map(), Cr = [], RR = "mousedown mouseup touchcancel touchend touchstart auxclick dblclick pointercancel pointerdown pointerup dragend dragstart drop compositionend compositionstart keydown keypress keyup input textInput copy cut paste click change contextmenu reset".split(
    " "
  );
  function Xb(e, n) {
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
        qr = null;
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
  function yu(e, n, a, l, d, y) {
    return e === null || e.nativeEvent !== y ? (e = {
      blockedOn: n,
      domEventName: a,
      eventSystemFlags: l,
      nativeEvent: y,
      targetContainers: [d]
    }, n !== null && (n = rr(n), n !== null && Vb(n)), e) : (e.eventSystemFlags |= l, n = e.targetContainers, d !== null && n.indexOf(d) === -1 && n.push(d), e);
  }
  function NR(e, n, a, l, d) {
    switch (n) {
      case "focusin":
        return Tr = yu(
          Tr,
          e,
          n,
          a,
          l,
          d
        ), !0;
      case "dragenter":
        return Mr = yu(
          Mr,
          e,
          n,
          a,
          l,
          d
        ), !0;
      case "mouseover":
        return qr = yu(
          qr,
          e,
          n,
          a,
          l,
          d
        ), !0;
      case "pointerover":
        var y = d.pointerId;
        return gu.set(
          y,
          yu(
            gu.get(y) || null,
            e,
            n,
            a,
            l,
            d
          )
        ), !0;
      case "gotpointercapture":
        return y = d.pointerId, vu.set(
          y,
          yu(
            vu.get(y) || null,
            e,
            n,
            a,
            l,
            d
          )
        ), !0;
    }
    return !1;
  }
  function Ib(e) {
    var n = nr(e.target);
    if (n !== null) {
      var a = s(n);
      if (a !== null) {
        if (n = a.tag, n === 13) {
          if (n = c(a), n !== null) {
            e.blockedOn = n, el(e.priority, function() {
              Yb(a);
            });
            return;
          }
        } else if (n === 31) {
          if (n = f(a), n !== null) {
            e.blockedOn = n, el(e.priority, function() {
              Yb(a);
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
  function ho(e) {
    if (e.blockedOn !== null) return !1;
    for (var n = e.targetContainers; 0 < n.length; ) {
      var a = Rf(e.nativeEvent);
      if (a === null) {
        a = e.nativeEvent;
        var l = new a.constructor(
          a.type,
          a
        );
        Rs = l, a.target.dispatchEvent(l), Rs = null;
      } else
        return n = rr(a), n !== null && Vb(n), e.blockedOn = a, !1;
      n.shift();
    }
    return !0;
  }
  function Qb(e, n, a) {
    ho(e) && a.delete(n);
  }
  function OR() {
    Of = !1, Tr !== null && ho(Tr) && (Tr = null), Mr !== null && ho(Mr) && (Mr = null), qr !== null && ho(qr) && (qr = null), gu.forEach(Qb), vu.forEach(Qb);
  }
  function go(e, n) {
    e.blockedOn === n && (e.blockedOn = null, Of || (Of = !0, t.unstable_scheduleCallback(
      t.unstable_NormalPriority,
      OR
    )));
  }
  var vo = null;
  function Zb(e) {
    vo !== e && (vo = e, t.unstable_scheduleCallback(
      t.unstable_NormalPriority,
      function() {
        vo === e && (vo = null);
        for (var n = 0; n < e.length; n += 3) {
          var a = e[n], l = e[n + 1], d = e[n + 2];
          if (typeof l != "function") {
            if (Nf(l || a) === null)
              continue;
            break;
          }
          var y = rr(a);
          y !== null && (e.splice(n, 3), n -= 3, Cc(
            y,
            {
              pending: !0,
              data: d,
              method: a.method,
              action: l
            },
            l,
            d
          ));
        }
      }
    ));
  }
  function ni(e) {
    function n(U) {
      return go(U, e);
    }
    Tr !== null && go(Tr, e), Mr !== null && go(Mr, e), qr !== null && go(qr, e), gu.forEach(n), vu.forEach(n);
    for (var a = 0; a < Cr.length; a++) {
      var l = Cr[a];
      l.blockedOn === e && (l.blockedOn = null);
    }
    for (; 0 < Cr.length && (a = Cr[0], a.blockedOn === null); )
      Ib(a), a.blockedOn === null && Cr.shift();
    if (a = (e.ownerDocument || e).$$reactFormReplay, a != null)
      for (l = 0; l < a.length; l += 3) {
        var d = a[l], y = a[l + 1], x = d[wt] || null;
        if (typeof y == "function")
          x || Zb(a);
        else if (x) {
          var q = null;
          if (y && y.hasAttribute("formAction")) {
            if (d = y, x = y[wt] || null)
              q = x.formAction;
            else if (Nf(d) !== null) continue;
          } else q = x.action;
          typeof q == "function" ? a[l + 1] = q : (a.splice(l, 3), l -= 3), Zb(a);
        }
      }
  }
  function Kb() {
    function e(y) {
      y.canIntercept && y.info === "react-transition" && y.intercept({
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
      d !== null && (d(), d = null), l || setTimeout(a, 20);
    }
    function a() {
      if (!l && !navigation.transition) {
        var y = navigation.currentEntry;
        y && y.url != null && navigation.navigate(y.url, {
          state: y.getState(),
          info: "react-transition",
          history: "replace"
        });
      }
    }
    if (typeof navigation == "object") {
      var l = !1, d = null;
      return navigation.addEventListener("navigate", e), navigation.addEventListener("navigatesuccess", n), navigation.addEventListener("navigateerror", n), setTimeout(a, 100), function() {
        l = !0, navigation.removeEventListener("navigate", e), navigation.removeEventListener("navigatesuccess", n), navigation.removeEventListener("navigateerror", n), d !== null && (d(), d = null);
      };
    }
  }
  function zf(e) {
    this._internalRoot = e;
  }
  yo.prototype.render = zf.prototype.render = function(e) {
    var n = this._internalRoot;
    if (n === null) throw Error(u(409));
    var a = n.current, l = Qt();
    Ub(a, l, e, n, null, null);
  }, yo.prototype.unmount = zf.prototype.unmount = function() {
    var e = this._internalRoot;
    if (e !== null) {
      this._internalRoot = null;
      var n = e.containerInfo;
      Ub(e.current, 2, null, e, null, null), $l(), n[Nn] = null;
    }
  };
  function yo(e) {
    this._internalRoot = e;
  }
  yo.prototype.unstable_scheduleHydration = function(e) {
    if (e) {
      var n = Wu();
      e = { blockedOn: null, target: e, priority: n };
      for (var a = 0; a < Cr.length && n !== 0 && n < Cr[a].priority; a++) ;
      Cr.splice(a, 0, e), a === 0 && Ib(e);
    }
  };
  var $b = r.version;
  if ($b !== "19.2.8")
    throw Error(
      u(
        527,
        $b,
        "19.2.8"
      )
    );
  j.findDOMNode = function(e) {
    var n = e._reactInternals;
    if (n === void 0)
      throw typeof e.render == "function" ? Error(u(188)) : (e = Object.keys(e).join(","), Error(u(268, e)));
    return e = h(n), e = e !== null ? v(e) : null, e = e === null ? null : e.stateNode, e;
  };
  var zR = {
    bundleType: 0,
    version: "19.2.8",
    rendererPackageName: "react-dom",
    currentDispatcherRef: N,
    reconcilerVersion: "19.2.8"
  };
  if (typeof __REACT_DEVTOOLS_GLOBAL_HOOK__ < "u") {
    var po = __REACT_DEVTOOLS_GLOBAL_HOOK__;
    if (!po.isDisabled && po.supportsFiber)
      try {
        Et = po.inject(
          zR
        ), Fe = po;
      } catch {
      }
  }
  return mu.createRoot = function(e, n) {
    if (!o(e)) throw Error(u(299));
    var a = !1, l = "", d = n1, y = r1, x = a1;
    return n != null && (n.unstable_strictMode === !0 && (a = !0), n.identifierPrefix !== void 0 && (l = n.identifierPrefix), n.onUncaughtError !== void 0 && (d = n.onUncaughtError), n.onCaughtError !== void 0 && (y = n.onCaughtError), n.onRecoverableError !== void 0 && (x = n.onRecoverableError)), n = Bb(
      e,
      1,
      !1,
      null,
      null,
      a,
      l,
      null,
      d,
      y,
      x,
      Kb
    ), e[Nn] = n.current, gf(e), new zf(n);
  }, mu.hydrateRoot = function(e, n, a) {
    if (!o(e)) throw Error(u(299));
    var l = !1, d = "", y = n1, x = r1, q = a1, U = null;
    return a != null && (a.unstable_strictMode === !0 && (l = !0), a.identifierPrefix !== void 0 && (d = a.identifierPrefix), a.onUncaughtError !== void 0 && (y = a.onUncaughtError), a.onCaughtError !== void 0 && (x = a.onCaughtError), a.onRecoverableError !== void 0 && (q = a.onRecoverableError), a.formState !== void 0 && (U = a.formState)), n = Bb(
      e,
      1,
      !0,
      n,
      a ?? null,
      l,
      d,
      U,
      y,
      x,
      q,
      Kb
    ), n.context = jb(null), a = n.current, l = Qt(), l = Si(l), d = gr(l), d.callback = null, vr(a, d, l), a = l, n.current.lanes = a, Br(n, a), En(n), e[Nn] = n.current, gf(e), new yo(n);
  }, mu.version = "19.2.8", mu;
}
var u_;
function ZR() {
  if (u_) return Bf.exports;
  u_ = 1;
  function t() {
    if (!(typeof __REACT_DEVTOOLS_GLOBAL_HOOK__ > "u" || typeof __REACT_DEVTOOLS_GLOBAL_HOOK__.checkDCE != "function"))
      try {
        __REACT_DEVTOOLS_GLOBAL_HOOK__.checkDCE(t);
      } catch (r) {
        console.error(r);
      }
  }
  return t(), Bf.exports = QR(), Bf.exports;
}
var KR = ZR();
function Op(t) {
  throw new Error('Could not dynamically require "' + t + '". Please configure the dynamicRequireTargets or/and ignoreDynamicRequires option of @rollup/plugin-commonjs appropriately for this require call to work.');
}
var Vf, l_;
function $R() {
  if (l_) return Vf;
  l_ = 1;
  function t() {
    this.__data__ = [], this.size = 0;
  }
  return Vf = t, Vf;
}
var Yf, o_;
function vi() {
  if (o_) return Yf;
  o_ = 1;
  function t(r, i) {
    return r === i || r !== r && i !== i;
  }
  return Yf = t, Yf;
}
var kf, s_;
function Io() {
  if (s_) return kf;
  s_ = 1;
  var t = vi();
  function r(i, u) {
    for (var o = i.length; o--; )
      if (t(i[o][0], u))
        return o;
    return -1;
  }
  return kf = r, kf;
}
var Xf, c_;
function FR() {
  if (c_) return Xf;
  c_ = 1;
  var t = Io(), r = Array.prototype, i = r.splice;
  function u(o) {
    var s = this.__data__, c = t(s, o);
    if (c < 0)
      return !1;
    var f = s.length - 1;
    return c == f ? s.pop() : i.call(s, c, 1), --this.size, !0;
  }
  return Xf = u, Xf;
}
var If, f_;
function JR() {
  if (f_) return If;
  f_ = 1;
  var t = Io();
  function r(i) {
    var u = this.__data__, o = t(u, i);
    return o < 0 ? void 0 : u[o][1];
  }
  return If = r, If;
}
var Qf, d_;
function PR() {
  if (d_) return Qf;
  d_ = 1;
  var t = Io();
  function r(i) {
    return t(this.__data__, i) > -1;
  }
  return Qf = r, Qf;
}
var Zf, h_;
function WR() {
  if (h_) return Zf;
  h_ = 1;
  var t = Io();
  function r(i, u) {
    var o = this.__data__, s = t(o, i);
    return s < 0 ? (++this.size, o.push([i, u])) : o[s][1] = u, this;
  }
  return Zf = r, Zf;
}
var Kf, g_;
function Qo() {
  if (g_) return Kf;
  g_ = 1;
  var t = $R(), r = FR(), i = JR(), u = PR(), o = WR();
  function s(c) {
    var f = -1, g = c == null ? 0 : c.length;
    for (this.clear(); ++f < g; ) {
      var h = c[f];
      this.set(h[0], h[1]);
    }
  }
  return s.prototype.clear = t, s.prototype.delete = r, s.prototype.get = i, s.prototype.has = u, s.prototype.set = o, Kf = s, Kf;
}
var $f, v_;
function eN() {
  if (v_) return $f;
  v_ = 1;
  var t = Qo();
  function r() {
    this.__data__ = new t(), this.size = 0;
  }
  return $f = r, $f;
}
var Ff, y_;
function tN() {
  if (y_) return Ff;
  y_ = 1;
  function t(r) {
    var i = this.__data__, u = i.delete(r);
    return this.size = i.size, u;
  }
  return Ff = t, Ff;
}
var Jf, p_;
function nN() {
  if (p_) return Jf;
  p_ = 1;
  function t(r) {
    return this.__data__.get(r);
  }
  return Jf = t, Jf;
}
var Pf, m_;
function rN() {
  if (m_) return Pf;
  m_ = 1;
  function t(r) {
    return this.__data__.has(r);
  }
  return Pf = t, Pf;
}
var Wf, b_;
function GA() {
  if (b_) return Wf;
  b_ = 1;
  var t = typeof mo == "object" && mo && mo.Object === Object && mo;
  return Wf = t, Wf;
}
var ed, __;
function yn() {
  if (__) return ed;
  __ = 1;
  var t = GA(), r = typeof self == "object" && self && self.Object === Object && self, i = t || r || Function("return this")();
  return ed = i, ed;
}
var td, x_;
function yi() {
  if (x_) return td;
  x_ = 1;
  var t = yn(), r = t.Symbol;
  return td = r, td;
}
var nd, S_;
function aN() {
  if (S_) return nd;
  S_ = 1;
  var t = yi(), r = Object.prototype, i = r.hasOwnProperty, u = r.toString, o = t ? t.toStringTag : void 0;
  function s(c) {
    var f = i.call(c, o), g = c[o];
    try {
      c[o] = void 0;
      var h = !0;
    } catch {
    }
    var v = u.call(c);
    return h && (f ? c[o] = g : delete c[o]), v;
  }
  return nd = s, nd;
}
var rd, E_;
function iN() {
  if (E_) return rd;
  E_ = 1;
  var t = Object.prototype, r = t.toString;
  function i(u) {
    return r.call(u);
  }
  return rd = i, rd;
}
var ad, w_;
function fa() {
  if (w_) return ad;
  w_ = 1;
  var t = yi(), r = aN(), i = iN(), u = "[object Null]", o = "[object Undefined]", s = t ? t.toStringTag : void 0;
  function c(f) {
    return f == null ? f === void 0 ? o : u : s && s in Object(f) ? r(f) : i(f);
  }
  return ad = c, ad;
}
var id, A_;
function rn() {
  if (A_) return id;
  A_ = 1;
  function t(r) {
    var i = typeof r;
    return r != null && (i == "object" || i == "function");
  }
  return id = t, id;
}
var ud, T_;
function ju() {
  if (T_) return ud;
  T_ = 1;
  var t = fa(), r = rn(), i = "[object AsyncFunction]", u = "[object Function]", o = "[object GeneratorFunction]", s = "[object Proxy]";
  function c(f) {
    if (!r(f))
      return !1;
    var g = t(f);
    return g == u || g == o || g == i || g == s;
  }
  return ud = c, ud;
}
var ld, M_;
function uN() {
  if (M_) return ld;
  M_ = 1;
  var t = yn(), r = t["__core-js_shared__"];
  return ld = r, ld;
}
var od, q_;
function lN() {
  if (q_) return od;
  q_ = 1;
  var t = uN(), r = (function() {
    var u = /[^.]+$/.exec(t && t.keys && t.keys.IE_PROTO || "");
    return u ? "Symbol(src)_1." + u : "";
  })();
  function i(u) {
    return !!r && r in u;
  }
  return od = i, od;
}
var sd, C_;
function VA() {
  if (C_) return sd;
  C_ = 1;
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
  return sd = i, sd;
}
var cd, R_;
function oN() {
  if (R_) return cd;
  R_ = 1;
  var t = ju(), r = lN(), i = rn(), u = VA(), o = /[\\^$.*+?()[\]{}|]/g, s = /^\[object .+?Constructor\]$/, c = Function.prototype, f = Object.prototype, g = c.toString, h = f.hasOwnProperty, v = RegExp(
    "^" + g.call(h).replace(o, "\\$&").replace(/hasOwnProperty|(function).*?(?=\\\()| for .+?(?=\\\])/g, "$1.*?") + "$"
  );
  function p(m) {
    if (!i(m) || r(m))
      return !1;
    var b = t(m) ? v : s;
    return b.test(u(m));
  }
  return cd = p, cd;
}
var fd, N_;
function sN() {
  if (N_) return fd;
  N_ = 1;
  function t(r, i) {
    return r == null ? void 0 : r[i];
  }
  return fd = t, fd;
}
var dd, O_;
function da() {
  if (O_) return dd;
  O_ = 1;
  var t = oN(), r = sN();
  function i(u, o) {
    var s = r(u, o);
    return t(s) ? s : void 0;
  }
  return dd = i, dd;
}
var hd, z_;
function zp() {
  if (z_) return hd;
  z_ = 1;
  var t = da(), r = yn(), i = t(r, "Map");
  return hd = i, hd;
}
var gd, D_;
function Zo() {
  if (D_) return gd;
  D_ = 1;
  var t = da(), r = t(Object, "create");
  return gd = r, gd;
}
var vd, H_;
function cN() {
  if (H_) return vd;
  H_ = 1;
  var t = Zo();
  function r() {
    this.__data__ = t ? t(null) : {}, this.size = 0;
  }
  return vd = r, vd;
}
var yd, L_;
function fN() {
  if (L_) return yd;
  L_ = 1;
  function t(r) {
    var i = this.has(r) && delete this.__data__[r];
    return this.size -= i ? 1 : 0, i;
  }
  return yd = t, yd;
}
var pd, B_;
function dN() {
  if (B_) return pd;
  B_ = 1;
  var t = Zo(), r = "__lodash_hash_undefined__", i = Object.prototype, u = i.hasOwnProperty;
  function o(s) {
    var c = this.__data__;
    if (t) {
      var f = c[s];
      return f === r ? void 0 : f;
    }
    return u.call(c, s) ? c[s] : void 0;
  }
  return pd = o, pd;
}
var md, j_;
function hN() {
  if (j_) return md;
  j_ = 1;
  var t = Zo(), r = Object.prototype, i = r.hasOwnProperty;
  function u(o) {
    var s = this.__data__;
    return t ? s[o] !== void 0 : i.call(s, o);
  }
  return md = u, md;
}
var bd, U_;
function gN() {
  if (U_) return bd;
  U_ = 1;
  var t = Zo(), r = "__lodash_hash_undefined__";
  function i(u, o) {
    var s = this.__data__;
    return this.size += this.has(u) ? 0 : 1, s[u] = t && o === void 0 ? r : o, this;
  }
  return bd = i, bd;
}
var _d, G_;
function vN() {
  if (G_) return _d;
  G_ = 1;
  var t = cN(), r = fN(), i = dN(), u = hN(), o = gN();
  function s(c) {
    var f = -1, g = c == null ? 0 : c.length;
    for (this.clear(); ++f < g; ) {
      var h = c[f];
      this.set(h[0], h[1]);
    }
  }
  return s.prototype.clear = t, s.prototype.delete = r, s.prototype.get = i, s.prototype.has = u, s.prototype.set = o, _d = s, _d;
}
var xd, V_;
function yN() {
  if (V_) return xd;
  V_ = 1;
  var t = vN(), r = Qo(), i = zp();
  function u() {
    this.size = 0, this.__data__ = {
      hash: new t(),
      map: new (i || r)(),
      string: new t()
    };
  }
  return xd = u, xd;
}
var Sd, Y_;
function pN() {
  if (Y_) return Sd;
  Y_ = 1;
  function t(r) {
    var i = typeof r;
    return i == "string" || i == "number" || i == "symbol" || i == "boolean" ? r !== "__proto__" : r === null;
  }
  return Sd = t, Sd;
}
var Ed, k_;
function Ko() {
  if (k_) return Ed;
  k_ = 1;
  var t = pN();
  function r(i, u) {
    var o = i.__data__;
    return t(u) ? o[typeof u == "string" ? "string" : "hash"] : o.map;
  }
  return Ed = r, Ed;
}
var wd, X_;
function mN() {
  if (X_) return wd;
  X_ = 1;
  var t = Ko();
  function r(i) {
    var u = t(this, i).delete(i);
    return this.size -= u ? 1 : 0, u;
  }
  return wd = r, wd;
}
var Ad, I_;
function bN() {
  if (I_) return Ad;
  I_ = 1;
  var t = Ko();
  function r(i) {
    return t(this, i).get(i);
  }
  return Ad = r, Ad;
}
var Td, Q_;
function _N() {
  if (Q_) return Td;
  Q_ = 1;
  var t = Ko();
  function r(i) {
    return t(this, i).has(i);
  }
  return Td = r, Td;
}
var Md, Z_;
function xN() {
  if (Z_) return Md;
  Z_ = 1;
  var t = Ko();
  function r(i, u) {
    var o = t(this, i), s = o.size;
    return o.set(i, u), this.size += o.size == s ? 0 : 1, this;
  }
  return Md = r, Md;
}
var qd, K_;
function Dp() {
  if (K_) return qd;
  K_ = 1;
  var t = yN(), r = mN(), i = bN(), u = _N(), o = xN();
  function s(c) {
    var f = -1, g = c == null ? 0 : c.length;
    for (this.clear(); ++f < g; ) {
      var h = c[f];
      this.set(h[0], h[1]);
    }
  }
  return s.prototype.clear = t, s.prototype.delete = r, s.prototype.get = i, s.prototype.has = u, s.prototype.set = o, qd = s, qd;
}
var Cd, $_;
function SN() {
  if ($_) return Cd;
  $_ = 1;
  var t = Qo(), r = zp(), i = Dp(), u = 200;
  function o(s, c) {
    var f = this.__data__;
    if (f instanceof t) {
      var g = f.__data__;
      if (!r || g.length < u - 1)
        return g.push([s, c]), this.size = ++f.size, this;
      f = this.__data__ = new i(g);
    }
    return f.set(s, c), this.size = f.size, this;
  }
  return Cd = o, Cd;
}
var Rd, F_;
function $o() {
  if (F_) return Rd;
  F_ = 1;
  var t = Qo(), r = eN(), i = tN(), u = nN(), o = rN(), s = SN();
  function c(f) {
    var g = this.__data__ = new t(f);
    this.size = g.size;
  }
  return c.prototype.clear = r, c.prototype.delete = i, c.prototype.get = u, c.prototype.has = o, c.prototype.set = s, Rd = c, Rd;
}
var Nd, J_;
function Hp() {
  if (J_) return Nd;
  J_ = 1;
  function t(r, i) {
    for (var u = -1, o = r == null ? 0 : r.length; ++u < o && i(r[u], u, r) !== !1; )
      ;
    return r;
  }
  return Nd = t, Nd;
}
var Od, P_;
function YA() {
  if (P_) return Od;
  P_ = 1;
  var t = da(), r = (function() {
    try {
      var i = t(Object, "defineProperty");
      return i({}, "", {}), i;
    } catch {
    }
  })();
  return Od = r, Od;
}
var zd, W_;
function Fo() {
  if (W_) return zd;
  W_ = 1;
  var t = YA();
  function r(i, u, o) {
    u == "__proto__" && t ? t(i, u, {
      configurable: !0,
      enumerable: !0,
      value: o,
      writable: !0
    }) : i[u] = o;
  }
  return zd = r, zd;
}
var Dd, ex;
function Jo() {
  if (ex) return Dd;
  ex = 1;
  var t = Fo(), r = vi(), i = Object.prototype, u = i.hasOwnProperty;
  function o(s, c, f) {
    var g = s[c];
    (!(u.call(s, c) && r(g, f)) || f === void 0 && !(c in s)) && t(s, c, f);
  }
  return Dd = o, Dd;
}
var Hd, tx;
function Uu() {
  if (tx) return Hd;
  tx = 1;
  var t = Jo(), r = Fo();
  function i(u, o, s, c) {
    var f = !s;
    s || (s = {});
    for (var g = -1, h = o.length; ++g < h; ) {
      var v = o[g], p = c ? c(s[v], u[v], v, s, u) : void 0;
      p === void 0 && (p = u[v]), f ? r(s, v, p) : t(s, v, p);
    }
    return s;
  }
  return Hd = i, Hd;
}
var Ld, nx;
function EN() {
  if (nx) return Ld;
  nx = 1;
  function t(r, i) {
    for (var u = -1, o = Array(r); ++u < r; )
      o[u] = i(u);
    return o;
  }
  return Ld = t, Ld;
}
var Bd, rx;
function Tn() {
  if (rx) return Bd;
  rx = 1;
  function t(r) {
    return r != null && typeof r == "object";
  }
  return Bd = t, Bd;
}
var jd, ax;
function wN() {
  if (ax) return jd;
  ax = 1;
  var t = fa(), r = Tn(), i = "[object Arguments]";
  function u(o) {
    return r(o) && t(o) == i;
  }
  return jd = u, jd;
}
var Ud, ix;
function Gu() {
  if (ix) return Ud;
  ix = 1;
  var t = wN(), r = Tn(), i = Object.prototype, u = i.hasOwnProperty, o = i.propertyIsEnumerable, s = t(/* @__PURE__ */ (function() {
    return arguments;
  })()) ? t : function(c) {
    return r(c) && u.call(c, "callee") && !o.call(c, "callee");
  };
  return Ud = s, Ud;
}
var Gd, ux;
function st() {
  if (ux) return Gd;
  ux = 1;
  var t = Array.isArray;
  return Gd = t, Gd;
}
var xu = { exports: {} }, Vd, lx;
function AN() {
  if (lx) return Vd;
  lx = 1;
  function t() {
    return !1;
  }
  return Vd = t, Vd;
}
xu.exports;
var ox;
function pi() {
  return ox || (ox = 1, (function(t, r) {
    var i = yn(), u = AN(), o = r && !r.nodeType && r, s = o && !0 && t && !t.nodeType && t, c = s && s.exports === o, f = c ? i.Buffer : void 0, g = f ? f.isBuffer : void 0, h = g || u;
    t.exports = h;
  })(xu, xu.exports)), xu.exports;
}
var Yd, sx;
function Po() {
  if (sx) return Yd;
  sx = 1;
  var t = 9007199254740991, r = /^(?:0|[1-9]\d*)$/;
  function i(u, o) {
    var s = typeof u;
    return o = o ?? t, !!o && (s == "number" || s != "symbol" && r.test(u)) && u > -1 && u % 1 == 0 && u < o;
  }
  return Yd = i, Yd;
}
var kd, cx;
function Lp() {
  if (cx) return kd;
  cx = 1;
  var t = 9007199254740991;
  function r(i) {
    return typeof i == "number" && i > -1 && i % 1 == 0 && i <= t;
  }
  return kd = r, kd;
}
var Xd, fx;
function TN() {
  if (fx) return Xd;
  fx = 1;
  var t = fa(), r = Lp(), i = Tn(), u = "[object Arguments]", o = "[object Array]", s = "[object Boolean]", c = "[object Date]", f = "[object Error]", g = "[object Function]", h = "[object Map]", v = "[object Number]", p = "[object Object]", m = "[object RegExp]", b = "[object Set]", _ = "[object String]", A = "[object WeakMap]", w = "[object ArrayBuffer]", E = "[object DataView]", M = "[object Float32Array]", S = "[object Float64Array]", T = "[object Int8Array]", O = "[object Int16Array]", C = "[object Int32Array]", R = "[object Uint8Array]", H = "[object Uint8ClampedArray]", B = "[object Uint16Array]", X = "[object Uint32Array]", Y = {};
  Y[M] = Y[S] = Y[T] = Y[O] = Y[C] = Y[R] = Y[H] = Y[B] = Y[X] = !0, Y[u] = Y[o] = Y[w] = Y[s] = Y[E] = Y[c] = Y[f] = Y[g] = Y[h] = Y[v] = Y[p] = Y[m] = Y[b] = Y[_] = Y[A] = !1;
  function F(K) {
    return i(K) && r(K.length) && !!Y[t(K)];
  }
  return Xd = F, Xd;
}
var Id, dx;
function Wo() {
  if (dx) return Id;
  dx = 1;
  function t(r) {
    return function(i) {
      return r(i);
    };
  }
  return Id = t, Id;
}
var Su = { exports: {} };
Su.exports;
var hx;
function Bp() {
  return hx || (hx = 1, (function(t, r) {
    var i = GA(), u = r && !r.nodeType && r, o = u && !0 && t && !t.nodeType && t, s = o && o.exports === u, c = s && i.process, f = (function() {
      try {
        var g = o && o.require && o.require("util").types;
        return g || c && c.binding && c.binding("util");
      } catch {
      }
    })();
    t.exports = f;
  })(Su, Su.exports)), Su.exports;
}
var Qd, gx;
function Vu() {
  if (gx) return Qd;
  gx = 1;
  var t = TN(), r = Wo(), i = Bp(), u = i && i.isTypedArray, o = u ? r(u) : t;
  return Qd = o, Qd;
}
var Zd, vx;
function kA() {
  if (vx) return Zd;
  vx = 1;
  var t = EN(), r = Gu(), i = st(), u = pi(), o = Po(), s = Vu(), c = Object.prototype, f = c.hasOwnProperty;
  function g(h, v) {
    var p = i(h), m = !p && r(h), b = !p && !m && u(h), _ = !p && !m && !b && s(h), A = p || m || b || _, w = A ? t(h.length, String) : [], E = w.length;
    for (var M in h)
      (v || f.call(h, M)) && !(A && // Safari 9 has enumerable `arguments.length` in strict mode.
      (M == "length" || // Node.js 0.10 has enumerable non-index properties on buffers.
      b && (M == "offset" || M == "parent") || // PhantomJS 2 has enumerable non-index properties on typed arrays.
      _ && (M == "buffer" || M == "byteLength" || M == "byteOffset") || // Skip index properties.
      o(M, E))) && w.push(M);
    return w;
  }
  return Zd = g, Zd;
}
var Kd, yx;
function es() {
  if (yx) return Kd;
  yx = 1;
  var t = Object.prototype;
  function r(i) {
    var u = i && i.constructor, o = typeof u == "function" && u.prototype || t;
    return i === o;
  }
  return Kd = r, Kd;
}
var $d, px;
function XA() {
  if (px) return $d;
  px = 1;
  function t(r, i) {
    return function(u) {
      return r(i(u));
    };
  }
  return $d = t, $d;
}
var Fd, mx;
function MN() {
  if (mx) return Fd;
  mx = 1;
  var t = XA(), r = t(Object.keys, Object);
  return Fd = r, Fd;
}
var Jd, bx;
function jp() {
  if (bx) return Jd;
  bx = 1;
  var t = es(), r = MN(), i = Object.prototype, u = i.hasOwnProperty;
  function o(s) {
    if (!t(s))
      return r(s);
    var c = [];
    for (var f in Object(s))
      u.call(s, f) && f != "constructor" && c.push(f);
    return c;
  }
  return Jd = o, Jd;
}
var Pd, _x;
function Wn() {
  if (_x) return Pd;
  _x = 1;
  var t = ju(), r = Lp();
  function i(u) {
    return u != null && r(u.length) && !t(u);
  }
  return Pd = i, Pd;
}
var Wd, xx;
function Hr() {
  if (xx) return Wd;
  xx = 1;
  var t = kA(), r = jp(), i = Wn();
  function u(o) {
    return i(o) ? t(o) : r(o);
  }
  return Wd = u, Wd;
}
var eh, Sx;
function qN() {
  if (Sx) return eh;
  Sx = 1;
  var t = Uu(), r = Hr();
  function i(u, o) {
    return u && t(o, r(o), u);
  }
  return eh = i, eh;
}
var th, Ex;
function CN() {
  if (Ex) return th;
  Ex = 1;
  function t(r) {
    var i = [];
    if (r != null)
      for (var u in Object(r))
        i.push(u);
    return i;
  }
  return th = t, th;
}
var nh, wx;
function RN() {
  if (wx) return nh;
  wx = 1;
  var t = rn(), r = es(), i = CN(), u = Object.prototype, o = u.hasOwnProperty;
  function s(c) {
    if (!t(c))
      return i(c);
    var f = r(c), g = [];
    for (var h in c)
      h == "constructor" && (f || !o.call(c, h)) || g.push(h);
    return g;
  }
  return nh = s, nh;
}
var rh, Ax;
function ha() {
  if (Ax) return rh;
  Ax = 1;
  var t = kA(), r = RN(), i = Wn();
  function u(o) {
    return i(o) ? t(o, !0) : r(o);
  }
  return rh = u, rh;
}
var ah, Tx;
function NN() {
  if (Tx) return ah;
  Tx = 1;
  var t = Uu(), r = ha();
  function i(u, o) {
    return u && t(o, r(o), u);
  }
  return ah = i, ah;
}
var Eu = { exports: {} };
Eu.exports;
var Mx;
function IA() {
  return Mx || (Mx = 1, (function(t, r) {
    var i = yn(), u = r && !r.nodeType && r, o = u && !0 && t && !t.nodeType && t, s = o && o.exports === u, c = s ? i.Buffer : void 0, f = c ? c.allocUnsafe : void 0;
    function g(h, v) {
      if (v)
        return h.slice();
      var p = h.length, m = f ? f(p) : new h.constructor(p);
      return h.copy(m), m;
    }
    t.exports = g;
  })(Eu, Eu.exports)), Eu.exports;
}
var ih, qx;
function QA() {
  if (qx) return ih;
  qx = 1;
  function t(r, i) {
    var u = -1, o = r.length;
    for (i || (i = Array(o)); ++u < o; )
      i[u] = r[u];
    return i;
  }
  return ih = t, ih;
}
var uh, Cx;
function ZA() {
  if (Cx) return uh;
  Cx = 1;
  function t(r, i) {
    for (var u = -1, o = r == null ? 0 : r.length, s = 0, c = []; ++u < o; ) {
      var f = r[u];
      i(f, u, r) && (c[s++] = f);
    }
    return c;
  }
  return uh = t, uh;
}
var lh, Rx;
function KA() {
  if (Rx) return lh;
  Rx = 1;
  function t() {
    return [];
  }
  return lh = t, lh;
}
var oh, Nx;
function Up() {
  if (Nx) return oh;
  Nx = 1;
  var t = ZA(), r = KA(), i = Object.prototype, u = i.propertyIsEnumerable, o = Object.getOwnPropertySymbols, s = o ? function(c) {
    return c == null ? [] : (c = Object(c), t(o(c), function(f) {
      return u.call(c, f);
    }));
  } : r;
  return oh = s, oh;
}
var sh, Ox;
function ON() {
  if (Ox) return sh;
  Ox = 1;
  var t = Uu(), r = Up();
  function i(u, o) {
    return t(u, r(u), o);
  }
  return sh = i, sh;
}
var ch, zx;
function Gp() {
  if (zx) return ch;
  zx = 1;
  function t(r, i) {
    for (var u = -1, o = i.length, s = r.length; ++u < o; )
      r[s + u] = i[u];
    return r;
  }
  return ch = t, ch;
}
var fh, Dx;
function ts() {
  if (Dx) return fh;
  Dx = 1;
  var t = XA(), r = t(Object.getPrototypeOf, Object);
  return fh = r, fh;
}
var dh, Hx;
function $A() {
  if (Hx) return dh;
  Hx = 1;
  var t = Gp(), r = ts(), i = Up(), u = KA(), o = Object.getOwnPropertySymbols, s = o ? function(c) {
    for (var f = []; c; )
      t(f, i(c)), c = r(c);
    return f;
  } : u;
  return dh = s, dh;
}
var hh, Lx;
function zN() {
  if (Lx) return hh;
  Lx = 1;
  var t = Uu(), r = $A();
  function i(u, o) {
    return t(u, r(u), o);
  }
  return hh = i, hh;
}
var gh, Bx;
function FA() {
  if (Bx) return gh;
  Bx = 1;
  var t = Gp(), r = st();
  function i(u, o, s) {
    var c = o(u);
    return r(u) ? c : t(c, s(u));
  }
  return gh = i, gh;
}
var vh, jx;
function JA() {
  if (jx) return vh;
  jx = 1;
  var t = FA(), r = Up(), i = Hr();
  function u(o) {
    return t(o, i, r);
  }
  return vh = u, vh;
}
var yh, Ux;
function DN() {
  if (Ux) return yh;
  Ux = 1;
  var t = FA(), r = $A(), i = ha();
  function u(o) {
    return t(o, i, r);
  }
  return yh = u, yh;
}
var ph, Gx;
function HN() {
  if (Gx) return ph;
  Gx = 1;
  var t = da(), r = yn(), i = t(r, "DataView");
  return ph = i, ph;
}
var mh, Vx;
function LN() {
  if (Vx) return mh;
  Vx = 1;
  var t = da(), r = yn(), i = t(r, "Promise");
  return mh = i, mh;
}
var bh, Yx;
function PA() {
  if (Yx) return bh;
  Yx = 1;
  var t = da(), r = yn(), i = t(r, "Set");
  return bh = i, bh;
}
var _h, kx;
function BN() {
  if (kx) return _h;
  kx = 1;
  var t = da(), r = yn(), i = t(r, "WeakMap");
  return _h = i, _h;
}
var xh, Xx;
function mi() {
  if (Xx) return xh;
  Xx = 1;
  var t = HN(), r = zp(), i = LN(), u = PA(), o = BN(), s = fa(), c = VA(), f = "[object Map]", g = "[object Object]", h = "[object Promise]", v = "[object Set]", p = "[object WeakMap]", m = "[object DataView]", b = c(t), _ = c(r), A = c(i), w = c(u), E = c(o), M = s;
  return (t && M(new t(new ArrayBuffer(1))) != m || r && M(new r()) != f || i && M(i.resolve()) != h || u && M(new u()) != v || o && M(new o()) != p) && (M = function(S) {
    var T = s(S), O = T == g ? S.constructor : void 0, C = O ? c(O) : "";
    if (C)
      switch (C) {
        case b:
          return m;
        case _:
          return f;
        case A:
          return h;
        case w:
          return v;
        case E:
          return p;
      }
    return T;
  }), xh = M, xh;
}
var Sh, Ix;
function jN() {
  if (Ix) return Sh;
  Ix = 1;
  var t = Object.prototype, r = t.hasOwnProperty;
  function i(u) {
    var o = u.length, s = new u.constructor(o);
    return o && typeof u[0] == "string" && r.call(u, "index") && (s.index = u.index, s.input = u.input), s;
  }
  return Sh = i, Sh;
}
var Eh, Qx;
function WA() {
  if (Qx) return Eh;
  Qx = 1;
  var t = yn(), r = t.Uint8Array;
  return Eh = r, Eh;
}
var wh, Zx;
function Vp() {
  if (Zx) return wh;
  Zx = 1;
  var t = WA();
  function r(i) {
    var u = new i.constructor(i.byteLength);
    return new t(u).set(new t(i)), u;
  }
  return wh = r, wh;
}
var Ah, Kx;
function UN() {
  if (Kx) return Ah;
  Kx = 1;
  var t = Vp();
  function r(i, u) {
    var o = u ? t(i.buffer) : i.buffer;
    return new i.constructor(o, i.byteOffset, i.byteLength);
  }
  return Ah = r, Ah;
}
var Th, $x;
function GN() {
  if ($x) return Th;
  $x = 1;
  var t = /\w*$/;
  function r(i) {
    var u = new i.constructor(i.source, t.exec(i));
    return u.lastIndex = i.lastIndex, u;
  }
  return Th = r, Th;
}
var Mh, Fx;
function VN() {
  if (Fx) return Mh;
  Fx = 1;
  var t = yi(), r = t ? t.prototype : void 0, i = r ? r.valueOf : void 0;
  function u(o) {
    return i ? Object(i.call(o)) : {};
  }
  return Mh = u, Mh;
}
var qh, Jx;
function eT() {
  if (Jx) return qh;
  Jx = 1;
  var t = Vp();
  function r(i, u) {
    var o = u ? t(i.buffer) : i.buffer;
    return new i.constructor(o, i.byteOffset, i.length);
  }
  return qh = r, qh;
}
var Ch, Px;
function YN() {
  if (Px) return Ch;
  Px = 1;
  var t = Vp(), r = UN(), i = GN(), u = VN(), o = eT(), s = "[object Boolean]", c = "[object Date]", f = "[object Map]", g = "[object Number]", h = "[object RegExp]", v = "[object Set]", p = "[object String]", m = "[object Symbol]", b = "[object ArrayBuffer]", _ = "[object DataView]", A = "[object Float32Array]", w = "[object Float64Array]", E = "[object Int8Array]", M = "[object Int16Array]", S = "[object Int32Array]", T = "[object Uint8Array]", O = "[object Uint8ClampedArray]", C = "[object Uint16Array]", R = "[object Uint32Array]";
  function H(B, X, Y) {
    var F = B.constructor;
    switch (X) {
      case b:
        return t(B);
      case s:
      case c:
        return new F(+B);
      case _:
        return r(B, Y);
      case A:
      case w:
      case E:
      case M:
      case S:
      case T:
      case O:
      case C:
      case R:
        return o(B, Y);
      case f:
        return new F();
      case g:
      case p:
        return new F(B);
      case h:
        return i(B);
      case v:
        return new F();
      case m:
        return u(B);
    }
  }
  return Ch = H, Ch;
}
var Rh, Wx;
function tT() {
  if (Wx) return Rh;
  Wx = 1;
  var t = rn(), r = Object.create, i = /* @__PURE__ */ (function() {
    function u() {
    }
    return function(o) {
      if (!t(o))
        return {};
      if (r)
        return r(o);
      u.prototype = o;
      var s = new u();
      return u.prototype = void 0, s;
    };
  })();
  return Rh = i, Rh;
}
var Nh, eS;
function nT() {
  if (eS) return Nh;
  eS = 1;
  var t = tT(), r = ts(), i = es();
  function u(o) {
    return typeof o.constructor == "function" && !i(o) ? t(r(o)) : {};
  }
  return Nh = u, Nh;
}
var Oh, tS;
function kN() {
  if (tS) return Oh;
  tS = 1;
  var t = mi(), r = Tn(), i = "[object Map]";
  function u(o) {
    return r(o) && t(o) == i;
  }
  return Oh = u, Oh;
}
var zh, nS;
function XN() {
  if (nS) return zh;
  nS = 1;
  var t = kN(), r = Wo(), i = Bp(), u = i && i.isMap, o = u ? r(u) : t;
  return zh = o, zh;
}
var Dh, rS;
function IN() {
  if (rS) return Dh;
  rS = 1;
  var t = mi(), r = Tn(), i = "[object Set]";
  function u(o) {
    return r(o) && t(o) == i;
  }
  return Dh = u, Dh;
}
var Hh, aS;
function QN() {
  if (aS) return Hh;
  aS = 1;
  var t = IN(), r = Wo(), i = Bp(), u = i && i.isSet, o = u ? r(u) : t;
  return Hh = o, Hh;
}
var Lh, iS;
function rT() {
  if (iS) return Lh;
  iS = 1;
  var t = $o(), r = Hp(), i = Jo(), u = qN(), o = NN(), s = IA(), c = QA(), f = ON(), g = zN(), h = JA(), v = DN(), p = mi(), m = jN(), b = YN(), _ = nT(), A = st(), w = pi(), E = XN(), M = rn(), S = QN(), T = Hr(), O = ha(), C = 1, R = 2, H = 4, B = "[object Arguments]", X = "[object Array]", Y = "[object Boolean]", F = "[object Date]", K = "[object Error]", D = "[object Function]", G = "[object GeneratorFunction]", N = "[object Map]", j = "[object Number]", Z = "[object Object]", Q = "[object RegExp]", le = "[object Set]", z = "[object String]", V = "[object Symbol]", ie = "[object WeakMap]", L = "[object ArrayBuffer]", I = "[object DataView]", P = "[object Float32Array]", ae = "[object Float64Array]", W = "[object Int8Array]", se = "[object Int16Array]", de = "[object Int32Array]", ve = "[object Uint8Array]", pe = "[object Uint8ClampedArray]", he = "[object Uint16Array]", me = "[object Uint32Array]", ge = {};
  ge[B] = ge[X] = ge[L] = ge[I] = ge[Y] = ge[F] = ge[P] = ge[ae] = ge[W] = ge[se] = ge[de] = ge[N] = ge[j] = ge[Z] = ge[Q] = ge[le] = ge[z] = ge[V] = ge[ve] = ge[pe] = ge[he] = ge[me] = !0, ge[K] = ge[D] = ge[ie] = !1;
  function Ae(xe, Pe, tt, xt, gt, St) {
    var Ze, ke = Pe & C, vt = Pe & R, Ct = Pe & H;
    if (tt && (Ze = gt ? tt(xe, xt, gt, St) : tt(xe)), Ze !== void 0)
      return Ze;
    if (!M(xe))
      return xe;
    var Tt = A(xe);
    if (Tt) {
      if (Ze = m(xe), !ke)
        return c(xe, Ze);
    } else {
      var ft = p(xe), tr = ft == D || ft == G;
      if (w(xe))
        return s(xe, ke);
      if (ft == Z || ft == B || tr && !gt) {
        if (Ze = vt || tr ? {} : _(xe), !ke)
          return vt ? g(xe, o(Ze, xe)) : f(xe, u(Ze, xe));
      } else {
        if (!ge[ft])
          return gt ? xe : {};
        Ze = b(xe, ft, ke);
      }
    }
    St || (St = new t());
    var Cn = St.get(xe);
    if (Cn)
      return Cn;
    St.set(xe, Ze), S(xe) ? xe.forEach(function(Et) {
      Ze.add(Ae(Et, Pe, tt, Et, xe, St));
    }) : E(xe) && xe.forEach(function(Et, Fe) {
      Ze.set(Fe, Ae(Et, Pe, tt, Fe, xe, St));
    });
    var _i = Ct ? vt ? v : h : vt ? O : T, va = Tt ? void 0 : _i(xe);
    return r(va || xe, function(Et, Fe) {
      va && (Fe = Et, Et = xe[Fe]), i(Ze, Fe, Ae(Et, Pe, tt, Fe, xe, St));
    }), Ze;
  }
  return Lh = Ae, Lh;
}
var Bh, uS;
function ZN() {
  if (uS) return Bh;
  uS = 1;
  var t = rT(), r = 4;
  function i(u) {
    return t(u, r);
  }
  return Bh = i, Bh;
}
var jh, lS;
function Yp() {
  if (lS) return jh;
  lS = 1;
  function t(r) {
    return function() {
      return r;
    };
  }
  return jh = t, jh;
}
var Uh, oS;
function KN() {
  if (oS) return Uh;
  oS = 1;
  function t(r) {
    return function(i, u, o) {
      for (var s = -1, c = Object(i), f = o(i), g = f.length; g--; ) {
        var h = f[r ? g : ++s];
        if (u(c[h], h, c) === !1)
          break;
      }
      return i;
    };
  }
  return Uh = t, Uh;
}
var Gh, sS;
function kp() {
  if (sS) return Gh;
  sS = 1;
  var t = KN(), r = t();
  return Gh = r, Gh;
}
var Vh, cS;
function Xp() {
  if (cS) return Vh;
  cS = 1;
  var t = kp(), r = Hr();
  function i(u, o) {
    return u && t(u, o, r);
  }
  return Vh = i, Vh;
}
var Yh, fS;
function $N() {
  if (fS) return Yh;
  fS = 1;
  var t = Wn();
  function r(i, u) {
    return function(o, s) {
      if (o == null)
        return o;
      if (!t(o))
        return i(o, s);
      for (var c = o.length, f = u ? c : -1, g = Object(o); (u ? f-- : ++f < c) && s(g[f], f, g) !== !1; )
        ;
      return o;
    };
  }
  return Yh = r, Yh;
}
var kh, dS;
function ns() {
  if (dS) return kh;
  dS = 1;
  var t = Xp(), r = $N(), i = r(t);
  return kh = i, kh;
}
var Xh, hS;
function ga() {
  if (hS) return Xh;
  hS = 1;
  function t(r) {
    return r;
  }
  return Xh = t, Xh;
}
var Ih, gS;
function aT() {
  if (gS) return Ih;
  gS = 1;
  var t = ga();
  function r(i) {
    return typeof i == "function" ? i : t;
  }
  return Ih = r, Ih;
}
var Qh, vS;
function iT() {
  if (vS) return Qh;
  vS = 1;
  var t = Hp(), r = ns(), i = aT(), u = st();
  function o(s, c) {
    var f = u(s) ? t : r;
    return f(s, i(c));
  }
  return Qh = o, Qh;
}
var Zh, yS;
function uT() {
  return yS || (yS = 1, Zh = iT()), Zh;
}
var Kh, pS;
function FN() {
  if (pS) return Kh;
  pS = 1;
  var t = ns();
  function r(i, u) {
    var o = [];
    return t(i, function(s, c, f) {
      u(s, c, f) && o.push(s);
    }), o;
  }
  return Kh = r, Kh;
}
var $h, mS;
function JN() {
  if (mS) return $h;
  mS = 1;
  var t = "__lodash_hash_undefined__";
  function r(i) {
    return this.__data__.set(i, t), this;
  }
  return $h = r, $h;
}
var Fh, bS;
function PN() {
  if (bS) return Fh;
  bS = 1;
  function t(r) {
    return this.__data__.has(r);
  }
  return Fh = t, Fh;
}
var Jh, _S;
function lT() {
  if (_S) return Jh;
  _S = 1;
  var t = Dp(), r = JN(), i = PN();
  function u(o) {
    var s = -1, c = o == null ? 0 : o.length;
    for (this.__data__ = new t(); ++s < c; )
      this.add(o[s]);
  }
  return u.prototype.add = u.prototype.push = r, u.prototype.has = i, Jh = u, Jh;
}
var Ph, xS;
function WN() {
  if (xS) return Ph;
  xS = 1;
  function t(r, i) {
    for (var u = -1, o = r == null ? 0 : r.length; ++u < o; )
      if (i(r[u], u, r))
        return !0;
    return !1;
  }
  return Ph = t, Ph;
}
var Wh, SS;
function oT() {
  if (SS) return Wh;
  SS = 1;
  function t(r, i) {
    return r.has(i);
  }
  return Wh = t, Wh;
}
var eg, ES;
function sT() {
  if (ES) return eg;
  ES = 1;
  var t = lT(), r = WN(), i = oT(), u = 1, o = 2;
  function s(c, f, g, h, v, p) {
    var m = g & u, b = c.length, _ = f.length;
    if (b != _ && !(m && _ > b))
      return !1;
    var A = p.get(c), w = p.get(f);
    if (A && w)
      return A == f && w == c;
    var E = -1, M = !0, S = g & o ? new t() : void 0;
    for (p.set(c, f), p.set(f, c); ++E < b; ) {
      var T = c[E], O = f[E];
      if (h)
        var C = m ? h(O, T, E, f, c, p) : h(T, O, E, c, f, p);
      if (C !== void 0) {
        if (C)
          continue;
        M = !1;
        break;
      }
      if (S) {
        if (!r(f, function(R, H) {
          if (!i(S, H) && (T === R || v(T, R, g, h, p)))
            return S.push(H);
        })) {
          M = !1;
          break;
        }
      } else if (!(T === O || v(T, O, g, h, p))) {
        M = !1;
        break;
      }
    }
    return p.delete(c), p.delete(f), M;
  }
  return eg = s, eg;
}
var tg, wS;
function e3() {
  if (wS) return tg;
  wS = 1;
  function t(r) {
    var i = -1, u = Array(r.size);
    return r.forEach(function(o, s) {
      u[++i] = [s, o];
    }), u;
  }
  return tg = t, tg;
}
var ng, AS;
function Ip() {
  if (AS) return ng;
  AS = 1;
  function t(r) {
    var i = -1, u = Array(r.size);
    return r.forEach(function(o) {
      u[++i] = o;
    }), u;
  }
  return ng = t, ng;
}
var rg, TS;
function t3() {
  if (TS) return rg;
  TS = 1;
  var t = yi(), r = WA(), i = vi(), u = sT(), o = e3(), s = Ip(), c = 1, f = 2, g = "[object Boolean]", h = "[object Date]", v = "[object Error]", p = "[object Map]", m = "[object Number]", b = "[object RegExp]", _ = "[object Set]", A = "[object String]", w = "[object Symbol]", E = "[object ArrayBuffer]", M = "[object DataView]", S = t ? t.prototype : void 0, T = S ? S.valueOf : void 0;
  function O(C, R, H, B, X, Y, F) {
    switch (H) {
      case M:
        if (C.byteLength != R.byteLength || C.byteOffset != R.byteOffset)
          return !1;
        C = C.buffer, R = R.buffer;
      case E:
        return !(C.byteLength != R.byteLength || !Y(new r(C), new r(R)));
      case g:
      case h:
      case m:
        return i(+C, +R);
      case v:
        return C.name == R.name && C.message == R.message;
      case b:
      case A:
        return C == R + "";
      case p:
        var K = o;
      case _:
        var D = B & c;
        if (K || (K = s), C.size != R.size && !D)
          return !1;
        var G = F.get(C);
        if (G)
          return G == R;
        B |= f, F.set(C, R);
        var N = u(K(C), K(R), B, X, Y, F);
        return F.delete(C), N;
      case w:
        if (T)
          return T.call(C) == T.call(R);
    }
    return !1;
  }
  return rg = O, rg;
}
var ag, MS;
function n3() {
  if (MS) return ag;
  MS = 1;
  var t = JA(), r = 1, i = Object.prototype, u = i.hasOwnProperty;
  function o(s, c, f, g, h, v) {
    var p = f & r, m = t(s), b = m.length, _ = t(c), A = _.length;
    if (b != A && !p)
      return !1;
    for (var w = b; w--; ) {
      var E = m[w];
      if (!(p ? E in c : u.call(c, E)))
        return !1;
    }
    var M = v.get(s), S = v.get(c);
    if (M && S)
      return M == c && S == s;
    var T = !0;
    v.set(s, c), v.set(c, s);
    for (var O = p; ++w < b; ) {
      E = m[w];
      var C = s[E], R = c[E];
      if (g)
        var H = p ? g(R, C, E, c, s, v) : g(C, R, E, s, c, v);
      if (!(H === void 0 ? C === R || h(C, R, f, g, v) : H)) {
        T = !1;
        break;
      }
      O || (O = E == "constructor");
    }
    if (T && !O) {
      var B = s.constructor, X = c.constructor;
      B != X && "constructor" in s && "constructor" in c && !(typeof B == "function" && B instanceof B && typeof X == "function" && X instanceof X) && (T = !1);
    }
    return v.delete(s), v.delete(c), T;
  }
  return ag = o, ag;
}
var ig, qS;
function r3() {
  if (qS) return ig;
  qS = 1;
  var t = $o(), r = sT(), i = t3(), u = n3(), o = mi(), s = st(), c = pi(), f = Vu(), g = 1, h = "[object Arguments]", v = "[object Array]", p = "[object Object]", m = Object.prototype, b = m.hasOwnProperty;
  function _(A, w, E, M, S, T) {
    var O = s(A), C = s(w), R = O ? v : o(A), H = C ? v : o(w);
    R = R == h ? p : R, H = H == h ? p : H;
    var B = R == p, X = H == p, Y = R == H;
    if (Y && c(A)) {
      if (!c(w))
        return !1;
      O = !0, B = !1;
    }
    if (Y && !B)
      return T || (T = new t()), O || f(A) ? r(A, w, E, M, S, T) : i(A, w, R, E, M, S, T);
    if (!(E & g)) {
      var F = B && b.call(A, "__wrapped__"), K = X && b.call(w, "__wrapped__");
      if (F || K) {
        var D = F ? A.value() : A, G = K ? w.value() : w;
        return T || (T = new t()), S(D, G, E, M, T);
      }
    }
    return Y ? (T || (T = new t()), u(A, w, E, M, S, T)) : !1;
  }
  return ig = _, ig;
}
var ug, CS;
function cT() {
  if (CS) return ug;
  CS = 1;
  var t = r3(), r = Tn();
  function i(u, o, s, c, f) {
    return u === o ? !0 : u == null || o == null || !r(u) && !r(o) ? u !== u && o !== o : t(u, o, s, c, i, f);
  }
  return ug = i, ug;
}
var lg, RS;
function a3() {
  if (RS) return lg;
  RS = 1;
  var t = $o(), r = cT(), i = 1, u = 2;
  function o(s, c, f, g) {
    var h = f.length, v = h, p = !g;
    if (s == null)
      return !v;
    for (s = Object(s); h--; ) {
      var m = f[h];
      if (p && m[2] ? m[1] !== s[m[0]] : !(m[0] in s))
        return !1;
    }
    for (; ++h < v; ) {
      m = f[h];
      var b = m[0], _ = s[b], A = m[1];
      if (p && m[2]) {
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
  return lg = o, lg;
}
var og, NS;
function fT() {
  if (NS) return og;
  NS = 1;
  var t = rn();
  function r(i) {
    return i === i && !t(i);
  }
  return og = r, og;
}
var sg, OS;
function i3() {
  if (OS) return sg;
  OS = 1;
  var t = fT(), r = Hr();
  function i(u) {
    for (var o = r(u), s = o.length; s--; ) {
      var c = o[s], f = u[c];
      o[s] = [c, f, t(f)];
    }
    return o;
  }
  return sg = i, sg;
}
var cg, zS;
function dT() {
  if (zS) return cg;
  zS = 1;
  function t(r, i) {
    return function(u) {
      return u == null ? !1 : u[r] === i && (i !== void 0 || r in Object(u));
    };
  }
  return cg = t, cg;
}
var fg, DS;
function u3() {
  if (DS) return fg;
  DS = 1;
  var t = a3(), r = i3(), i = dT();
  function u(o) {
    var s = r(o);
    return s.length == 1 && s[0][2] ? i(s[0][0], s[0][1]) : function(c) {
      return c === o || t(c, o, s);
    };
  }
  return fg = u, fg;
}
var dg, HS;
function bi() {
  if (HS) return dg;
  HS = 1;
  var t = fa(), r = Tn(), i = "[object Symbol]";
  function u(o) {
    return typeof o == "symbol" || r(o) && t(o) == i;
  }
  return dg = u, dg;
}
var hg, LS;
function Qp() {
  if (LS) return hg;
  LS = 1;
  var t = st(), r = bi(), i = /\.|\[(?:[^[\]]*|(["'])(?:(?!\1)[^\\]|\\.)*?\1)\]/, u = /^\w*$/;
  function o(s, c) {
    if (t(s))
      return !1;
    var f = typeof s;
    return f == "number" || f == "symbol" || f == "boolean" || s == null || r(s) ? !0 : u.test(s) || !i.test(s) || c != null && s in Object(c);
  }
  return hg = o, hg;
}
var gg, BS;
function l3() {
  if (BS) return gg;
  BS = 1;
  var t = Dp(), r = "Expected a function";
  function i(u, o) {
    if (typeof u != "function" || o != null && typeof o != "function")
      throw new TypeError(r);
    var s = function() {
      var c = arguments, f = o ? o.apply(this, c) : c[0], g = s.cache;
      if (g.has(f))
        return g.get(f);
      var h = u.apply(this, c);
      return s.cache = g.set(f, h) || g, h;
    };
    return s.cache = new (i.Cache || t)(), s;
  }
  return i.Cache = t, gg = i, gg;
}
var vg, jS;
function o3() {
  if (jS) return vg;
  jS = 1;
  var t = l3(), r = 500;
  function i(u) {
    var o = t(u, function(c) {
      return s.size === r && s.clear(), c;
    }), s = o.cache;
    return o;
  }
  return vg = i, vg;
}
var yg, US;
function s3() {
  if (US) return yg;
  US = 1;
  var t = o3(), r = /[^.[\]]+|\[(?:(-?\d+(?:\.\d+)?)|(["'])((?:(?!\2)[^\\]|\\.)*?)\2)\]|(?=(?:\.|\[\])(?:\.|\[\]|$))/g, i = /\\(\\)?/g, u = t(function(o) {
    var s = [];
    return o.charCodeAt(0) === 46 && s.push(""), o.replace(r, function(c, f, g, h) {
      s.push(g ? h.replace(i, "$1") : f || c);
    }), s;
  });
  return yg = u, yg;
}
var pg, GS;
function rs() {
  if (GS) return pg;
  GS = 1;
  function t(r, i) {
    for (var u = -1, o = r == null ? 0 : r.length, s = Array(o); ++u < o; )
      s[u] = i(r[u], u, r);
    return s;
  }
  return pg = t, pg;
}
var mg, VS;
function c3() {
  if (VS) return mg;
  VS = 1;
  var t = yi(), r = rs(), i = st(), u = bi(), o = t ? t.prototype : void 0, s = o ? o.toString : void 0;
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
  return mg = c, mg;
}
var bg, YS;
function hT() {
  if (YS) return bg;
  YS = 1;
  var t = c3();
  function r(i) {
    return i == null ? "" : t(i);
  }
  return bg = r, bg;
}
var _g, kS;
function as() {
  if (kS) return _g;
  kS = 1;
  var t = st(), r = Qp(), i = s3(), u = hT();
  function o(s, c) {
    return t(s) ? s : r(s, c) ? [s] : i(u(s));
  }
  return _g = o, _g;
}
var xg, XS;
function Yu() {
  if (XS) return xg;
  XS = 1;
  var t = bi();
  function r(i) {
    if (typeof i == "string" || t(i))
      return i;
    var u = i + "";
    return u == "0" && 1 / i == -1 / 0 ? "-0" : u;
  }
  return xg = r, xg;
}
var Sg, IS;
function is() {
  if (IS) return Sg;
  IS = 1;
  var t = as(), r = Yu();
  function i(u, o) {
    o = t(o, u);
    for (var s = 0, c = o.length; u != null && s < c; )
      u = u[r(o[s++])];
    return s && s == c ? u : void 0;
  }
  return Sg = i, Sg;
}
var Eg, QS;
function f3() {
  if (QS) return Eg;
  QS = 1;
  var t = is();
  function r(i, u, o) {
    var s = i == null ? void 0 : t(i, u);
    return s === void 0 ? o : s;
  }
  return Eg = r, Eg;
}
var wg, ZS;
function d3() {
  if (ZS) return wg;
  ZS = 1;
  function t(r, i) {
    return r != null && i in Object(r);
  }
  return wg = t, wg;
}
var Ag, KS;
function gT() {
  if (KS) return Ag;
  KS = 1;
  var t = as(), r = Gu(), i = st(), u = Po(), o = Lp(), s = Yu();
  function c(f, g, h) {
    g = t(g, f);
    for (var v = -1, p = g.length, m = !1; ++v < p; ) {
      var b = s(g[v]);
      if (!(m = f != null && h(f, b)))
        break;
      f = f[b];
    }
    return m || ++v != p ? m : (p = f == null ? 0 : f.length, !!p && o(p) && u(b, p) && (i(f) || r(f)));
  }
  return Ag = c, Ag;
}
var Tg, $S;
function vT() {
  if ($S) return Tg;
  $S = 1;
  var t = d3(), r = gT();
  function i(u, o) {
    return u != null && r(u, o, t);
  }
  return Tg = i, Tg;
}
var Mg, FS;
function h3() {
  if (FS) return Mg;
  FS = 1;
  var t = cT(), r = f3(), i = vT(), u = Qp(), o = fT(), s = dT(), c = Yu(), f = 1, g = 2;
  function h(v, p) {
    return u(v) && o(p) ? s(c(v), p) : function(m) {
      var b = r(m, v);
      return b === void 0 && b === p ? i(m, v) : t(p, b, f | g);
    };
  }
  return Mg = h, Mg;
}
var qg, JS;
function yT() {
  if (JS) return qg;
  JS = 1;
  function t(r) {
    return function(i) {
      return i == null ? void 0 : i[r];
    };
  }
  return qg = t, qg;
}
var Cg, PS;
function g3() {
  if (PS) return Cg;
  PS = 1;
  var t = is();
  function r(i) {
    return function(u) {
      return t(u, i);
    };
  }
  return Cg = r, Cg;
}
var Rg, WS;
function v3() {
  if (WS) return Rg;
  WS = 1;
  var t = yT(), r = g3(), i = Qp(), u = Yu();
  function o(s) {
    return i(s) ? t(u(s)) : r(s);
  }
  return Rg = o, Rg;
}
var Ng, eE;
function er() {
  if (eE) return Ng;
  eE = 1;
  var t = u3(), r = h3(), i = ga(), u = st(), o = v3();
  function s(c) {
    return typeof c == "function" ? c : c == null ? i : typeof c == "object" ? u(c) ? r(c[0], c[1]) : t(c) : o(c);
  }
  return Ng = s, Ng;
}
var Og, tE;
function pT() {
  if (tE) return Og;
  tE = 1;
  var t = ZA(), r = FN(), i = er(), u = st();
  function o(s, c) {
    var f = u(s) ? t : r;
    return f(s, i(c, 3));
  }
  return Og = o, Og;
}
var zg, nE;
function y3() {
  if (nE) return zg;
  nE = 1;
  var t = Object.prototype, r = t.hasOwnProperty;
  function i(u, o) {
    return u != null && r.call(u, o);
  }
  return zg = i, zg;
}
var Dg, rE;
function mT() {
  if (rE) return Dg;
  rE = 1;
  var t = y3(), r = gT();
  function i(u, o) {
    return u != null && r(u, o, t);
  }
  return Dg = i, Dg;
}
var Hg, aE;
function p3() {
  if (aE) return Hg;
  aE = 1;
  var t = jp(), r = mi(), i = Gu(), u = st(), o = Wn(), s = pi(), c = es(), f = Vu(), g = "[object Map]", h = "[object Set]", v = Object.prototype, p = v.hasOwnProperty;
  function m(b) {
    if (b == null)
      return !0;
    if (o(b) && (u(b) || typeof b == "string" || typeof b.splice == "function" || s(b) || f(b) || i(b)))
      return !b.length;
    var _ = r(b);
    if (_ == g || _ == h)
      return !b.size;
    if (c(b))
      return !t(b).length;
    for (var A in b)
      if (p.call(b, A))
        return !1;
    return !0;
  }
  return Hg = m, Hg;
}
var Lg, iE;
function bT() {
  if (iE) return Lg;
  iE = 1;
  function t(r) {
    return r === void 0;
  }
  return Lg = t, Lg;
}
var Bg, uE;
function _T() {
  if (uE) return Bg;
  uE = 1;
  var t = ns(), r = Wn();
  function i(u, o) {
    var s = -1, c = r(u) ? Array(u.length) : [];
    return t(u, function(f, g, h) {
      c[++s] = o(f, g, h);
    }), c;
  }
  return Bg = i, Bg;
}
var jg, lE;
function xT() {
  if (lE) return jg;
  lE = 1;
  var t = rs(), r = er(), i = _T(), u = st();
  function o(s, c) {
    var f = u(s) ? t : i;
    return f(s, r(c, 3));
  }
  return jg = o, jg;
}
var Ug, oE;
function m3() {
  if (oE) return Ug;
  oE = 1;
  function t(r, i, u, o) {
    var s = -1, c = r == null ? 0 : r.length;
    for (o && c && (u = r[++s]); ++s < c; )
      u = i(u, r[s], s, r);
    return u;
  }
  return Ug = t, Ug;
}
var Gg, sE;
function b3() {
  if (sE) return Gg;
  sE = 1;
  function t(r, i, u, o, s) {
    return s(r, function(c, f, g) {
      u = o ? (o = !1, c) : i(u, c, f, g);
    }), u;
  }
  return Gg = t, Gg;
}
var Vg, cE;
function ST() {
  if (cE) return Vg;
  cE = 1;
  var t = m3(), r = ns(), i = er(), u = b3(), o = st();
  function s(c, f, g) {
    var h = o(c) ? t : u, v = arguments.length < 3;
    return h(c, i(f, 4), g, v, r);
  }
  return Vg = s, Vg;
}
var Yg, fE;
function _3() {
  if (fE) return Yg;
  fE = 1;
  var t = fa(), r = st(), i = Tn(), u = "[object String]";
  function o(s) {
    return typeof s == "string" || !r(s) && i(s) && t(s) == u;
  }
  return Yg = o, Yg;
}
var kg, dE;
function x3() {
  if (dE) return kg;
  dE = 1;
  var t = yT(), r = t("length");
  return kg = r, kg;
}
var Xg, hE;
function S3() {
  if (hE) return Xg;
  hE = 1;
  var t = "\\ud800-\\udfff", r = "\\u0300-\\u036f", i = "\\ufe20-\\ufe2f", u = "\\u20d0-\\u20ff", o = r + i + u, s = "\\ufe0e\\ufe0f", c = "\\u200d", f = RegExp("[" + c + t + o + s + "]");
  function g(h) {
    return f.test(h);
  }
  return Xg = g, Xg;
}
var Ig, gE;
function E3() {
  if (gE) return Ig;
  gE = 1;
  var t = "\\ud800-\\udfff", r = "\\u0300-\\u036f", i = "\\ufe20-\\ufe2f", u = "\\u20d0-\\u20ff", o = r + i + u, s = "\\ufe0e\\ufe0f", c = "[" + t + "]", f = "[" + o + "]", g = "\\ud83c[\\udffb-\\udfff]", h = "(?:" + f + "|" + g + ")", v = "[^" + t + "]", p = "(?:\\ud83c[\\udde6-\\uddff]){2}", m = "[\\ud800-\\udbff][\\udc00-\\udfff]", b = "\\u200d", _ = h + "?", A = "[" + s + "]?", w = "(?:" + b + "(?:" + [v, p, m].join("|") + ")" + A + _ + ")*", E = A + _ + w, M = "(?:" + [v + f + "?", f, p, m, c].join("|") + ")", S = RegExp(g + "(?=" + g + ")|" + M + E, "g");
  function T(O) {
    for (var C = S.lastIndex = 0; S.test(O); )
      ++C;
    return C;
  }
  return Ig = T, Ig;
}
var Qg, vE;
function w3() {
  if (vE) return Qg;
  vE = 1;
  var t = x3(), r = S3(), i = E3();
  function u(o) {
    return r(o) ? i(o) : t(o);
  }
  return Qg = u, Qg;
}
var Zg, yE;
function A3() {
  if (yE) return Zg;
  yE = 1;
  var t = jp(), r = mi(), i = Wn(), u = _3(), o = w3(), s = "[object Map]", c = "[object Set]";
  function f(g) {
    if (g == null)
      return 0;
    if (i(g))
      return u(g) ? o(g) : g.length;
    var h = r(g);
    return h == s || h == c ? g.size : t(g).length;
  }
  return Zg = f, Zg;
}
var Kg, pE;
function T3() {
  if (pE) return Kg;
  pE = 1;
  var t = Hp(), r = tT(), i = Xp(), u = er(), o = ts(), s = st(), c = pi(), f = ju(), g = rn(), h = Vu();
  function v(p, m, b) {
    var _ = s(p), A = _ || c(p) || h(p);
    if (m = u(m, 4), b == null) {
      var w = p && p.constructor;
      A ? b = _ ? new w() : [] : g(p) ? b = f(w) ? r(o(p)) : {} : b = {};
    }
    return (A ? t : i)(p, function(E, M, S) {
      return m(b, E, M, S);
    }), b;
  }
  return Kg = v, Kg;
}
var $g, mE;
function M3() {
  if (mE) return $g;
  mE = 1;
  var t = yi(), r = Gu(), i = st(), u = t ? t.isConcatSpreadable : void 0;
  function o(s) {
    return i(s) || r(s) || !!(u && s && s[u]);
  }
  return $g = o, $g;
}
var Fg, bE;
function Zp() {
  if (bE) return Fg;
  bE = 1;
  var t = Gp(), r = M3();
  function i(u, o, s, c, f) {
    var g = -1, h = u.length;
    for (s || (s = r), f || (f = []); ++g < h; ) {
      var v = u[g];
      o > 0 && s(v) ? o > 1 ? i(v, o - 1, s, c, f) : t(f, v) : c || (f[f.length] = v);
    }
    return f;
  }
  return Fg = i, Fg;
}
var Jg, _E;
function q3() {
  if (_E) return Jg;
  _E = 1;
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
  return Jg = t, Jg;
}
var Pg, xE;
function ET() {
  if (xE) return Pg;
  xE = 1;
  var t = q3(), r = Math.max;
  function i(u, o, s) {
    return o = r(o === void 0 ? u.length - 1 : o, 0), function() {
      for (var c = arguments, f = -1, g = r(c.length - o, 0), h = Array(g); ++f < g; )
        h[f] = c[o + f];
      f = -1;
      for (var v = Array(o + 1); ++f < o; )
        v[f] = c[f];
      return v[o] = s(h), t(u, this, v);
    };
  }
  return Pg = i, Pg;
}
var Wg, SE;
function C3() {
  if (SE) return Wg;
  SE = 1;
  var t = Yp(), r = YA(), i = ga(), u = r ? function(o, s) {
    return r(o, "toString", {
      configurable: !0,
      enumerable: !1,
      value: t(s),
      writable: !0
    });
  } : i;
  return Wg = u, Wg;
}
var ev, EE;
function R3() {
  if (EE) return ev;
  EE = 1;
  var t = 800, r = 16, i = Date.now;
  function u(o) {
    var s = 0, c = 0;
    return function() {
      var f = i(), g = r - (f - c);
      if (c = f, g > 0) {
        if (++s >= t)
          return arguments[0];
      } else
        s = 0;
      return o.apply(void 0, arguments);
    };
  }
  return ev = u, ev;
}
var tv, wE;
function wT() {
  if (wE) return tv;
  wE = 1;
  var t = C3(), r = R3(), i = r(t);
  return tv = i, tv;
}
var nv, AE;
function us() {
  if (AE) return nv;
  AE = 1;
  var t = ga(), r = ET(), i = wT();
  function u(o, s) {
    return i(r(o, s, t), o + "");
  }
  return nv = u, nv;
}
var rv, TE;
function AT() {
  if (TE) return rv;
  TE = 1;
  function t(r, i, u, o) {
    for (var s = r.length, c = u + (o ? 1 : -1); o ? c-- : ++c < s; )
      if (i(r[c], c, r))
        return c;
    return -1;
  }
  return rv = t, rv;
}
var av, ME;
function N3() {
  if (ME) return av;
  ME = 1;
  function t(r) {
    return r !== r;
  }
  return av = t, av;
}
var iv, qE;
function O3() {
  if (qE) return iv;
  qE = 1;
  function t(r, i, u) {
    for (var o = u - 1, s = r.length; ++o < s; )
      if (r[o] === i)
        return o;
    return -1;
  }
  return iv = t, iv;
}
var uv, CE;
function z3() {
  if (CE) return uv;
  CE = 1;
  var t = AT(), r = N3(), i = O3();
  function u(o, s, c) {
    return s === s ? i(o, s, c) : t(o, r, c);
  }
  return uv = u, uv;
}
var lv, RE;
function D3() {
  if (RE) return lv;
  RE = 1;
  var t = z3();
  function r(i, u) {
    var o = i == null ? 0 : i.length;
    return !!o && t(i, u, 0) > -1;
  }
  return lv = r, lv;
}
var ov, NE;
function H3() {
  if (NE) return ov;
  NE = 1;
  function t(r, i, u) {
    for (var o = -1, s = r == null ? 0 : r.length; ++o < s; )
      if (u(i, r[o]))
        return !0;
    return !1;
  }
  return ov = t, ov;
}
var sv, OE;
function L3() {
  if (OE) return sv;
  OE = 1;
  function t() {
  }
  return sv = t, sv;
}
var cv, zE;
function B3() {
  if (zE) return cv;
  zE = 1;
  var t = PA(), r = L3(), i = Ip(), u = 1 / 0, o = t && 1 / i(new t([, -0]))[1] == u ? function(s) {
    return new t(s);
  } : r;
  return cv = o, cv;
}
var fv, DE;
function j3() {
  if (DE) return fv;
  DE = 1;
  var t = lT(), r = D3(), i = H3(), u = oT(), o = B3(), s = Ip(), c = 200;
  function f(g, h, v) {
    var p = -1, m = r, b = g.length, _ = !0, A = [], w = A;
    if (v)
      _ = !1, m = i;
    else if (b >= c) {
      var E = h ? null : o(g);
      if (E)
        return s(E);
      _ = !1, m = u, w = new t();
    } else
      w = h ? [] : A;
    e:
      for (; ++p < b; ) {
        var M = g[p], S = h ? h(M) : M;
        if (M = v || M !== 0 ? M : 0, _ && S === S) {
          for (var T = w.length; T--; )
            if (w[T] === S)
              continue e;
          h && w.push(S), A.push(M);
        } else m(w, S, v) || (w !== A && w.push(S), A.push(M));
      }
    return A;
  }
  return fv = f, fv;
}
var dv, HE;
function TT() {
  if (HE) return dv;
  HE = 1;
  var t = Wn(), r = Tn();
  function i(u) {
    return r(u) && t(u);
  }
  return dv = i, dv;
}
var hv, LE;
function U3() {
  if (LE) return hv;
  LE = 1;
  var t = Zp(), r = us(), i = j3(), u = TT(), o = r(function(s) {
    return i(t(s, 1, u, !0));
  });
  return hv = o, hv;
}
var gv, BE;
function G3() {
  if (BE) return gv;
  BE = 1;
  var t = rs();
  function r(i, u) {
    return t(u, function(o) {
      return i[o];
    });
  }
  return gv = r, gv;
}
var vv, jE;
function MT() {
  if (jE) return vv;
  jE = 1;
  var t = G3(), r = Hr();
  function i(u) {
    return u == null ? [] : t(u, r(u));
  }
  return vv = i, vv;
}
var yv, UE;
function an() {
  if (UE) return yv;
  UE = 1;
  var t;
  if (typeof Op == "function")
    try {
      t = {
        clone: ZN(),
        constant: Yp(),
        each: uT(),
        filter: pT(),
        has: mT(),
        isArray: st(),
        isEmpty: p3(),
        isFunction: ju(),
        isUndefined: bT(),
        keys: Hr(),
        map: xT(),
        reduce: ST(),
        size: A3(),
        transform: T3(),
        union: U3(),
        values: MT()
      };
    } catch {
    }
  return t || (t = window._), yv = t, yv;
}
var pv, GE;
function Kp() {
  if (GE) return pv;
  GE = 1;
  var t = an();
  pv = o;
  var r = "\0", i = "\0", u = "";
  function o(v) {
    this._isDirected = t.has(v, "directed") ? v.directed : !0, this._isMultigraph = t.has(v, "multigraph") ? v.multigraph : !1, this._isCompound = t.has(v, "compound") ? v.compound : !1, this._label = void 0, this._defaultNodeLabelFn = t.constant(void 0), this._defaultEdgeLabelFn = t.constant(void 0), this._nodes = {}, this._isCompound && (this._parent = {}, this._children = {}, this._children[i] = {}), this._in = {}, this._preds = {}, this._out = {}, this._sucs = {}, this._edgeObjs = {}, this._edgeLabels = {};
  }
  o.prototype._nodeCount = 0, o.prototype._edgeCount = 0, o.prototype.isDirected = function() {
    return this._isDirected;
  }, o.prototype.isMultigraph = function() {
    return this._isMultigraph;
  }, o.prototype.isCompound = function() {
    return this._isCompound;
  }, o.prototype.setGraph = function(v) {
    return this._label = v, this;
  }, o.prototype.graph = function() {
    return this._label;
  }, o.prototype.setDefaultNodeLabel = function(v) {
    return t.isFunction(v) || (v = t.constant(v)), this._defaultNodeLabelFn = v, this;
  }, o.prototype.nodeCount = function() {
    return this._nodeCount;
  }, o.prototype.nodes = function() {
    return t.keys(this._nodes);
  }, o.prototype.sources = function() {
    var v = this;
    return t.filter(this.nodes(), function(p) {
      return t.isEmpty(v._in[p]);
    });
  }, o.prototype.sinks = function() {
    var v = this;
    return t.filter(this.nodes(), function(p) {
      return t.isEmpty(v._out[p]);
    });
  }, o.prototype.setNodes = function(v, p) {
    var m = arguments, b = this;
    return t.each(v, function(_) {
      m.length > 1 ? b.setNode(_, p) : b.setNode(_);
    }), this;
  }, o.prototype.setNode = function(v, p) {
    return t.has(this._nodes, v) ? (arguments.length > 1 && (this._nodes[v] = p), this) : (this._nodes[v] = arguments.length > 1 ? p : this._defaultNodeLabelFn(v), this._isCompound && (this._parent[v] = i, this._children[v] = {}, this._children[i][v] = !0), this._in[v] = {}, this._preds[v] = {}, this._out[v] = {}, this._sucs[v] = {}, ++this._nodeCount, this);
  }, o.prototype.node = function(v) {
    return this._nodes[v];
  }, o.prototype.hasNode = function(v) {
    return t.has(this._nodes, v);
  }, o.prototype.removeNode = function(v) {
    var p = this;
    if (t.has(this._nodes, v)) {
      var m = function(b) {
        p.removeEdge(p._edgeObjs[b]);
      };
      delete this._nodes[v], this._isCompound && (this._removeFromParentsChildList(v), delete this._parent[v], t.each(this.children(v), function(b) {
        p.setParent(b);
      }), delete this._children[v]), t.each(t.keys(this._in[v]), m), delete this._in[v], delete this._preds[v], t.each(t.keys(this._out[v]), m), delete this._out[v], delete this._sucs[v], --this._nodeCount;
    }
    return this;
  }, o.prototype.setParent = function(v, p) {
    if (!this._isCompound)
      throw new Error("Cannot set parent in a non-compound graph");
    if (t.isUndefined(p))
      p = i;
    else {
      p += "";
      for (var m = p; !t.isUndefined(m); m = this.parent(m))
        if (m === v)
          throw new Error("Setting " + p + " as parent of " + v + " would create a cycle");
      this.setNode(p);
    }
    return this.setNode(v), this._removeFromParentsChildList(v), this._parent[v] = p, this._children[p][v] = !0, this;
  }, o.prototype._removeFromParentsChildList = function(v) {
    delete this._children[this._parent[v]][v];
  }, o.prototype.parent = function(v) {
    if (this._isCompound) {
      var p = this._parent[v];
      if (p !== i)
        return p;
    }
  }, o.prototype.children = function(v) {
    if (t.isUndefined(v) && (v = i), this._isCompound) {
      var p = this._children[v];
      if (p)
        return t.keys(p);
    } else {
      if (v === i)
        return this.nodes();
      if (this.hasNode(v))
        return [];
    }
  }, o.prototype.predecessors = function(v) {
    var p = this._preds[v];
    if (p)
      return t.keys(p);
  }, o.prototype.successors = function(v) {
    var p = this._sucs[v];
    if (p)
      return t.keys(p);
  }, o.prototype.neighbors = function(v) {
    var p = this.predecessors(v);
    if (p)
      return t.union(p, this.successors(v));
  }, o.prototype.isLeaf = function(v) {
    var p;
    return this.isDirected() ? p = this.successors(v) : p = this.neighbors(v), p.length === 0;
  }, o.prototype.filterNodes = function(v) {
    var p = new this.constructor({
      directed: this._isDirected,
      multigraph: this._isMultigraph,
      compound: this._isCompound
    });
    p.setGraph(this.graph());
    var m = this;
    t.each(this._nodes, function(A, w) {
      v(w) && p.setNode(w, A);
    }), t.each(this._edgeObjs, function(A) {
      p.hasNode(A.v) && p.hasNode(A.w) && p.setEdge(A, m.edge(A));
    });
    var b = {};
    function _(A) {
      var w = m.parent(A);
      return w === void 0 || p.hasNode(w) ? (b[A] = w, w) : w in b ? b[w] : _(w);
    }
    return this._isCompound && t.each(p.nodes(), function(A) {
      p.setParent(A, _(A));
    }), p;
  }, o.prototype.setDefaultEdgeLabel = function(v) {
    return t.isFunction(v) || (v = t.constant(v)), this._defaultEdgeLabelFn = v, this;
  }, o.prototype.edgeCount = function() {
    return this._edgeCount;
  }, o.prototype.edges = function() {
    return t.values(this._edgeObjs);
  }, o.prototype.setPath = function(v, p) {
    var m = this, b = arguments;
    return t.reduce(v, function(_, A) {
      return b.length > 1 ? m.setEdge(_, A, p) : m.setEdge(_, A), A;
    }), this;
  }, o.prototype.setEdge = function() {
    var v, p, m, b, _ = !1, A = arguments[0];
    typeof A == "object" && A !== null && "v" in A ? (v = A.v, p = A.w, m = A.name, arguments.length === 2 && (b = arguments[1], _ = !0)) : (v = A, p = arguments[1], m = arguments[3], arguments.length > 2 && (b = arguments[2], _ = !0)), v = "" + v, p = "" + p, t.isUndefined(m) || (m = "" + m);
    var w = f(this._isDirected, v, p, m);
    if (t.has(this._edgeLabels, w))
      return _ && (this._edgeLabels[w] = b), this;
    if (!t.isUndefined(m) && !this._isMultigraph)
      throw new Error("Cannot set a named edge when isMultigraph = false");
    this.setNode(v), this.setNode(p), this._edgeLabels[w] = _ ? b : this._defaultEdgeLabelFn(v, p, m);
    var E = g(this._isDirected, v, p, m);
    return v = E.v, p = E.w, Object.freeze(E), this._edgeObjs[w] = E, s(this._preds[p], v), s(this._sucs[v], p), this._in[p][w] = E, this._out[v][w] = E, this._edgeCount++, this;
  }, o.prototype.edge = function(v, p, m) {
    var b = arguments.length === 1 ? h(this._isDirected, arguments[0]) : f(this._isDirected, v, p, m);
    return this._edgeLabels[b];
  }, o.prototype.hasEdge = function(v, p, m) {
    var b = arguments.length === 1 ? h(this._isDirected, arguments[0]) : f(this._isDirected, v, p, m);
    return t.has(this._edgeLabels, b);
  }, o.prototype.removeEdge = function(v, p, m) {
    var b = arguments.length === 1 ? h(this._isDirected, arguments[0]) : f(this._isDirected, v, p, m), _ = this._edgeObjs[b];
    return _ && (v = _.v, p = _.w, delete this._edgeLabels[b], delete this._edgeObjs[b], c(this._preds[p], v), c(this._sucs[v], p), delete this._in[p][b], delete this._out[v][b], this._edgeCount--), this;
  }, o.prototype.inEdges = function(v, p) {
    var m = this._in[v];
    if (m) {
      var b = t.values(m);
      return p ? t.filter(b, function(_) {
        return _.v === p;
      }) : b;
    }
  }, o.prototype.outEdges = function(v, p) {
    var m = this._out[v];
    if (m) {
      var b = t.values(m);
      return p ? t.filter(b, function(_) {
        return _.w === p;
      }) : b;
    }
  }, o.prototype.nodeEdges = function(v, p) {
    var m = this.inEdges(v, p);
    if (m)
      return m.concat(this.outEdges(v, p));
  };
  function s(v, p) {
    v[p] ? v[p]++ : v[p] = 1;
  }
  function c(v, p) {
    --v[p] || delete v[p];
  }
  function f(v, p, m, b) {
    var _ = "" + p, A = "" + m;
    if (!v && _ > A) {
      var w = _;
      _ = A, A = w;
    }
    return _ + u + A + u + (t.isUndefined(b) ? r : b);
  }
  function g(v, p, m, b) {
    var _ = "" + p, A = "" + m;
    if (!v && _ > A) {
      var w = _;
      _ = A, A = w;
    }
    var E = { v: _, w: A };
    return b && (E.name = b), E;
  }
  function h(v, p) {
    return f(v, p.v, p.w, p.name);
  }
  return pv;
}
var mv, VE;
function V3() {
  return VE || (VE = 1, mv = "2.1.8"), mv;
}
var bv, YE;
function Y3() {
  return YE || (YE = 1, bv = {
    Graph: Kp(),
    version: V3()
  }), bv;
}
var _v, kE;
function k3() {
  if (kE) return _v;
  kE = 1;
  var t = an(), r = Kp();
  _v = {
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
      edges: o(c)
    };
    return t.isUndefined(c.graph()) || (f.value = t.clone(c.graph())), f;
  }
  function u(c) {
    return t.map(c.nodes(), function(f) {
      var g = c.node(f), h = c.parent(f), v = { v: f };
      return t.isUndefined(g) || (v.value = g), t.isUndefined(h) || (v.parent = h), v;
    });
  }
  function o(c) {
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
  return _v;
}
var xv, XE;
function X3() {
  if (XE) return xv;
  XE = 1;
  var t = an();
  xv = r;
  function r(i) {
    var u = {}, o = [], s;
    function c(f) {
      t.has(u, f) || (u[f] = !0, s.push(f), t.each(i.successors(f), c), t.each(i.predecessors(f), c));
    }
    return t.each(i.nodes(), function(f) {
      s = [], c(f), s.length && o.push(s);
    }), o;
  }
  return xv;
}
var Sv, IE;
function qT() {
  if (IE) return Sv;
  IE = 1;
  var t = an();
  Sv = r;
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
    var o = this._keyIndices;
    if (i = String(i), !t.has(o, i)) {
      var s = this._arr, c = s.length;
      return o[i] = c, s.push({ key: i, priority: u }), this._decrease(c), !0;
    }
    return !1;
  }, r.prototype.removeMin = function() {
    this._swap(0, this._arr.length - 1);
    var i = this._arr.pop();
    return delete this._keyIndices[i.key], this._heapify(0), i.key;
  }, r.prototype.decrease = function(i, u) {
    var o = this._keyIndices[i];
    if (u > this._arr[o].priority)
      throw new Error("New priority is greater than current priority. Key: " + i + " Old: " + this._arr[o].priority + " New: " + u);
    this._arr[o].priority = u, this._decrease(o);
  }, r.prototype._heapify = function(i) {
    var u = this._arr, o = 2 * i, s = o + 1, c = i;
    o < u.length && (c = u[o].priority < u[c].priority ? o : c, s < u.length && (c = u[s].priority < u[c].priority ? s : c), c !== i && (this._swap(i, c), this._heapify(c)));
  }, r.prototype._decrease = function(i) {
    for (var u = this._arr, o = u[i].priority, s; i !== 0 && (s = i >> 1, !(u[s].priority < o)); )
      this._swap(i, s), i = s;
  }, r.prototype._swap = function(i, u) {
    var o = this._arr, s = this._keyIndices, c = o[i], f = o[u];
    o[i] = f, o[u] = c, s[f.key] = i, s[c.key] = u;
  }, Sv;
}
var Ev, QE;
function CT() {
  if (QE) return Ev;
  QE = 1;
  var t = an(), r = qT();
  Ev = u;
  var i = t.constant(1);
  function u(s, c, f, g) {
    return o(
      s,
      String(c),
      f || i,
      g || function(h) {
        return s.outEdges(h);
      }
    );
  }
  function o(s, c, f, g) {
    var h = {}, v = new r(), p, m, b = function(_) {
      var A = _.v !== p ? _.v : _.w, w = h[A], E = f(_), M = m.distance + E;
      if (E < 0)
        throw new Error("dijkstra does not allow negative edge weights. Bad edge: " + _ + " Weight: " + E);
      M < w.distance && (w.distance = M, w.predecessor = p, v.decrease(A, M));
    };
    for (s.nodes().forEach(function(_) {
      var A = _ === c ? 0 : Number.POSITIVE_INFINITY;
      h[_] = { distance: A }, v.add(_, A);
    }); v.size() > 0 && (p = v.removeMin(), m = h[p], m.distance !== Number.POSITIVE_INFINITY); )
      g(p).forEach(b);
    return h;
  }
  return Ev;
}
var wv, ZE;
function I3() {
  if (ZE) return wv;
  ZE = 1;
  var t = CT(), r = an();
  wv = i;
  function i(u, o, s) {
    return r.transform(u.nodes(), function(c, f) {
      c[f] = t(u, f, o, s);
    }, {});
  }
  return wv;
}
var Av, KE;
function RT() {
  if (KE) return Av;
  KE = 1;
  var t = an();
  Av = r;
  function r(i) {
    var u = 0, o = [], s = {}, c = [];
    function f(g) {
      var h = s[g] = {
        onStack: !0,
        lowlink: u,
        index: u++
      };
      if (o.push(g), i.successors(g).forEach(function(m) {
        t.has(s, m) ? s[m].onStack && (h.lowlink = Math.min(h.lowlink, s[m].index)) : (f(m), h.lowlink = Math.min(h.lowlink, s[m].lowlink));
      }), h.lowlink === h.index) {
        var v = [], p;
        do
          p = o.pop(), s[p].onStack = !1, v.push(p);
        while (g !== p);
        c.push(v);
      }
    }
    return i.nodes().forEach(function(g) {
      t.has(s, g) || f(g);
    }), c;
  }
  return Av;
}
var Tv, $E;
function Q3() {
  if ($E) return Tv;
  $E = 1;
  var t = an(), r = RT();
  Tv = i;
  function i(u) {
    return t.filter(r(u), function(o) {
      return o.length > 1 || o.length === 1 && u.hasEdge(o[0], o[0]);
    });
  }
  return Tv;
}
var Mv, FE;
function Z3() {
  if (FE) return Mv;
  FE = 1;
  var t = an();
  Mv = i;
  var r = t.constant(1);
  function i(o, s, c) {
    return u(
      o,
      s || r,
      c || function(f) {
        return o.outEdges(f);
      }
    );
  }
  function u(o, s, c) {
    var f = {}, g = o.nodes();
    return g.forEach(function(h) {
      f[h] = {}, f[h][h] = { distance: 0 }, g.forEach(function(v) {
        h !== v && (f[h][v] = { distance: Number.POSITIVE_INFINITY });
      }), c(h).forEach(function(v) {
        var p = v.v === h ? v.w : v.v, m = s(v);
        f[h][p] = { distance: m, predecessor: h };
      });
    }), g.forEach(function(h) {
      var v = f[h];
      g.forEach(function(p) {
        var m = f[p];
        g.forEach(function(b) {
          var _ = m[h], A = v[b], w = m[b], E = _.distance + A.distance;
          E < w.distance && (w.distance = E, w.predecessor = A.predecessor);
        });
      });
    }), f;
  }
  return Mv;
}
var qv, JE;
function NT() {
  if (JE) return qv;
  JE = 1;
  var t = an();
  qv = r, r.CycleException = i;
  function r(u) {
    var o = {}, s = {}, c = [];
    function f(g) {
      if (t.has(s, g))
        throw new i();
      t.has(o, g) || (s[g] = !0, o[g] = !0, t.each(u.predecessors(g), f), delete s[g], c.push(g));
    }
    if (t.each(u.sinks(), f), t.size(o) !== u.nodeCount())
      throw new i();
    return c;
  }
  function i() {
  }
  return i.prototype = new Error(), qv;
}
var Cv, PE;
function K3() {
  if (PE) return Cv;
  PE = 1;
  var t = NT();
  Cv = r;
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
  return Cv;
}
var Rv, WE;
function OT() {
  if (WE) return Rv;
  WE = 1;
  var t = an();
  Rv = r;
  function r(u, o, s) {
    t.isArray(o) || (o = [o]);
    var c = (u.isDirected() ? u.successors : u.neighbors).bind(u), f = [], g = {};
    return t.each(o, function(h) {
      if (!u.hasNode(h))
        throw new Error("Graph does not have node: " + h);
      i(u, h, s === "post", g, c, f);
    }), f;
  }
  function i(u, o, s, c, f, g) {
    t.has(c, o) || (c[o] = !0, s || g.push(o), t.each(f(o), function(h) {
      i(u, h, s, c, f, g);
    }), s && g.push(o));
  }
  return Rv;
}
var Nv, ew;
function $3() {
  if (ew) return Nv;
  ew = 1;
  var t = OT();
  Nv = r;
  function r(i, u) {
    return t(i, u, "post");
  }
  return Nv;
}
var Ov, tw;
function F3() {
  if (tw) return Ov;
  tw = 1;
  var t = OT();
  Ov = r;
  function r(i, u) {
    return t(i, u, "pre");
  }
  return Ov;
}
var zv, nw;
function J3() {
  if (nw) return zv;
  nw = 1;
  var t = an(), r = Kp(), i = qT();
  zv = u;
  function u(o, s) {
    var c = new r(), f = {}, g = new i(), h;
    function v(m) {
      var b = m.v === h ? m.w : m.v, _ = g.priority(b);
      if (_ !== void 0) {
        var A = s(m);
        A < _ && (f[b] = h, g.decrease(b, A));
      }
    }
    if (o.nodeCount() === 0)
      return c;
    t.each(o.nodes(), function(m) {
      g.add(m, Number.POSITIVE_INFINITY), c.setNode(m);
    }), g.decrease(o.nodes()[0], 0);
    for (var p = !1; g.size() > 0; ) {
      if (h = g.removeMin(), t.has(f, h))
        c.setEdge(h, f[h]);
      else {
        if (p)
          throw new Error("Input graph is not connected: " + o);
        p = !0;
      }
      o.nodeEdges(h).forEach(v);
    }
    return c;
  }
  return zv;
}
var Dv, rw;
function P3() {
  return rw || (rw = 1, Dv = {
    components: X3(),
    dijkstra: CT(),
    dijkstraAll: I3(),
    findCycles: Q3(),
    floydWarshall: Z3(),
    isAcyclic: K3(),
    postorder: $3(),
    preorder: F3(),
    prim: J3(),
    tarjan: RT(),
    topsort: NT()
  }), Dv;
}
var Hv, aw;
function W3() {
  if (aw) return Hv;
  aw = 1;
  var t = Y3();
  return Hv = {
    Graph: t.Graph,
    json: k3(),
    alg: P3(),
    version: t.version
  }, Hv;
}
var Lv, iw;
function gn() {
  if (iw) return Lv;
  iw = 1;
  var t;
  if (typeof Op == "function")
    try {
      t = W3();
    } catch {
    }
  return t || (t = window.graphlib), Lv = t, Lv;
}
var Bv, uw;
function eO() {
  if (uw) return Bv;
  uw = 1;
  var t = rT(), r = 1, i = 4;
  function u(o) {
    return t(o, r | i);
  }
  return Bv = u, Bv;
}
var jv, lw;
function ls() {
  if (lw) return jv;
  lw = 1;
  var t = vi(), r = Wn(), i = Po(), u = rn();
  function o(s, c, f) {
    if (!u(f))
      return !1;
    var g = typeof c;
    return (g == "number" ? r(f) && i(c, f.length) : g == "string" && c in f) ? t(f[c], s) : !1;
  }
  return jv = o, jv;
}
var Uv, ow;
function tO() {
  if (ow) return Uv;
  ow = 1;
  var t = us(), r = vi(), i = ls(), u = ha(), o = Object.prototype, s = o.hasOwnProperty, c = t(function(f, g) {
    f = Object(f);
    var h = -1, v = g.length, p = v > 2 ? g[2] : void 0;
    for (p && i(g[0], g[1], p) && (v = 1); ++h < v; )
      for (var m = g[h], b = u(m), _ = -1, A = b.length; ++_ < A; ) {
        var w = b[_], E = f[w];
        (E === void 0 || r(E, o[w]) && !s.call(f, w)) && (f[w] = m[w]);
      }
    return f;
  });
  return Uv = c, Uv;
}
var Gv, sw;
function nO() {
  if (sw) return Gv;
  sw = 1;
  var t = er(), r = Wn(), i = Hr();
  function u(o) {
    return function(s, c, f) {
      var g = Object(s);
      if (!r(s)) {
        var h = t(c, 3);
        s = i(s), c = function(p) {
          return h(g[p], p, g);
        };
      }
      var v = o(s, c, f);
      return v > -1 ? g[h ? s[v] : v] : void 0;
    };
  }
  return Gv = u, Gv;
}
var Vv, cw;
function rO() {
  if (cw) return Vv;
  cw = 1;
  var t = /\s/;
  function r(i) {
    for (var u = i.length; u-- && t.test(i.charAt(u)); )
      ;
    return u;
  }
  return Vv = r, Vv;
}
var Yv, fw;
function aO() {
  if (fw) return Yv;
  fw = 1;
  var t = rO(), r = /^\s+/;
  function i(u) {
    return u && u.slice(0, t(u) + 1).replace(r, "");
  }
  return Yv = i, Yv;
}
var kv, dw;
function iO() {
  if (dw) return kv;
  dw = 1;
  var t = aO(), r = rn(), i = bi(), u = NaN, o = /^[-+]0x[0-9a-f]+$/i, s = /^0b[01]+$/i, c = /^0o[0-7]+$/i, f = parseInt;
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
    var p = s.test(h);
    return p || c.test(h) ? f(h.slice(2), p ? 2 : 8) : o.test(h) ? u : +h;
  }
  return kv = g, kv;
}
var Xv, hw;
function zT() {
  if (hw) return Xv;
  hw = 1;
  var t = iO(), r = 1 / 0, i = 17976931348623157e292;
  function u(o) {
    if (!o)
      return o === 0 ? o : 0;
    if (o = t(o), o === r || o === -r) {
      var s = o < 0 ? -1 : 1;
      return s * i;
    }
    return o === o ? o : 0;
  }
  return Xv = u, Xv;
}
var Iv, gw;
function uO() {
  if (gw) return Iv;
  gw = 1;
  var t = zT();
  function r(i) {
    var u = t(i), o = u % 1;
    return u === u ? o ? u - o : u : 0;
  }
  return Iv = r, Iv;
}
var Qv, vw;
function lO() {
  if (vw) return Qv;
  vw = 1;
  var t = AT(), r = er(), i = uO(), u = Math.max;
  function o(s, c, f) {
    var g = s == null ? 0 : s.length;
    if (!g)
      return -1;
    var h = f == null ? 0 : i(f);
    return h < 0 && (h = u(g + h, 0)), t(s, r(c, 3), h);
  }
  return Qv = o, Qv;
}
var Zv, yw;
function oO() {
  if (yw) return Zv;
  yw = 1;
  var t = nO(), r = lO(), i = t(r);
  return Zv = i, Zv;
}
var Kv, pw;
function DT() {
  if (pw) return Kv;
  pw = 1;
  var t = Zp();
  function r(i) {
    var u = i == null ? 0 : i.length;
    return u ? t(i, 1) : [];
  }
  return Kv = r, Kv;
}
var $v, mw;
function sO() {
  if (mw) return $v;
  mw = 1;
  var t = kp(), r = aT(), i = ha();
  function u(o, s) {
    return o == null ? o : t(o, r(s), i);
  }
  return $v = u, $v;
}
var Fv, bw;
function cO() {
  if (bw) return Fv;
  bw = 1;
  function t(r) {
    var i = r == null ? 0 : r.length;
    return i ? r[i - 1] : void 0;
  }
  return Fv = t, Fv;
}
var Jv, _w;
function fO() {
  if (_w) return Jv;
  _w = 1;
  var t = Fo(), r = Xp(), i = er();
  function u(o, s) {
    var c = {};
    return s = i(s, 3), r(o, function(f, g, h) {
      t(c, g, s(f, g, h));
    }), c;
  }
  return Jv = u, Jv;
}
var Pv, xw;
function $p() {
  if (xw) return Pv;
  xw = 1;
  var t = bi();
  function r(i, u, o) {
    for (var s = -1, c = i.length; ++s < c; ) {
      var f = i[s], g = u(f);
      if (g != null && (h === void 0 ? g === g && !t(g) : o(g, h)))
        var h = g, v = f;
    }
    return v;
  }
  return Pv = r, Pv;
}
var Wv, Sw;
function dO() {
  if (Sw) return Wv;
  Sw = 1;
  function t(r, i) {
    return r > i;
  }
  return Wv = t, Wv;
}
var ey, Ew;
function hO() {
  if (Ew) return ey;
  Ew = 1;
  var t = $p(), r = dO(), i = ga();
  function u(o) {
    return o && o.length ? t(o, i, r) : void 0;
  }
  return ey = u, ey;
}
var ty, ww;
function HT() {
  if (ww) return ty;
  ww = 1;
  var t = Fo(), r = vi();
  function i(u, o, s) {
    (s !== void 0 && !r(u[o], s) || s === void 0 && !(o in u)) && t(u, o, s);
  }
  return ty = i, ty;
}
var ny, Aw;
function gO() {
  if (Aw) return ny;
  Aw = 1;
  var t = fa(), r = ts(), i = Tn(), u = "[object Object]", o = Function.prototype, s = Object.prototype, c = o.toString, f = s.hasOwnProperty, g = c.call(Object);
  function h(v) {
    if (!i(v) || t(v) != u)
      return !1;
    var p = r(v);
    if (p === null)
      return !0;
    var m = f.call(p, "constructor") && p.constructor;
    return typeof m == "function" && m instanceof m && c.call(m) == g;
  }
  return ny = h, ny;
}
var ry, Tw;
function LT() {
  if (Tw) return ry;
  Tw = 1;
  function t(r, i) {
    if (!(i === "constructor" && typeof r[i] == "function") && i != "__proto__")
      return r[i];
  }
  return ry = t, ry;
}
var ay, Mw;
function vO() {
  if (Mw) return ay;
  Mw = 1;
  var t = Uu(), r = ha();
  function i(u) {
    return t(u, r(u));
  }
  return ay = i, ay;
}
var iy, qw;
function yO() {
  if (qw) return iy;
  qw = 1;
  var t = HT(), r = IA(), i = eT(), u = QA(), o = nT(), s = Gu(), c = st(), f = TT(), g = pi(), h = ju(), v = rn(), p = gO(), m = Vu(), b = LT(), _ = vO();
  function A(w, E, M, S, T, O, C) {
    var R = b(w, M), H = b(E, M), B = C.get(H);
    if (B) {
      t(w, M, B);
      return;
    }
    var X = O ? O(R, H, M + "", w, E, C) : void 0, Y = X === void 0;
    if (Y) {
      var F = c(H), K = !F && g(H), D = !F && !K && m(H);
      X = H, F || K || D ? c(R) ? X = R : f(R) ? X = u(R) : K ? (Y = !1, X = r(H, !0)) : D ? (Y = !1, X = i(H, !0)) : X = [] : p(H) || s(H) ? (X = R, s(R) ? X = _(R) : (!v(R) || h(R)) && (X = o(H))) : Y = !1;
    }
    Y && (C.set(H, X), T(X, H, S, O, C), C.delete(H)), t(w, M, X);
  }
  return iy = A, iy;
}
var uy, Cw;
function pO() {
  if (Cw) return uy;
  Cw = 1;
  var t = $o(), r = HT(), i = kp(), u = yO(), o = rn(), s = ha(), c = LT();
  function f(g, h, v, p, m) {
    g !== h && i(h, function(b, _) {
      if (m || (m = new t()), o(b))
        u(g, h, _, v, f, p, m);
      else {
        var A = p ? p(c(g, _), b, _ + "", g, h, m) : void 0;
        A === void 0 && (A = b), r(g, _, A);
      }
    }, s);
  }
  return uy = f, uy;
}
var ly, Rw;
function mO() {
  if (Rw) return ly;
  Rw = 1;
  var t = us(), r = ls();
  function i(u) {
    return t(function(o, s) {
      var c = -1, f = s.length, g = f > 1 ? s[f - 1] : void 0, h = f > 2 ? s[2] : void 0;
      for (g = u.length > 3 && typeof g == "function" ? (f--, g) : void 0, h && r(s[0], s[1], h) && (g = f < 3 ? void 0 : g, f = 1), o = Object(o); ++c < f; ) {
        var v = s[c];
        v && u(o, v, c, g);
      }
      return o;
    });
  }
  return ly = i, ly;
}
var oy, Nw;
function bO() {
  if (Nw) return oy;
  Nw = 1;
  var t = pO(), r = mO(), i = r(function(u, o, s) {
    t(u, o, s);
  });
  return oy = i, oy;
}
var sy, Ow;
function BT() {
  if (Ow) return sy;
  Ow = 1;
  function t(r, i) {
    return r < i;
  }
  return sy = t, sy;
}
var cy, zw;
function _O() {
  if (zw) return cy;
  zw = 1;
  var t = $p(), r = BT(), i = ga();
  function u(o) {
    return o && o.length ? t(o, i, r) : void 0;
  }
  return cy = u, cy;
}
var fy, Dw;
function xO() {
  if (Dw) return fy;
  Dw = 1;
  var t = $p(), r = er(), i = BT();
  function u(o, s) {
    return o && o.length ? t(o, r(s, 2), i) : void 0;
  }
  return fy = u, fy;
}
var dy, Hw;
function SO() {
  if (Hw) return dy;
  Hw = 1;
  var t = yn(), r = function() {
    return t.Date.now();
  };
  return dy = r, dy;
}
var hy, Lw;
function EO() {
  if (Lw) return hy;
  Lw = 1;
  var t = Jo(), r = as(), i = Po(), u = rn(), o = Yu();
  function s(c, f, g, h) {
    if (!u(c))
      return c;
    f = r(f, c);
    for (var v = -1, p = f.length, m = p - 1, b = c; b != null && ++v < p; ) {
      var _ = o(f[v]), A = g;
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
  return hy = s, hy;
}
var gy, Bw;
function wO() {
  if (Bw) return gy;
  Bw = 1;
  var t = is(), r = EO(), i = as();
  function u(o, s, c) {
    for (var f = -1, g = s.length, h = {}; ++f < g; ) {
      var v = s[f], p = t(o, v);
      c(p, v) && r(h, i(v, o), p);
    }
    return h;
  }
  return gy = u, gy;
}
var vy, jw;
function AO() {
  if (jw) return vy;
  jw = 1;
  var t = wO(), r = vT();
  function i(u, o) {
    return t(u, o, function(s, c) {
      return r(u, c);
    });
  }
  return vy = i, vy;
}
var yy, Uw;
function TO() {
  if (Uw) return yy;
  Uw = 1;
  var t = DT(), r = ET(), i = wT();
  function u(o) {
    return i(r(o, void 0, t), o + "");
  }
  return yy = u, yy;
}
var py, Gw;
function MO() {
  if (Gw) return py;
  Gw = 1;
  var t = AO(), r = TO(), i = r(function(u, o) {
    return u == null ? {} : t(u, o);
  });
  return py = i, py;
}
var my, Vw;
function qO() {
  if (Vw) return my;
  Vw = 1;
  var t = Math.ceil, r = Math.max;
  function i(u, o, s, c) {
    for (var f = -1, g = r(t((o - u) / (s || 1)), 0), h = Array(g); g--; )
      h[c ? g : ++f] = u, u += s;
    return h;
  }
  return my = i, my;
}
var by, Yw;
function CO() {
  if (Yw) return by;
  Yw = 1;
  var t = qO(), r = ls(), i = zT();
  function u(o) {
    return function(s, c, f) {
      return f && typeof f != "number" && r(s, c, f) && (c = f = void 0), s = i(s), c === void 0 ? (c = s, s = 0) : c = i(c), f = f === void 0 ? s < c ? 1 : -1 : i(f), t(s, c, f, o);
    };
  }
  return by = u, by;
}
var _y, kw;
function RO() {
  if (kw) return _y;
  kw = 1;
  var t = CO(), r = t();
  return _y = r, _y;
}
var xy, Xw;
function NO() {
  if (Xw) return xy;
  Xw = 1;
  function t(r, i) {
    var u = r.length;
    for (r.sort(i); u--; )
      r[u] = r[u].value;
    return r;
  }
  return xy = t, xy;
}
var Sy, Iw;
function OO() {
  if (Iw) return Sy;
  Iw = 1;
  var t = bi();
  function r(i, u) {
    if (i !== u) {
      var o = i !== void 0, s = i === null, c = i === i, f = t(i), g = u !== void 0, h = u === null, v = u === u, p = t(u);
      if (!h && !p && !f && i > u || f && g && v && !h && !p || s && g && v || !o && v || !c)
        return 1;
      if (!s && !f && !p && i < u || p && o && c && !s && !f || h && o && c || !g && c || !v)
        return -1;
    }
    return 0;
  }
  return Sy = r, Sy;
}
var Ey, Qw;
function zO() {
  if (Qw) return Ey;
  Qw = 1;
  var t = OO();
  function r(i, u, o) {
    for (var s = -1, c = i.criteria, f = u.criteria, g = c.length, h = o.length; ++s < g; ) {
      var v = t(c[s], f[s]);
      if (v) {
        if (s >= h)
          return v;
        var p = o[s];
        return v * (p == "desc" ? -1 : 1);
      }
    }
    return i.index - u.index;
  }
  return Ey = r, Ey;
}
var wy, Zw;
function DO() {
  if (Zw) return wy;
  Zw = 1;
  var t = rs(), r = is(), i = er(), u = _T(), o = NO(), s = Wo(), c = zO(), f = ga(), g = st();
  function h(v, p, m) {
    p.length ? p = t(p, function(A) {
      return g(A) ? function(w) {
        return r(w, A.length === 1 ? A[0] : A);
      } : A;
    }) : p = [f];
    var b = -1;
    p = t(p, s(i));
    var _ = u(v, function(A, w, E) {
      var M = t(p, function(S) {
        return S(A);
      });
      return { criteria: M, index: ++b, value: A };
    });
    return o(_, function(A, w) {
      return c(A, w, m);
    });
  }
  return wy = h, wy;
}
var Ay, Kw;
function HO() {
  if (Kw) return Ay;
  Kw = 1;
  var t = Zp(), r = DO(), i = us(), u = ls(), o = i(function(s, c) {
    if (s == null)
      return [];
    var f = c.length;
    return f > 1 && u(s, c[0], c[1]) ? c = [] : f > 2 && u(c[0], c[1], c[2]) && (c = [c[0]]), r(s, t(c, 1), []);
  });
  return Ay = o, Ay;
}
var Ty, $w;
function LO() {
  if ($w) return Ty;
  $w = 1;
  var t = hT(), r = 0;
  function i(u) {
    var o = ++r;
    return t(u) + o;
  }
  return Ty = i, Ty;
}
var My, Fw;
function BO() {
  if (Fw) return My;
  Fw = 1;
  function t(r, i, u) {
    for (var o = -1, s = r.length, c = i.length, f = {}; ++o < s; ) {
      var g = o < c ? i[o] : void 0;
      u(f, r[o], g);
    }
    return f;
  }
  return My = t, My;
}
var qy, Jw;
function jO() {
  if (Jw) return qy;
  Jw = 1;
  var t = Jo(), r = BO();
  function i(u, o) {
    return r(u || [], o || [], t);
  }
  return qy = i, qy;
}
var Cy, Pw;
function $e() {
  if (Pw) return Cy;
  Pw = 1;
  var t;
  if (typeof Op == "function")
    try {
      t = {
        cloneDeep: eO(),
        constant: Yp(),
        defaults: tO(),
        each: uT(),
        filter: pT(),
        find: oO(),
        flatten: DT(),
        forEach: iT(),
        forIn: sO(),
        has: mT(),
        isUndefined: bT(),
        last: cO(),
        map: xT(),
        mapValues: fO(),
        max: hO(),
        merge: bO(),
        min: _O(),
        minBy: xO(),
        now: SO(),
        pick: MO(),
        range: RO(),
        reduce: ST(),
        sortBy: HO(),
        uniqueId: LO(),
        values: MT(),
        zipObject: jO()
      };
    } catch {
    }
  return t || (t = window._), Cy = t, Cy;
}
var Ry, Ww;
function UO() {
  if (Ww) return Ry;
  Ww = 1, Ry = t;
  function t() {
    var u = {};
    u._next = u._prev = u, this._sentinel = u;
  }
  t.prototype.dequeue = function() {
    var u = this._sentinel, o = u._prev;
    if (o !== u)
      return r(o), o;
  }, t.prototype.enqueue = function(u) {
    var o = this._sentinel;
    u._prev && u._next && r(u), u._next = o._next, o._next._prev = u, o._next = u, u._prev = o;
  }, t.prototype.toString = function() {
    for (var u = [], o = this._sentinel, s = o._prev; s !== o; )
      u.push(JSON.stringify(s, i)), s = s._prev;
    return "[" + u.join(", ") + "]";
  };
  function r(u) {
    u._prev._next = u._next, u._next._prev = u._prev, delete u._next, delete u._prev;
  }
  function i(u, o) {
    if (u !== "_next" && u !== "_prev")
      return o;
  }
  return Ry;
}
var Ny, e2;
function GO() {
  if (e2) return Ny;
  e2 = 1;
  var t = $e(), r = gn().Graph, i = UO();
  Ny = o;
  var u = t.constant(1);
  function o(h, v) {
    if (h.nodeCount() <= 1)
      return [];
    var p = f(h, v || u), m = s(p.graph, p.buckets, p.zeroIdx);
    return t.flatten(t.map(m, function(b) {
      return h.outEdges(b.v, b.w);
    }), !0);
  }
  function s(h, v, p) {
    for (var m = [], b = v[v.length - 1], _ = v[0], A; h.nodeCount(); ) {
      for (; A = _.dequeue(); )
        c(h, v, p, A);
      for (; A = b.dequeue(); )
        c(h, v, p, A);
      if (h.nodeCount()) {
        for (var w = v.length - 2; w > 0; --w)
          if (A = v[w].dequeue(), A) {
            m = m.concat(c(h, v, p, A, !0));
            break;
          }
      }
    }
    return m;
  }
  function c(h, v, p, m, b) {
    var _ = b ? [] : void 0;
    return t.forEach(h.inEdges(m.v), function(A) {
      var w = h.edge(A), E = h.node(A.v);
      b && _.push({ v: A.v, w: A.w }), E.out -= w, g(v, p, E);
    }), t.forEach(h.outEdges(m.v), function(A) {
      var w = h.edge(A), E = A.w, M = h.node(E);
      M.in -= w, g(v, p, M);
    }), h.removeNode(m.v), _;
  }
  function f(h, v) {
    var p = new r(), m = 0, b = 0;
    t.forEach(h.nodes(), function(w) {
      p.setNode(w, { v: w, in: 0, out: 0 });
    }), t.forEach(h.edges(), function(w) {
      var E = p.edge(w.v, w.w) || 0, M = v(w), S = E + M;
      p.setEdge(w.v, w.w, S), b = Math.max(b, p.node(w.v).out += M), m = Math.max(m, p.node(w.w).in += M);
    });
    var _ = t.range(b + m + 3).map(function() {
      return new i();
    }), A = m + 1;
    return t.forEach(p.nodes(), function(w) {
      g(_, A, p.node(w));
    }), { graph: p, buckets: _, zeroIdx: A };
  }
  function g(h, v, p) {
    p.out ? p.in ? h[p.out - p.in + v].enqueue(p) : h[h.length - 1].enqueue(p) : h[0].enqueue(p);
  }
  return Ny;
}
var Oy, t2;
function VO() {
  if (t2) return Oy;
  t2 = 1;
  var t = $e(), r = GO();
  Oy = {
    run: i,
    undo: o
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
      t.has(g, v) || (g[v] = !0, f[v] = !0, t.forEach(s.outEdges(v), function(p) {
        t.has(f, p.w) ? c.push(p) : h(p.w);
      }), delete f[v]);
    }
    return t.forEach(s.nodes(), h), c;
  }
  function o(s) {
    t.forEach(s.edges(), function(c) {
      var f = s.edge(c);
      if (f.reversed) {
        s.removeEdge(c);
        var g = f.forwardName;
        delete f.reversed, delete f.forwardName, s.setEdge(c.w, c.v, f, g);
      }
    });
  }
  return Oy;
}
var zy, n2;
function Ut() {
  if (n2) return zy;
  n2 = 1;
  var t = $e(), r = gn().Graph;
  zy = {
    addDummyNode: i,
    simplify: u,
    asNonCompoundGraph: o,
    successorWeights: s,
    predecessorWeights: c,
    intersectRect: f,
    buildLayerMatrix: g,
    normalizeRanks: h,
    removeEmptyRanks: v,
    addBorderNode: p,
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
  function o(w) {
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
    var M = w.x, S = w.y, T = E.x - M, O = E.y - S, C = w.width / 2, R = w.height / 2;
    if (!T && !O)
      throw new Error("Not possible to find intersection inside of the rectangle");
    var H, B;
    return Math.abs(O) * C > Math.abs(T) * R ? (O < 0 && (R = -R), H = R * T / O, B = R) : (T < 0 && (C = -C), H = C, B = C * O / T), { x: M + H, y: S + B };
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
    var E = t.min(t.map(w.nodes(), function(O) {
      return w.node(O).rank;
    })), M = [];
    t.forEach(w.nodes(), function(O) {
      var C = w.node(O).rank - E;
      M[C] || (M[C] = []), M[C].push(O);
    });
    var S = 0, T = w.graph().nodeRankFactor;
    t.forEach(M, function(O, C) {
      t.isUndefined(O) && C % T !== 0 ? --S : S && t.forEach(O, function(R) {
        w.node(R).rank += S;
      });
    });
  }
  function p(w, E, M, S) {
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
  return zy;
}
var Dy, r2;
function YO() {
  if (r2) return Dy;
  r2 = 1;
  var t = $e(), r = Ut();
  Dy = {
    run: i,
    undo: o
  };
  function i(s) {
    s.graph().dummyChains = [], t.forEach(s.edges(), function(c) {
      u(s, c);
    });
  }
  function u(s, c) {
    var f = c.v, g = s.node(f).rank, h = c.w, v = s.node(h).rank, p = c.name, m = s.edge(c), b = m.labelRank;
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
        }, _ = r.addDummyNode(s, "edge", A, "_d"), g === b && (A.width = m.width, A.height = m.height, A.dummy = "edge-label", A.labelpos = m.labelpos), s.setEdge(f, _, { weight: m.weight }, p), w === 0 && s.graph().dummyChains.push(_), f = _;
      s.setEdge(f, h, { weight: m.weight }, p);
    }
  }
  function o(s) {
    t.forEach(s.graph().dummyChains, function(c) {
      var f = s.node(c), g = f.edgeLabel, h;
      for (s.setEdge(f.edgeObj, g); f.dummy; )
        h = s.successors(c)[0], s.removeNode(c), g.points.push({ x: f.x, y: f.y }), f.dummy === "edge-label" && (g.x = f.x, g.y = f.y, g.width = f.width, g.height = f.height), c = h, f = s.node(c);
    });
  }
  return Dy;
}
var Hy, a2;
function zo() {
  if (a2) return Hy;
  a2 = 1;
  var t = $e();
  Hy = {
    longestPath: r,
    slack: i
  };
  function r(u) {
    var o = {};
    function s(c) {
      var f = u.node(c);
      if (t.has(o, c))
        return f.rank;
      o[c] = !0;
      var g = t.min(t.map(u.outEdges(c), function(h) {
        return s(h.w) - u.edge(h).minlen;
      }));
      return (g === Number.POSITIVE_INFINITY || // return value of _.map([]) for Lodash 3
      g === void 0 || // return value of _.map([]) for Lodash 4
      g === null) && (g = 0), f.rank = g;
    }
    t.forEach(u.sources(), s);
  }
  function i(u, o) {
    return u.node(o.w).rank - u.node(o.v).rank - u.edge(o).minlen;
  }
  return Hy;
}
var Ly, i2;
function jT() {
  if (i2) return Ly;
  i2 = 1;
  var t = $e(), r = gn().Graph, i = zo().slack;
  Ly = u;
  function u(f) {
    var g = new r({ directed: !1 }), h = f.nodes()[0], v = f.nodeCount();
    g.setNode(h, {});
    for (var p, m; o(g, f) < v; )
      p = s(g, f), m = g.hasNode(p.v) ? i(f, p) : -i(f, p), c(g, f, m);
    return g;
  }
  function o(f, g) {
    function h(v) {
      t.forEach(g.nodeEdges(v), function(p) {
        var m = p.v, b = v === m ? p.w : m;
        !f.hasNode(b) && !i(g, p) && (f.setNode(b, {}), f.setEdge(v, b, {}), h(b));
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
  return Ly;
}
var By, u2;
function kO() {
  if (u2) return By;
  u2 = 1;
  var t = $e(), r = jT(), i = zo().slack, u = zo().longestPath, o = gn().alg.preorder, s = gn().alg.postorder, c = Ut().simplify;
  By = f, f.initLowLimValues = p, f.initCutValues = g, f.calcCutValue = v, f.leaveEdge = b, f.enterEdge = _, f.exchangeEdges = A;
  function f(S) {
    S = c(S), u(S);
    var T = r(S);
    p(T), g(T, S);
    for (var O, C; O = b(T); )
      C = _(T, S, O), A(T, S, O, C);
  }
  function g(S, T) {
    var O = s(S, S.nodes());
    O = O.slice(0, O.length - 1), t.forEach(O, function(C) {
      h(S, T, C);
    });
  }
  function h(S, T, O) {
    var C = S.node(O), R = C.parent;
    S.edge(O, R).cutvalue = v(S, T, O);
  }
  function v(S, T, O) {
    var C = S.node(O), R = C.parent, H = !0, B = T.edge(O, R), X = 0;
    return B || (H = !1, B = T.edge(R, O)), X = B.weight, t.forEach(T.nodeEdges(O), function(Y) {
      var F = Y.v === O, K = F ? Y.w : Y.v;
      if (K !== R) {
        var D = F === H, G = T.edge(Y).weight;
        if (X += D ? G : -G, E(S, O, K)) {
          var N = S.edge(O, K).cutvalue;
          X += D ? -N : N;
        }
      }
    }), X;
  }
  function p(S, T) {
    arguments.length < 2 && (T = S.nodes()[0]), m(S, {}, 1, T);
  }
  function m(S, T, O, C, R) {
    var H = O, B = S.node(C);
    return T[C] = !0, t.forEach(S.neighbors(C), function(X) {
      t.has(T, X) || (O = m(S, T, O, X, C));
    }), B.low = H, B.lim = O++, R ? B.parent = R : delete B.parent, O;
  }
  function b(S) {
    return t.find(S.edges(), function(T) {
      return S.edge(T).cutvalue < 0;
    });
  }
  function _(S, T, O) {
    var C = O.v, R = O.w;
    T.hasEdge(C, R) || (C = O.w, R = O.v);
    var H = S.node(C), B = S.node(R), X = H, Y = !1;
    H.lim > B.lim && (X = B, Y = !0);
    var F = t.filter(T.edges(), function(K) {
      return Y === M(S, S.node(K.v), X) && Y !== M(S, S.node(K.w), X);
    });
    return t.minBy(F, function(K) {
      return i(T, K);
    });
  }
  function A(S, T, O, C) {
    var R = O.v, H = O.w;
    S.removeEdge(R, H), S.setEdge(C.v, C.w, {}), p(S), g(S, T), w(S, T);
  }
  function w(S, T) {
    var O = t.find(S.nodes(), function(R) {
      return !T.node(R).parent;
    }), C = o(S, O);
    C = C.slice(1), t.forEach(C, function(R) {
      var H = S.node(R).parent, B = T.edge(R, H), X = !1;
      B || (B = T.edge(H, R), X = !0), T.node(R).rank = T.node(H).rank + (X ? B.minlen : -B.minlen);
    });
  }
  function E(S, T, O) {
    return S.hasEdge(T, O);
  }
  function M(S, T, O) {
    return O.low <= T.lim && T.lim <= O.lim;
  }
  return By;
}
var jy, l2;
function XO() {
  if (l2) return jy;
  l2 = 1;
  var t = zo(), r = t.longestPath, i = jT(), u = kO();
  jy = o;
  function o(g) {
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
  return jy;
}
var Uy, o2;
function IO() {
  if (o2) return Uy;
  o2 = 1;
  var t = $e();
  Uy = r;
  function r(o) {
    var s = u(o);
    t.forEach(o.graph().dummyChains, function(c) {
      for (var f = o.node(c), g = f.edgeObj, h = i(o, s, g.v, g.w), v = h.path, p = h.lca, m = 0, b = v[m], _ = !0; c !== g.w; ) {
        if (f = o.node(c), _) {
          for (; (b = v[m]) !== p && o.node(b).maxRank < f.rank; )
            m++;
          b === p && (_ = !1);
        }
        if (!_) {
          for (; m < v.length - 1 && o.node(b = v[m + 1]).minRank <= f.rank; )
            m++;
          b = v[m];
        }
        o.setParent(c, b), c = o.successors(c)[0];
      }
    });
  }
  function i(o, s, c, f) {
    var g = [], h = [], v = Math.min(s[c].low, s[f].low), p = Math.max(s[c].lim, s[f].lim), m, b;
    m = c;
    do
      m = o.parent(m), g.push(m);
    while (m && (s[m].low > v || p > s[m].lim));
    for (b = m, m = f; (m = o.parent(m)) !== b; )
      h.push(m);
    return { path: g.concat(h.reverse()), lca: b };
  }
  function u(o) {
    var s = {}, c = 0;
    function f(g) {
      var h = c;
      t.forEach(o.children(g), f), s[g] = { low: h, lim: c++ };
    }
    return t.forEach(o.children(), f), s;
  }
  return Uy;
}
var Gy, s2;
function QO() {
  if (s2) return Gy;
  s2 = 1;
  var t = $e(), r = Ut();
  Gy = {
    run: i,
    cleanup: c
  };
  function i(f) {
    var g = r.addDummyNode(f, "root", {}, "_root"), h = o(f), v = t.max(t.values(h)) - 1, p = 2 * v + 1;
    f.graph().nestingRoot = g, t.forEach(f.edges(), function(b) {
      f.edge(b).minlen *= p;
    });
    var m = s(f) + 1;
    t.forEach(f.children(), function(b) {
      u(f, g, p, m, v, h, b);
    }), f.graph().nodeRankFactor = p;
  }
  function u(f, g, h, v, p, m, b) {
    var _ = f.children(b);
    if (!_.length) {
      b !== g && f.setEdge(g, b, { weight: 0, minlen: h });
      return;
    }
    var A = r.addBorderNode(f, "_bt"), w = r.addBorderNode(f, "_bb"), E = f.node(b);
    f.setParent(A, b), E.borderTop = A, f.setParent(w, b), E.borderBottom = w, t.forEach(_, function(M) {
      u(f, g, h, v, p, m, M);
      var S = f.node(M), T = S.borderTop ? S.borderTop : M, O = S.borderBottom ? S.borderBottom : M, C = S.borderTop ? v : 2 * v, R = T !== O ? 1 : p - m[b] + 1;
      f.setEdge(A, T, {
        weight: C,
        minlen: R,
        nestingEdge: !0
      }), f.setEdge(O, w, {
        weight: C,
        minlen: R,
        nestingEdge: !0
      });
    }), f.parent(b) || f.setEdge(g, A, { weight: 0, minlen: p + m[b] });
  }
  function o(f) {
    var g = {};
    function h(v, p) {
      var m = f.children(v);
      m && m.length && t.forEach(m, function(b) {
        h(b, p + 1);
      }), g[v] = p;
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
  return Gy;
}
var Vy, c2;
function ZO() {
  if (c2) return Vy;
  c2 = 1;
  var t = $e(), r = Ut();
  Vy = i;
  function i(o) {
    function s(c) {
      var f = o.children(c), g = o.node(c);
      if (f.length && t.forEach(f, s), t.has(g, "minRank")) {
        g.borderLeft = [], g.borderRight = [];
        for (var h = g.minRank, v = g.maxRank + 1; h < v; ++h)
          u(o, "borderLeft", "_bl", c, g, h), u(o, "borderRight", "_br", c, g, h);
      }
    }
    t.forEach(o.children(), s);
  }
  function u(o, s, c, f, g, h) {
    var v = { width: 0, height: 0, rank: h, borderType: s }, p = g[s][h - 1], m = r.addDummyNode(o, "border", v, c);
    g[s][h] = m, o.setParent(m, f), p && o.setEdge(p, m, { weight: 1 });
  }
  return Vy;
}
var Yy, f2;
function KO() {
  if (f2) return Yy;
  f2 = 1;
  var t = $e();
  Yy = {
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
      o(h.node(v));
    }), t.forEach(h.edges(), function(v) {
      o(h.edge(v));
    });
  }
  function o(h) {
    var v = h.width;
    h.width = h.height, h.height = v;
  }
  function s(h) {
    t.forEach(h.nodes(), function(v) {
      c(h.node(v));
    }), t.forEach(h.edges(), function(v) {
      var p = h.edge(v);
      t.forEach(p.points, c), t.has(p, "y") && c(p);
    });
  }
  function c(h) {
    h.y = -h.y;
  }
  function f(h) {
    t.forEach(h.nodes(), function(v) {
      g(h.node(v));
    }), t.forEach(h.edges(), function(v) {
      var p = h.edge(v);
      t.forEach(p.points, g), t.has(p, "x") && g(p);
    });
  }
  function g(h) {
    var v = h.x;
    h.x = h.y, h.y = v;
  }
  return Yy;
}
var ky, d2;
function $O() {
  if (d2) return ky;
  d2 = 1;
  var t = $e();
  ky = r;
  function r(i) {
    var u = {}, o = t.filter(i.nodes(), function(h) {
      return !i.children(h).length;
    }), s = t.max(t.map(o, function(h) {
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
    var g = t.sortBy(o, function(h) {
      return i.node(h).rank;
    });
    return t.forEach(g, f), c;
  }
  return ky;
}
var Xy, h2;
function FO() {
  if (h2) return Xy;
  h2 = 1;
  var t = $e();
  Xy = r;
  function r(u, o) {
    for (var s = 0, c = 1; c < o.length; ++c)
      s += i(u, o[c - 1], o[c]);
    return s;
  }
  function i(u, o, s) {
    for (var c = t.zipObject(
      s,
      t.map(s, function(m, b) {
        return b;
      })
    ), f = t.flatten(t.map(o, function(m) {
      return t.sortBy(t.map(u.outEdges(m), function(b) {
        return { pos: c[b.w], weight: u.edge(b).weight };
      }), "pos");
    }), !0), g = 1; g < s.length; ) g <<= 1;
    var h = 2 * g - 1;
    g -= 1;
    var v = t.map(new Array(h), function() {
      return 0;
    }), p = 0;
    return t.forEach(f.forEach(function(m) {
      var b = m.pos + g;
      v[b] += m.weight;
      for (var _ = 0; b > 0; )
        b % 2 && (_ += v[b + 1]), b = b - 1 >> 1, v[b] += m.weight;
      p += m.weight * _;
    })), p;
  }
  return Xy;
}
var Iy, g2;
function JO() {
  if (g2) return Iy;
  g2 = 1;
  var t = $e();
  Iy = r;
  function r(i, u) {
    return t.map(u, function(o) {
      var s = i.inEdges(o);
      if (s.length) {
        var c = t.reduce(s, function(f, g) {
          var h = i.edge(g), v = i.node(g.v);
          return {
            sum: f.sum + h.weight * v.order,
            weight: f.weight + h.weight
          };
        }, { sum: 0, weight: 0 });
        return {
          v: o,
          barycenter: c.sum / c.weight,
          weight: c.weight
        };
      } else
        return { v: o };
    });
  }
  return Iy;
}
var Qy, v2;
function PO() {
  if (v2) return Qy;
  v2 = 1;
  var t = $e();
  Qy = r;
  function r(o, s) {
    var c = {};
    t.forEach(o, function(g, h) {
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
  function i(o) {
    var s = [];
    function c(h) {
      return function(v) {
        v.merged || (t.isUndefined(v.barycenter) || t.isUndefined(h.barycenter) || v.barycenter >= h.barycenter) && u(h, v);
      };
    }
    function f(h) {
      return function(v) {
        v.in.push(h), --v.indegree === 0 && o.push(v);
      };
    }
    for (; o.length; ) {
      var g = o.pop();
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
  function u(o, s) {
    var c = 0, f = 0;
    o.weight && (c += o.barycenter * o.weight, f += o.weight), s.weight && (c += s.barycenter * s.weight, f += s.weight), o.vs = s.vs.concat(o.vs), o.barycenter = c / f, o.weight = f, o.i = Math.min(s.i, o.i), s.merged = !0;
  }
  return Qy;
}
var Zy, y2;
function WO() {
  if (y2) return Zy;
  y2 = 1;
  var t = $e(), r = Ut();
  Zy = i;
  function i(s, c) {
    var f = r.partition(s, function(A) {
      return t.has(A, "barycenter");
    }), g = f.lhs, h = t.sortBy(f.rhs, function(A) {
      return -A.i;
    }), v = [], p = 0, m = 0, b = 0;
    g.sort(o(!!c)), b = u(v, h, b), t.forEach(g, function(A) {
      b += A.vs.length, v.push(A.vs), p += A.barycenter * A.weight, m += A.weight, b = u(v, h, b);
    });
    var _ = { vs: t.flatten(v, !0) };
    return m && (_.barycenter = p / m, _.weight = m), _;
  }
  function u(s, c, f) {
    for (var g; c.length && (g = t.last(c)).i <= f; )
      c.pop(), s.push(g.vs), f++;
    return f;
  }
  function o(s) {
    return function(c, f) {
      return c.barycenter < f.barycenter ? -1 : c.barycenter > f.barycenter ? 1 : s ? f.i - c.i : c.i - f.i;
    };
  }
  return Zy;
}
var Ky, p2;
function ez() {
  if (p2) return Ky;
  p2 = 1;
  var t = $e(), r = JO(), i = PO(), u = WO();
  Ky = o;
  function o(f, g, h, v) {
    var p = f.children(g), m = f.node(g), b = m ? m.borderLeft : void 0, _ = m ? m.borderRight : void 0, A = {};
    b && (p = t.filter(p, function(O) {
      return O !== b && O !== _;
    }));
    var w = r(f, p);
    t.forEach(w, function(O) {
      if (f.children(O.v).length) {
        var C = o(f, O.v, h, v);
        A[O.v] = C, t.has(C, "barycenter") && c(O, C);
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
  return Ky;
}
var $y, m2;
function tz() {
  if (m2) return $y;
  m2 = 1;
  var t = $e(), r = gn().Graph;
  $y = i;
  function i(o, s, c) {
    var f = u(o), g = new r({ compound: !0 }).setGraph({ root: f }).setDefaultNodeLabel(function(h) {
      return o.node(h);
    });
    return t.forEach(o.nodes(), function(h) {
      var v = o.node(h), p = o.parent(h);
      (v.rank === s || v.minRank <= s && s <= v.maxRank) && (g.setNode(h), g.setParent(h, p || f), t.forEach(o[c](h), function(m) {
        var b = m.v === h ? m.w : m.v, _ = g.edge(b, h), A = t.isUndefined(_) ? 0 : _.weight;
        g.setEdge(b, h, { weight: o.edge(m).weight + A });
      }), t.has(v, "minRank") && g.setNode(h, {
        borderLeft: v.borderLeft[s],
        borderRight: v.borderRight[s]
      }));
    }), g;
  }
  function u(o) {
    for (var s; o.hasNode(s = t.uniqueId("_root")); ) ;
    return s;
  }
  return $y;
}
var Fy, b2;
function nz() {
  if (b2) return Fy;
  b2 = 1;
  var t = $e();
  Fy = r;
  function r(i, u, o) {
    var s = {}, c;
    t.forEach(o, function(f) {
      for (var g = i.parent(f), h, v; g; ) {
        if (h = i.parent(g), h ? (v = s[h], s[h] = g) : (v = c, c = g), v && v !== g) {
          u.setEdge(v, g);
          return;
        }
        g = h;
      }
    });
  }
  return Fy;
}
var Jy, _2;
function rz() {
  if (_2) return Jy;
  _2 = 1;
  var t = $e(), r = $O(), i = FO(), u = ez(), o = tz(), s = nz(), c = gn().Graph, f = Ut();
  Jy = g;
  function g(m) {
    var b = f.maxRank(m), _ = h(m, t.range(1, b + 1), "inEdges"), A = h(m, t.range(b - 1, -1, -1), "outEdges"), w = r(m);
    p(m, w);
    for (var E = Number.POSITIVE_INFINITY, M, S = 0, T = 0; T < 4; ++S, ++T) {
      v(S % 2 ? _ : A, S % 4 >= 2), w = f.buildLayerMatrix(m);
      var O = i(m, w);
      O < E && (T = 0, M = t.cloneDeep(w), E = O);
    }
    p(m, M);
  }
  function h(m, b, _) {
    return t.map(b, function(A) {
      return o(m, A, _);
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
  function p(m, b) {
    t.forEach(b, function(_) {
      t.forEach(_, function(A, w) {
        m.node(A).order = w;
      });
    });
  }
  return Jy;
}
var Py, x2;
function az() {
  if (x2) return Py;
  x2 = 1;
  var t = $e(), r = gn().Graph, i = Ut();
  Py = {
    positionX: _,
    findType1Conflicts: u,
    findType2Conflicts: o,
    addConflict: c,
    hasConflict: f,
    verticalAlignment: g,
    horizontalCompaction: h,
    alignCoordinates: m,
    findSmallestWidthAlignment: p,
    balance: b
  };
  function u(E, M) {
    var S = {};
    function T(O, C) {
      var R = 0, H = 0, B = O.length, X = t.last(C);
      return t.forEach(C, function(Y, F) {
        var K = s(E, Y), D = K ? E.node(K).order : B;
        (K || Y === X) && (t.forEach(C.slice(H, F + 1), function(G) {
          t.forEach(E.predecessors(G), function(N) {
            var j = E.node(N), Z = j.order;
            (Z < R || D < Z) && !(j.dummy && E.node(G).dummy) && c(S, N, G);
          });
        }), H = F + 1, R = D);
      }), C;
    }
    return t.reduce(M, T), S;
  }
  function o(E, M) {
    var S = {};
    function T(C, R, H, B, X) {
      var Y;
      t.forEach(t.range(R, H), function(F) {
        Y = C[F], E.node(Y).dummy && t.forEach(E.predecessors(Y), function(K) {
          var D = E.node(K);
          D.dummy && (D.order < B || D.order > X) && c(S, K, Y);
        });
      });
    }
    function O(C, R) {
      var H = -1, B, X = 0;
      return t.forEach(R, function(Y, F) {
        if (E.node(Y).dummy === "border") {
          var K = E.predecessors(Y);
          K.length && (B = E.node(K[0]).order, T(R, X, F, H, B), X = F, H = B);
        }
        T(R, X, R.length, B, C.length);
      }), R;
    }
    return t.reduce(M, O), S;
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
    var O = E[M];
    O || (E[M] = O = {}), O[S] = !0;
  }
  function f(E, M, S) {
    if (M > S) {
      var T = M;
      M = S, S = T;
    }
    return t.has(E[M], S);
  }
  function g(E, M, S, T) {
    var O = {}, C = {}, R = {};
    return t.forEach(M, function(H) {
      t.forEach(H, function(B, X) {
        O[B] = B, C[B] = B, R[B] = X;
      });
    }), t.forEach(M, function(H) {
      var B = -1;
      t.forEach(H, function(X) {
        var Y = T(X);
        if (Y.length) {
          Y = t.sortBy(Y, function(N) {
            return R[N];
          });
          for (var F = (Y.length - 1) / 2, K = Math.floor(F), D = Math.ceil(F); K <= D; ++K) {
            var G = Y[K];
            C[X] === X && B < R[G] && !f(S, X, G) && (C[G] = X, C[X] = O[X] = O[G], B = R[G]);
          }
        }
      });
    }), { root: O, align: C };
  }
  function h(E, M, S, T, O) {
    var C = {}, R = v(E, M, S, O), H = O ? "borderLeft" : "borderRight";
    function B(F, K) {
      for (var D = R.nodes(), G = D.pop(), N = {}; G; )
        N[G] ? F(G) : (N[G] = !0, D.push(G), D = D.concat(K(G))), G = D.pop();
    }
    function X(F) {
      C[F] = R.inEdges(F).reduce(function(K, D) {
        return Math.max(K, C[D.v] + R.edge(D));
      }, 0);
    }
    function Y(F) {
      var K = R.outEdges(F).reduce(function(G, N) {
        return Math.min(G, C[N.w] - R.edge(N));
      }, Number.POSITIVE_INFINITY), D = E.node(F);
      K !== Number.POSITIVE_INFINITY && D.borderType !== H && (C[F] = Math.max(C[F], K));
    }
    return B(X, R.predecessors.bind(R)), B(Y, R.successors.bind(R)), t.forEach(T, function(F) {
      C[F] = C[S[F]];
    }), C;
  }
  function v(E, M, S, T) {
    var O = new r(), C = E.graph(), R = A(C.nodesep, C.edgesep, T);
    return t.forEach(M, function(H) {
      var B;
      t.forEach(H, function(X) {
        var Y = S[X];
        if (O.setNode(Y), B) {
          var F = S[B], K = O.edge(F, Y);
          O.setEdge(F, Y, Math.max(R(E, X, B), K || 0));
        }
        B = X;
      });
    }), O;
  }
  function p(E, M) {
    return t.minBy(t.values(M), function(S) {
      var T = Number.NEGATIVE_INFINITY, O = Number.POSITIVE_INFINITY;
      return t.forIn(S, function(C, R) {
        var H = w(E, R) / 2;
        T = Math.max(C + H, T), O = Math.min(C - H, O);
      }), T - O;
    });
  }
  function m(E, M) {
    var S = t.values(M), T = t.min(S), O = t.max(S);
    t.forEach(["u", "d"], function(C) {
      t.forEach(["l", "r"], function(R) {
        var H = C + R, B = E[H], X;
        if (B !== M) {
          var Y = t.values(B);
          X = R === "l" ? T - t.min(Y) : O - t.max(Y), X && (E[H] = t.mapValues(B, function(F) {
            return F + X;
          }));
        }
      });
    });
  }
  function b(E, M) {
    return t.mapValues(E.ul, function(S, T) {
      if (M)
        return E[M.toLowerCase()][T];
      var O = t.sortBy(t.map(E, T));
      return (O[1] + O[2]) / 2;
    });
  }
  function _(E) {
    var M = i.buildLayerMatrix(E), S = t.merge(
      u(E, M),
      o(E, M)
    ), T = {}, O;
    t.forEach(["u", "d"], function(R) {
      O = R === "u" ? M : t.values(M).reverse(), t.forEach(["l", "r"], function(H) {
        H === "r" && (O = t.map(O, function(F) {
          return t.values(F).reverse();
        }));
        var B = (R === "u" ? E.predecessors : E.successors).bind(E), X = g(E, O, S, B), Y = h(
          E,
          O,
          X.root,
          X.align,
          H === "r"
        );
        H === "r" && (Y = t.mapValues(Y, function(F) {
          return -F;
        })), T[R + H] = Y;
      });
    });
    var C = p(E, T);
    return m(T, C), b(T, E.graph().align);
  }
  function A(E, M, S) {
    return function(T, O, C) {
      var R = T.node(O), H = T.node(C), B = 0, X;
      if (B += R.width / 2, t.has(R, "labelpos"))
        switch (R.labelpos.toLowerCase()) {
          case "l":
            X = -R.width / 2;
            break;
          case "r":
            X = R.width / 2;
            break;
        }
      if (X && (B += S ? X : -X), X = 0, B += (R.dummy ? M : E) / 2, B += (H.dummy ? M : E) / 2, B += H.width / 2, t.has(H, "labelpos"))
        switch (H.labelpos.toLowerCase()) {
          case "l":
            X = H.width / 2;
            break;
          case "r":
            X = -H.width / 2;
            break;
        }
      return X && (B += S ? X : -X), X = 0, B;
    };
  }
  function w(E, M) {
    return E.node(M).width;
  }
  return Py;
}
var Wy, S2;
function iz() {
  if (S2) return Wy;
  S2 = 1;
  var t = $e(), r = Ut(), i = az().positionX;
  Wy = u;
  function u(s) {
    s = r.asNonCompoundGraph(s), o(s), t.forEach(i(s), function(c, f) {
      s.node(f).x = c;
    });
  }
  function o(s) {
    var c = r.buildLayerMatrix(s), f = s.graph().ranksep, g = 0;
    t.forEach(c, function(h) {
      var v = t.max(t.map(h, function(p) {
        return s.node(p).height;
      }));
      t.forEach(h, function(p) {
        s.node(p).y = g + v / 2;
      }), g += v + f;
    });
  }
  return Wy;
}
var ep, E2;
function uz() {
  if (E2) return ep;
  E2 = 1;
  var t = $e(), r = VO(), i = YO(), u = XO(), o = Ut().normalizeRanks, s = IO(), c = Ut().removeEmptyRanks, f = QO(), g = ZO(), h = KO(), v = rz(), p = iz(), m = Ut(), b = gn().Graph;
  ep = _;
  function _(L, I) {
    var P = I && I.debugTiming ? m.time : m.notime;
    P("layout", function() {
      var ae = P("  buildLayoutGraph", function() {
        return B(L);
      });
      P("  runLayout", function() {
        A(ae, P);
      }), P("  updateInputGraph", function() {
        w(L, ae);
      });
    });
  }
  function A(L, I) {
    I("    makeSpaceForEdgeLabels", function() {
      X(L);
    }), I("    removeSelfEdges", function() {
      Q(L);
    }), I("    acyclic", function() {
      r.run(L);
    }), I("    nestingGraph.run", function() {
      f.run(L);
    }), I("    rank", function() {
      u(m.asNonCompoundGraph(L));
    }), I("    injectEdgeLabelProxies", function() {
      Y(L);
    }), I("    removeEmptyRanks", function() {
      c(L);
    }), I("    nestingGraph.cleanup", function() {
      f.cleanup(L);
    }), I("    normalizeRanks", function() {
      o(L);
    }), I("    assignRankMinMax", function() {
      F(L);
    }), I("    removeEdgeLabelProxies", function() {
      K(L);
    }), I("    normalize.run", function() {
      i.run(L);
    }), I("    parentDummyChains", function() {
      s(L);
    }), I("    addBorderSegments", function() {
      g(L);
    }), I("    order", function() {
      v(L);
    }), I("    insertSelfEdges", function() {
      le(L);
    }), I("    adjustCoordinateSystem", function() {
      h.adjust(L);
    }), I("    position", function() {
      p(L);
    }), I("    positionSelfEdges", function() {
      z(L);
    }), I("    removeBorderNodes", function() {
      Z(L);
    }), I("    normalize.undo", function() {
      i.undo(L);
    }), I("    fixupEdgeLabelCoords", function() {
      N(L);
    }), I("    undoCoordinateSystem", function() {
      h.undo(L);
    }), I("    translateGraph", function() {
      D(L);
    }), I("    assignNodeIntersects", function() {
      G(L);
    }), I("    reversePoints", function() {
      j(L);
    }), I("    acyclic.undo", function() {
      r.undo(L);
    });
  }
  function w(L, I) {
    t.forEach(L.nodes(), function(P) {
      var ae = L.node(P), W = I.node(P);
      ae && (ae.x = W.x, ae.y = W.y, I.children(P).length && (ae.width = W.width, ae.height = W.height));
    }), t.forEach(L.edges(), function(P) {
      var ae = L.edge(P), W = I.edge(P);
      ae.points = W.points, t.has(W, "x") && (ae.x = W.x, ae.y = W.y);
    }), L.graph().width = I.graph().width, L.graph().height = I.graph().height;
  }
  var E = ["nodesep", "edgesep", "ranksep", "marginx", "marginy"], M = { ranksep: 50, edgesep: 20, nodesep: 50, rankdir: "tb" }, S = ["acyclicer", "ranker", "rankdir", "align"], T = ["width", "height"], O = { width: 0, height: 0 }, C = ["minlen", "weight", "width", "height", "labeloffset"], R = {
    minlen: 1,
    weight: 1,
    width: 0,
    height: 0,
    labeloffset: 10,
    labelpos: "r"
  }, H = ["labelpos"];
  function B(L) {
    var I = new b({ multigraph: !0, compound: !0 }), P = ie(L.graph());
    return I.setGraph(t.merge(
      {},
      M,
      V(P, E),
      t.pick(P, S)
    )), t.forEach(L.nodes(), function(ae) {
      var W = ie(L.node(ae));
      I.setNode(ae, t.defaults(V(W, T), O)), I.setParent(ae, L.parent(ae));
    }), t.forEach(L.edges(), function(ae) {
      var W = ie(L.edge(ae));
      I.setEdge(ae, t.merge(
        {},
        R,
        V(W, C),
        t.pick(W, H)
      ));
    }), I;
  }
  function X(L) {
    var I = L.graph();
    I.ranksep /= 2, t.forEach(L.edges(), function(P) {
      var ae = L.edge(P);
      ae.minlen *= 2, ae.labelpos.toLowerCase() !== "c" && (I.rankdir === "TB" || I.rankdir === "BT" ? ae.width += ae.labeloffset : ae.height += ae.labeloffset);
    });
  }
  function Y(L) {
    t.forEach(L.edges(), function(I) {
      var P = L.edge(I);
      if (P.width && P.height) {
        var ae = L.node(I.v), W = L.node(I.w), se = { rank: (W.rank - ae.rank) / 2 + ae.rank, e: I };
        m.addDummyNode(L, "edge-proxy", se, "_ep");
      }
    });
  }
  function F(L) {
    var I = 0;
    t.forEach(L.nodes(), function(P) {
      var ae = L.node(P);
      ae.borderTop && (ae.minRank = L.node(ae.borderTop).rank, ae.maxRank = L.node(ae.borderBottom).rank, I = t.max(I, ae.maxRank));
    }), L.graph().maxRank = I;
  }
  function K(L) {
    t.forEach(L.nodes(), function(I) {
      var P = L.node(I);
      P.dummy === "edge-proxy" && (L.edge(P.e).labelRank = P.rank, L.removeNode(I));
    });
  }
  function D(L) {
    var I = Number.POSITIVE_INFINITY, P = 0, ae = Number.POSITIVE_INFINITY, W = 0, se = L.graph(), de = se.marginx || 0, ve = se.marginy || 0;
    function pe(he) {
      var me = he.x, ge = he.y, Ae = he.width, xe = he.height;
      I = Math.min(I, me - Ae / 2), P = Math.max(P, me + Ae / 2), ae = Math.min(ae, ge - xe / 2), W = Math.max(W, ge + xe / 2);
    }
    t.forEach(L.nodes(), function(he) {
      pe(L.node(he));
    }), t.forEach(L.edges(), function(he) {
      var me = L.edge(he);
      t.has(me, "x") && pe(me);
    }), I -= de, ae -= ve, t.forEach(L.nodes(), function(he) {
      var me = L.node(he);
      me.x -= I, me.y -= ae;
    }), t.forEach(L.edges(), function(he) {
      var me = L.edge(he);
      t.forEach(me.points, function(ge) {
        ge.x -= I, ge.y -= ae;
      }), t.has(me, "x") && (me.x -= I), t.has(me, "y") && (me.y -= ae);
    }), se.width = P - I + de, se.height = W - ae + ve;
  }
  function G(L) {
    t.forEach(L.edges(), function(I) {
      var P = L.edge(I), ae = L.node(I.v), W = L.node(I.w), se, de;
      P.points ? (se = P.points[0], de = P.points[P.points.length - 1]) : (P.points = [], se = W, de = ae), P.points.unshift(m.intersectRect(ae, se)), P.points.push(m.intersectRect(W, de));
    });
  }
  function N(L) {
    t.forEach(L.edges(), function(I) {
      var P = L.edge(I);
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
  function j(L) {
    t.forEach(L.edges(), function(I) {
      var P = L.edge(I);
      P.reversed && P.points.reverse();
    });
  }
  function Z(L) {
    t.forEach(L.nodes(), function(I) {
      if (L.children(I).length) {
        var P = L.node(I), ae = L.node(P.borderTop), W = L.node(P.borderBottom), se = L.node(t.last(P.borderLeft)), de = L.node(t.last(P.borderRight));
        P.width = Math.abs(de.x - se.x), P.height = Math.abs(W.y - ae.y), P.x = se.x + P.width / 2, P.y = ae.y + P.height / 2;
      }
    }), t.forEach(L.nodes(), function(I) {
      L.node(I).dummy === "border" && L.removeNode(I);
    });
  }
  function Q(L) {
    t.forEach(L.edges(), function(I) {
      if (I.v === I.w) {
        var P = L.node(I.v);
        P.selfEdges || (P.selfEdges = []), P.selfEdges.push({ e: I, label: L.edge(I) }), L.removeEdge(I);
      }
    });
  }
  function le(L) {
    var I = m.buildLayerMatrix(L);
    t.forEach(I, function(P) {
      var ae = 0;
      t.forEach(P, function(W, se) {
        var de = L.node(W);
        de.order = se + ae, t.forEach(de.selfEdges, function(ve) {
          m.addDummyNode(L, "selfedge", {
            width: ve.label.width,
            height: ve.label.height,
            rank: de.rank,
            order: se + ++ae,
            e: ve.e,
            label: ve.label
          }, "_se");
        }), delete de.selfEdges;
      });
    });
  }
  function z(L) {
    t.forEach(L.nodes(), function(I) {
      var P = L.node(I);
      if (P.dummy === "selfedge") {
        var ae = L.node(P.e.v), W = ae.x + ae.width / 2, se = ae.y, de = P.x - W, ve = ae.height / 2;
        L.setEdge(P.e, P.label), L.removeNode(I), P.label.points = [
          { x: W + 2 * de / 3, y: se - ve },
          { x: W + 5 * de / 6, y: se - ve },
          { x: W + de, y: se },
          { x: W + 5 * de / 6, y: se + ve },
          { x: W + 2 * de / 3, y: se + ve }
        ], P.label.x = P.x, P.label.y = P.y;
      }
    });
  }
  function V(L, I) {
    return t.mapValues(t.pick(L, I), Number);
  }
  function ie(L) {
    var I = {};
    return t.forEach(L, function(P, ae) {
      I[ae.toLowerCase()] = P;
    }), I;
  }
  return ep;
}
var tp, w2;
function lz() {
  if (w2) return tp;
  w2 = 1;
  var t = $e(), r = Ut(), i = gn().Graph;
  tp = {
    debugOrdering: u
  };
  function u(o) {
    var s = r.buildLayerMatrix(o), c = new i({ compound: !0, multigraph: !0 }).setGraph({});
    return t.forEach(o.nodes(), function(f) {
      c.setNode(f, { label: f }), c.setParent(f, "layer" + o.node(f).rank);
    }), t.forEach(o.edges(), function(f) {
      c.setEdge(f.v, f.w, {}, f.name);
    }), t.forEach(s, function(f, g) {
      var h = "layer" + g;
      c.setNode(h, { rank: "same" }), t.reduce(f, function(v, p) {
        return c.setEdge(v, p, { style: "invis" }), p;
      });
    }), c;
  }
  return tp;
}
var np, A2;
function oz() {
  return A2 || (A2 = 1, np = "0.8.5"), np;
}
var rp, T2;
function sz() {
  return T2 || (T2 = 1, rp = {
    graphlib: gn(),
    layout: uz(),
    debug: lz(),
    util: {
      time: Ut().time,
      notime: Ut().notime
    },
    version: oz()
  }), rp;
}
var cz = sz();
const M2 = /* @__PURE__ */ Np(cz);
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
var fz = { value: () => {
} };
function os() {
  for (var t = 0, r = arguments.length, i = {}, u; t < r; ++t) {
    if (!(u = arguments[t] + "") || u in i || /[\s.]/.test(u)) throw new Error("illegal type: " + u);
    i[u] = [];
  }
  return new qo(i);
}
function qo(t) {
  this._ = t;
}
function dz(t, r) {
  return t.trim().split(/^|\s+/).map(function(i) {
    var u = "", o = i.indexOf(".");
    if (o >= 0 && (u = i.slice(o + 1), i = i.slice(0, o)), i && !r.hasOwnProperty(i)) throw new Error("unknown type: " + i);
    return { type: i, name: u };
  });
}
qo.prototype = os.prototype = {
  constructor: qo,
  on: function(t, r) {
    var i = this._, u = dz(t + "", i), o, s = -1, c = u.length;
    if (arguments.length < 2) {
      for (; ++s < c; ) if ((o = (t = u[s]).type) && (o = hz(i[o], t.name))) return o;
      return;
    }
    if (r != null && typeof r != "function") throw new Error("invalid callback: " + r);
    for (; ++s < c; )
      if (o = (t = u[s]).type) i[o] = q2(i[o], t.name, r);
      else if (r == null) for (o in i) i[o] = q2(i[o], t.name, null);
    return this;
  },
  copy: function() {
    var t = {}, r = this._;
    for (var i in r) t[i] = r[i].slice();
    return new qo(t);
  },
  call: function(t, r) {
    if ((o = arguments.length - 2) > 0) for (var i = new Array(o), u = 0, o, s; u < o; ++u) i[u] = arguments[u + 2];
    if (!this._.hasOwnProperty(t)) throw new Error("unknown type: " + t);
    for (s = this._[t], u = 0, o = s.length; u < o; ++u) s[u].value.apply(r, i);
  },
  apply: function(t, r, i) {
    if (!this._.hasOwnProperty(t)) throw new Error("unknown type: " + t);
    for (var u = this._[t], o = 0, s = u.length; o < s; ++o) u[o].value.apply(r, i);
  }
};
function hz(t, r) {
  for (var i = 0, u = t.length, o; i < u; ++i)
    if ((o = t[i]).name === r)
      return o.value;
}
function q2(t, r, i) {
  for (var u = 0, o = t.length; u < o; ++u)
    if (t[u].name === r) {
      t[u] = fz, t = t.slice(0, u).concat(t.slice(u + 1));
      break;
    }
  return i != null && t.push({ name: r, value: i }), t;
}
var pp = "http://www.w3.org/1999/xhtml";
const C2 = {
  svg: "http://www.w3.org/2000/svg",
  xhtml: pp,
  xlink: "http://www.w3.org/1999/xlink",
  xml: "http://www.w3.org/XML/1998/namespace",
  xmlns: "http://www.w3.org/2000/xmlns/"
};
function ss(t) {
  var r = t += "", i = r.indexOf(":");
  return i >= 0 && (r = t.slice(0, i)) !== "xmlns" && (t = t.slice(i + 1)), C2.hasOwnProperty(r) ? { space: C2[r], local: t } : t;
}
function gz(t) {
  return function() {
    var r = this.ownerDocument, i = this.namespaceURI;
    return i === pp && r.documentElement.namespaceURI === pp ? r.createElement(t) : r.createElementNS(i, t);
  };
}
function vz(t) {
  return function() {
    return this.ownerDocument.createElementNS(t.space, t.local);
  };
}
function UT(t) {
  var r = ss(t);
  return (r.local ? vz : gz)(r);
}
function yz() {
}
function Fp(t) {
  return t == null ? yz : function() {
    return this.querySelector(t);
  };
}
function pz(t) {
  typeof t != "function" && (t = Fp(t));
  for (var r = this._groups, i = r.length, u = new Array(i), o = 0; o < i; ++o)
    for (var s = r[o], c = s.length, f = u[o] = new Array(c), g, h, v = 0; v < c; ++v)
      (g = s[v]) && (h = t.call(g, g.__data__, v, s)) && ("__data__" in g && (h.__data__ = g.__data__), f[v] = h);
  return new Kt(u, this._parents);
}
function mz(t) {
  return t == null ? [] : Array.isArray(t) ? t : Array.from(t);
}
function bz() {
  return [];
}
function GT(t) {
  return t == null ? bz : function() {
    return this.querySelectorAll(t);
  };
}
function _z(t) {
  return function() {
    return mz(t.apply(this, arguments));
  };
}
function xz(t) {
  typeof t == "function" ? t = _z(t) : t = GT(t);
  for (var r = this._groups, i = r.length, u = [], o = [], s = 0; s < i; ++s)
    for (var c = r[s], f = c.length, g, h = 0; h < f; ++h)
      (g = c[h]) && (u.push(t.call(g, g.__data__, h, c)), o.push(g));
  return new Kt(u, o);
}
function VT(t) {
  return function() {
    return this.matches(t);
  };
}
function YT(t) {
  return function(r) {
    return r.matches(t);
  };
}
var Sz = Array.prototype.find;
function Ez(t) {
  return function() {
    return Sz.call(this.children, t);
  };
}
function wz() {
  return this.firstElementChild;
}
function Az(t) {
  return this.select(t == null ? wz : Ez(typeof t == "function" ? t : YT(t)));
}
var Tz = Array.prototype.filter;
function Mz() {
  return Array.from(this.children);
}
function qz(t) {
  return function() {
    return Tz.call(this.children, t);
  };
}
function Cz(t) {
  return this.selectAll(t == null ? Mz : qz(typeof t == "function" ? t : YT(t)));
}
function Rz(t) {
  typeof t != "function" && (t = VT(t));
  for (var r = this._groups, i = r.length, u = new Array(i), o = 0; o < i; ++o)
    for (var s = r[o], c = s.length, f = u[o] = [], g, h = 0; h < c; ++h)
      (g = s[h]) && t.call(g, g.__data__, h, s) && f.push(g);
  return new Kt(u, this._parents);
}
function kT(t) {
  return new Array(t.length);
}
function Nz() {
  return new Kt(this._enter || this._groups.map(kT), this._parents);
}
function Do(t, r) {
  this.ownerDocument = t.ownerDocument, this.namespaceURI = t.namespaceURI, this._next = null, this._parent = t, this.__data__ = r;
}
Do.prototype = {
  constructor: Do,
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
function Oz(t) {
  return function() {
    return t;
  };
}
function zz(t, r, i, u, o, s) {
  for (var c = 0, f, g = r.length, h = s.length; c < h; ++c)
    (f = r[c]) ? (f.__data__ = s[c], u[c] = f) : i[c] = new Do(t, s[c]);
  for (; c < g; ++c)
    (f = r[c]) && (o[c] = f);
}
function Dz(t, r, i, u, o, s, c) {
  var f, g, h = /* @__PURE__ */ new Map(), v = r.length, p = s.length, m = new Array(v), b;
  for (f = 0; f < v; ++f)
    (g = r[f]) && (m[f] = b = c.call(g, g.__data__, f, r) + "", h.has(b) ? o[f] = g : h.set(b, g));
  for (f = 0; f < p; ++f)
    b = c.call(t, s[f], f, s) + "", (g = h.get(b)) ? (u[f] = g, g.__data__ = s[f], h.delete(b)) : i[f] = new Do(t, s[f]);
  for (f = 0; f < v; ++f)
    (g = r[f]) && h.get(m[f]) === g && (o[f] = g);
}
function Hz(t) {
  return t.__data__;
}
function Lz(t, r) {
  if (!arguments.length) return Array.from(this, Hz);
  var i = r ? Dz : zz, u = this._parents, o = this._groups;
  typeof t != "function" && (t = Oz(t));
  for (var s = o.length, c = new Array(s), f = new Array(s), g = new Array(s), h = 0; h < s; ++h) {
    var v = u[h], p = o[h], m = p.length, b = Bz(t.call(v, v && v.__data__, h, u)), _ = b.length, A = f[h] = new Array(_), w = c[h] = new Array(_), E = g[h] = new Array(m);
    i(v, p, A, w, E, b, r);
    for (var M = 0, S = 0, T, O; M < _; ++M)
      if (T = A[M]) {
        for (M >= S && (S = M + 1); !(O = w[S]) && ++S < _; ) ;
        T._next = O || null;
      }
  }
  return c = new Kt(c, u), c._enter = f, c._exit = g, c;
}
function Bz(t) {
  return typeof t == "object" && "length" in t ? t : Array.from(t);
}
function jz() {
  return new Kt(this._exit || this._groups.map(kT), this._parents);
}
function Uz(t, r, i) {
  var u = this.enter(), o = this, s = this.exit();
  return typeof t == "function" ? (u = t(u), u && (u = u.selection())) : u = u.append(t + ""), r != null && (o = r(o), o && (o = o.selection())), i == null ? s.remove() : i(s), u && o ? u.merge(o).order() : o;
}
function Gz(t) {
  for (var r = t.selection ? t.selection() : t, i = this._groups, u = r._groups, o = i.length, s = u.length, c = Math.min(o, s), f = new Array(o), g = 0; g < c; ++g)
    for (var h = i[g], v = u[g], p = h.length, m = f[g] = new Array(p), b, _ = 0; _ < p; ++_)
      (b = h[_] || v[_]) && (m[_] = b);
  for (; g < o; ++g)
    f[g] = i[g];
  return new Kt(f, this._parents);
}
function Vz() {
  for (var t = this._groups, r = -1, i = t.length; ++r < i; )
    for (var u = t[r], o = u.length - 1, s = u[o], c; --o >= 0; )
      (c = u[o]) && (s && c.compareDocumentPosition(s) ^ 4 && s.parentNode.insertBefore(c, s), s = c);
  return this;
}
function Yz(t) {
  t || (t = kz);
  function r(p, m) {
    return p && m ? t(p.__data__, m.__data__) : !p - !m;
  }
  for (var i = this._groups, u = i.length, o = new Array(u), s = 0; s < u; ++s) {
    for (var c = i[s], f = c.length, g = o[s] = new Array(f), h, v = 0; v < f; ++v)
      (h = c[v]) && (g[v] = h);
    g.sort(r);
  }
  return new Kt(o, this._parents).order();
}
function kz(t, r) {
  return t < r ? -1 : t > r ? 1 : t >= r ? 0 : NaN;
}
function Xz() {
  var t = arguments[0];
  return arguments[0] = this, t.apply(null, arguments), this;
}
function Iz() {
  return Array.from(this);
}
function Qz() {
  for (var t = this._groups, r = 0, i = t.length; r < i; ++r)
    for (var u = t[r], o = 0, s = u.length; o < s; ++o) {
      var c = u[o];
      if (c) return c;
    }
  return null;
}
function Zz() {
  let t = 0;
  for (const r of this) ++t;
  return t;
}
function Kz() {
  return !this.node();
}
function $z(t) {
  for (var r = this._groups, i = 0, u = r.length; i < u; ++i)
    for (var o = r[i], s = 0, c = o.length, f; s < c; ++s)
      (f = o[s]) && t.call(f, f.__data__, s, o);
  return this;
}
function Fz(t) {
  return function() {
    this.removeAttribute(t);
  };
}
function Jz(t) {
  return function() {
    this.removeAttributeNS(t.space, t.local);
  };
}
function Pz(t, r) {
  return function() {
    this.setAttribute(t, r);
  };
}
function Wz(t, r) {
  return function() {
    this.setAttributeNS(t.space, t.local, r);
  };
}
function e4(t, r) {
  return function() {
    var i = r.apply(this, arguments);
    i == null ? this.removeAttribute(t) : this.setAttribute(t, i);
  };
}
function t4(t, r) {
  return function() {
    var i = r.apply(this, arguments);
    i == null ? this.removeAttributeNS(t.space, t.local) : this.setAttributeNS(t.space, t.local, i);
  };
}
function n4(t, r) {
  var i = ss(t);
  if (arguments.length < 2) {
    var u = this.node();
    return i.local ? u.getAttributeNS(i.space, i.local) : u.getAttribute(i);
  }
  return this.each((r == null ? i.local ? Jz : Fz : typeof r == "function" ? i.local ? t4 : e4 : i.local ? Wz : Pz)(i, r));
}
function XT(t) {
  return t.ownerDocument && t.ownerDocument.defaultView || t.document && t || t.defaultView;
}
function r4(t) {
  return function() {
    this.style.removeProperty(t);
  };
}
function a4(t, r, i) {
  return function() {
    this.style.setProperty(t, r, i);
  };
}
function i4(t, r, i) {
  return function() {
    var u = r.apply(this, arguments);
    u == null ? this.style.removeProperty(t) : this.style.setProperty(t, u, i);
  };
}
function u4(t, r, i) {
  return arguments.length > 1 ? this.each((r == null ? r4 : typeof r == "function" ? i4 : a4)(t, r, i ?? "")) : oi(this.node(), t);
}
function oi(t, r) {
  return t.style.getPropertyValue(r) || XT(t).getComputedStyle(t, null).getPropertyValue(r);
}
function l4(t) {
  return function() {
    delete this[t];
  };
}
function o4(t, r) {
  return function() {
    this[t] = r;
  };
}
function s4(t, r) {
  return function() {
    var i = r.apply(this, arguments);
    i == null ? delete this[t] : this[t] = i;
  };
}
function c4(t, r) {
  return arguments.length > 1 ? this.each((r == null ? l4 : typeof r == "function" ? s4 : o4)(t, r)) : this.node()[t];
}
function IT(t) {
  return t.trim().split(/^|\s+/);
}
function Jp(t) {
  return t.classList || new QT(t);
}
function QT(t) {
  this._node = t, this._names = IT(t.getAttribute("class") || "");
}
QT.prototype = {
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
function ZT(t, r) {
  for (var i = Jp(t), u = -1, o = r.length; ++u < o; ) i.add(r[u]);
}
function KT(t, r) {
  for (var i = Jp(t), u = -1, o = r.length; ++u < o; ) i.remove(r[u]);
}
function f4(t) {
  return function() {
    ZT(this, t);
  };
}
function d4(t) {
  return function() {
    KT(this, t);
  };
}
function h4(t, r) {
  return function() {
    (r.apply(this, arguments) ? ZT : KT)(this, t);
  };
}
function g4(t, r) {
  var i = IT(t + "");
  if (arguments.length < 2) {
    for (var u = Jp(this.node()), o = -1, s = i.length; ++o < s; ) if (!u.contains(i[o])) return !1;
    return !0;
  }
  return this.each((typeof r == "function" ? h4 : r ? f4 : d4)(i, r));
}
function v4() {
  this.textContent = "";
}
function y4(t) {
  return function() {
    this.textContent = t;
  };
}
function p4(t) {
  return function() {
    var r = t.apply(this, arguments);
    this.textContent = r ?? "";
  };
}
function m4(t) {
  return arguments.length ? this.each(t == null ? v4 : (typeof t == "function" ? p4 : y4)(t)) : this.node().textContent;
}
function b4() {
  this.innerHTML = "";
}
function _4(t) {
  return function() {
    this.innerHTML = t;
  };
}
function x4(t) {
  return function() {
    var r = t.apply(this, arguments);
    this.innerHTML = r ?? "";
  };
}
function S4(t) {
  return arguments.length ? this.each(t == null ? b4 : (typeof t == "function" ? x4 : _4)(t)) : this.node().innerHTML;
}
function E4() {
  this.nextSibling && this.parentNode.appendChild(this);
}
function w4() {
  return this.each(E4);
}
function A4() {
  this.previousSibling && this.parentNode.insertBefore(this, this.parentNode.firstChild);
}
function T4() {
  return this.each(A4);
}
function M4(t) {
  var r = typeof t == "function" ? t : UT(t);
  return this.select(function() {
    return this.appendChild(r.apply(this, arguments));
  });
}
function q4() {
  return null;
}
function C4(t, r) {
  var i = typeof t == "function" ? t : UT(t), u = r == null ? q4 : typeof r == "function" ? r : Fp(r);
  return this.select(function() {
    return this.insertBefore(i.apply(this, arguments), u.apply(this, arguments) || null);
  });
}
function R4() {
  var t = this.parentNode;
  t && t.removeChild(this);
}
function N4() {
  return this.each(R4);
}
function O4() {
  var t = this.cloneNode(!1), r = this.parentNode;
  return r ? r.insertBefore(t, this.nextSibling) : t;
}
function z4() {
  var t = this.cloneNode(!0), r = this.parentNode;
  return r ? r.insertBefore(t, this.nextSibling) : t;
}
function D4(t) {
  return this.select(t ? z4 : O4);
}
function H4(t) {
  return arguments.length ? this.property("__data__", t) : this.node().__data__;
}
function L4(t) {
  return function(r) {
    t.call(this, r, this.__data__);
  };
}
function B4(t) {
  return t.trim().split(/^|\s+/).map(function(r) {
    var i = "", u = r.indexOf(".");
    return u >= 0 && (i = r.slice(u + 1), r = r.slice(0, u)), { type: r, name: i };
  });
}
function j4(t) {
  return function() {
    var r = this.__on;
    if (r) {
      for (var i = 0, u = -1, o = r.length, s; i < o; ++i)
        s = r[i], (!t.type || s.type === t.type) && s.name === t.name ? this.removeEventListener(s.type, s.listener, s.options) : r[++u] = s;
      ++u ? r.length = u : delete this.__on;
    }
  };
}
function U4(t, r, i) {
  return function() {
    var u = this.__on, o, s = L4(r);
    if (u) {
      for (var c = 0, f = u.length; c < f; ++c)
        if ((o = u[c]).type === t.type && o.name === t.name) {
          this.removeEventListener(o.type, o.listener, o.options), this.addEventListener(o.type, o.listener = s, o.options = i), o.value = r;
          return;
        }
    }
    this.addEventListener(t.type, s, i), o = { type: t.type, name: t.name, value: r, listener: s, options: i }, u ? u.push(o) : this.__on = [o];
  };
}
function G4(t, r, i) {
  var u = B4(t + ""), o, s = u.length, c;
  if (arguments.length < 2) {
    var f = this.node().__on;
    if (f) {
      for (var g = 0, h = f.length, v; g < h; ++g)
        for (o = 0, v = f[g]; o < s; ++o)
          if ((c = u[o]).type === v.type && c.name === v.name)
            return v.value;
    }
    return;
  }
  for (f = r ? U4 : j4, o = 0; o < s; ++o) this.each(f(u[o], r, i));
  return this;
}
function $T(t, r, i) {
  var u = XT(t), o = u.CustomEvent;
  typeof o == "function" ? o = new o(r, i) : (o = u.document.createEvent("Event"), i ? (o.initEvent(r, i.bubbles, i.cancelable), o.detail = i.detail) : o.initEvent(r, !1, !1)), t.dispatchEvent(o);
}
function V4(t, r) {
  return function() {
    return $T(this, t, r);
  };
}
function Y4(t, r) {
  return function() {
    return $T(this, t, r.apply(this, arguments));
  };
}
function k4(t, r) {
  return this.each((typeof r == "function" ? Y4 : V4)(t, r));
}
function* X4() {
  for (var t = this._groups, r = 0, i = t.length; r < i; ++r)
    for (var u = t[r], o = 0, s = u.length, c; o < s; ++o)
      (c = u[o]) && (yield c);
}
var FT = [null];
function Kt(t, r) {
  this._groups = t, this._parents = r;
}
function ku() {
  return new Kt([[document.documentElement]], FT);
}
function I4() {
  return this;
}
Kt.prototype = ku.prototype = {
  constructor: Kt,
  select: pz,
  selectAll: xz,
  selectChild: Az,
  selectChildren: Cz,
  filter: Rz,
  data: Lz,
  enter: Nz,
  exit: jz,
  join: Uz,
  merge: Gz,
  selection: I4,
  order: Vz,
  sort: Yz,
  call: Xz,
  nodes: Iz,
  node: Qz,
  size: Zz,
  empty: Kz,
  each: $z,
  attr: n4,
  style: u4,
  property: c4,
  classed: g4,
  text: m4,
  html: S4,
  raise: w4,
  lower: T4,
  append: M4,
  insert: C4,
  remove: N4,
  clone: D4,
  datum: H4,
  on: G4,
  dispatch: k4,
  [Symbol.iterator]: X4
};
function Zt(t) {
  return typeof t == "string" ? new Kt([[document.querySelector(t)]], [document.documentElement]) : new Kt([[t]], FT);
}
function Q4(t) {
  let r;
  for (; r = t.sourceEvent; ) t = r;
  return t;
}
function cn(t, r) {
  if (t = Q4(t), r === void 0 && (r = t.currentTarget), r) {
    var i = r.ownerSVGElement || r;
    if (i.createSVGPoint) {
      var u = i.createSVGPoint();
      return u.x = t.clientX, u.y = t.clientY, u = u.matrixTransform(r.getScreenCTM().inverse()), [u.x, u.y];
    }
    if (r.getBoundingClientRect) {
      var o = r.getBoundingClientRect();
      return [t.clientX - o.left - r.clientLeft, t.clientY - o.top - r.clientTop];
    }
  }
  return [t.pageX, t.pageY];
}
const Z4 = { passive: !1 }, qu = { capture: !0, passive: !1 };
function ap(t) {
  t.stopImmediatePropagation();
}
function ui(t) {
  t.preventDefault(), t.stopImmediatePropagation();
}
function JT(t) {
  var r = t.document.documentElement, i = Zt(t).on("dragstart.drag", ui, qu);
  "onselectstart" in r ? i.on("selectstart.drag", ui, qu) : (r.__noselect = r.style.MozUserSelect, r.style.MozUserSelect = "none");
}
function PT(t, r) {
  var i = t.document.documentElement, u = Zt(t).on("dragstart.drag", null);
  r && (u.on("click.drag", ui, qu), setTimeout(function() {
    u.on("click.drag", null);
  }, 0)), "onselectstart" in i ? u.on("selectstart.drag", null) : (i.style.MozUserSelect = i.__noselect, delete i.__noselect);
}
const bo = (t) => () => t;
function mp(t, {
  sourceEvent: r,
  subject: i,
  target: u,
  identifier: o,
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
    identifier: { value: o, enumerable: !0, configurable: !0 },
    active: { value: s, enumerable: !0, configurable: !0 },
    x: { value: c, enumerable: !0, configurable: !0 },
    y: { value: f, enumerable: !0, configurable: !0 },
    dx: { value: g, enumerable: !0, configurable: !0 },
    dy: { value: h, enumerable: !0, configurable: !0 },
    _: { value: v }
  });
}
mp.prototype.on = function() {
  var t = this._.on.apply(this._, arguments);
  return t === this._ ? this : t;
};
function K4(t) {
  return !t.ctrlKey && !t.button;
}
function $4() {
  return this.parentNode;
}
function F4(t, r) {
  return r ?? { x: t.x, y: t.y };
}
function J4() {
  return navigator.maxTouchPoints || "ontouchstart" in this;
}
function WT() {
  var t = K4, r = $4, i = F4, u = J4, o = {}, s = os("start", "drag", "end"), c = 0, f, g, h, v, p = 0;
  function m(T) {
    T.on("mousedown.drag", b).filter(u).on("touchstart.drag", w).on("touchmove.drag", E, Z4).on("touchend.drag touchcancel.drag", M).style("touch-action", "none").style("-webkit-tap-highlight-color", "rgba(0,0,0,0)");
  }
  function b(T, O) {
    if (!(v || !t.call(this, T, O))) {
      var C = S(this, r.call(this, T, O), T, O, "mouse");
      C && (Zt(T.view).on("mousemove.drag", _, qu).on("mouseup.drag", A, qu), JT(T.view), ap(T), h = !1, f = T.clientX, g = T.clientY, C("start", T));
    }
  }
  function _(T) {
    if (ui(T), !h) {
      var O = T.clientX - f, C = T.clientY - g;
      h = O * O + C * C > p;
    }
    o.mouse("drag", T);
  }
  function A(T) {
    Zt(T.view).on("mousemove.drag mouseup.drag", null), PT(T.view, h), ui(T), o.mouse("end", T);
  }
  function w(T, O) {
    if (t.call(this, T, O)) {
      var C = T.changedTouches, R = r.call(this, T, O), H = C.length, B, X;
      for (B = 0; B < H; ++B)
        (X = S(this, R, T, O, C[B].identifier, C[B])) && (ap(T), X("start", T, C[B]));
    }
  }
  function E(T) {
    var O = T.changedTouches, C = O.length, R, H;
    for (R = 0; R < C; ++R)
      (H = o[O[R].identifier]) && (ui(T), H("drag", T, O[R]));
  }
  function M(T) {
    var O = T.changedTouches, C = O.length, R, H;
    for (v && clearTimeout(v), v = setTimeout(function() {
      v = null;
    }, 500), R = 0; R < C; ++R)
      (H = o[O[R].identifier]) && (ap(T), H("end", T, O[R]));
  }
  function S(T, O, C, R, H, B) {
    var X = s.copy(), Y = cn(B || C, O), F, K, D;
    if ((D = i.call(T, new mp("beforestart", {
      sourceEvent: C,
      target: m,
      identifier: H,
      active: c,
      x: Y[0],
      y: Y[1],
      dx: 0,
      dy: 0,
      dispatch: X
    }), R)) != null)
      return F = D.x - Y[0] || 0, K = D.y - Y[1] || 0, function G(N, j, Z) {
        var Q = Y, le;
        switch (N) {
          case "start":
            o[H] = G, le = c++;
            break;
          case "end":
            delete o[H], --c;
          // falls through
          case "drag":
            Y = cn(Z || j, O), le = c;
            break;
        }
        X.call(
          N,
          T,
          new mp(N, {
            sourceEvent: j,
            subject: D,
            target: m,
            identifier: H,
            active: le,
            x: Y[0] + F,
            y: Y[1] + K,
            dx: Y[0] - Q[0],
            dy: Y[1] - Q[1],
            dispatch: X
          }),
          R
        );
      };
  }
  return m.filter = function(T) {
    return arguments.length ? (t = typeof T == "function" ? T : bo(!!T), m) : t;
  }, m.container = function(T) {
    return arguments.length ? (r = typeof T == "function" ? T : bo(T), m) : r;
  }, m.subject = function(T) {
    return arguments.length ? (i = typeof T == "function" ? T : bo(T), m) : i;
  }, m.touchable = function(T) {
    return arguments.length ? (u = typeof T == "function" ? T : bo(!!T), m) : u;
  }, m.on = function() {
    var T = s.on.apply(s, arguments);
    return T === s ? m : T;
  }, m.clickDistance = function(T) {
    return arguments.length ? (p = (T = +T) * T, m) : Math.sqrt(p);
  }, m;
}
function Pp(t, r, i) {
  t.prototype = r.prototype = i, i.constructor = t;
}
function eM(t, r) {
  var i = Object.create(t.prototype);
  for (var u in r) i[u] = r[u];
  return i;
}
function Xu() {
}
var Cu = 0.7, Ho = 1 / Cu, li = "\\s*([+-]?\\d+)\\s*", Ru = "\\s*([+-]?(?:\\d*\\.)?\\d+(?:[eE][+-]?\\d+)?)\\s*", An = "\\s*([+-]?(?:\\d*\\.)?\\d+(?:[eE][+-]?\\d+)?)%\\s*", P4 = /^#([0-9a-f]{3,8})$/, W4 = new RegExp(`^rgb\\(${li},${li},${li}\\)$`), eD = new RegExp(`^rgb\\(${An},${An},${An}\\)$`), tD = new RegExp(`^rgba\\(${li},${li},${li},${Ru}\\)$`), nD = new RegExp(`^rgba\\(${An},${An},${An},${Ru}\\)$`), rD = new RegExp(`^hsl\\(${Ru},${An},${An}\\)$`), aD = new RegExp(`^hsla\\(${Ru},${An},${An},${Ru}\\)$`), R2 = {
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
Pp(Xu, ua, {
  copy(t) {
    return Object.assign(new this.constructor(), this, t);
  },
  displayable() {
    return this.rgb().displayable();
  },
  hex: N2,
  // Deprecated! Use color.formatHex.
  formatHex: N2,
  formatHex8: iD,
  formatHsl: uD,
  formatRgb: O2,
  toString: O2
});
function N2() {
  return this.rgb().formatHex();
}
function iD() {
  return this.rgb().formatHex8();
}
function uD() {
  return tM(this).formatHsl();
}
function O2() {
  return this.rgb().formatRgb();
}
function ua(t) {
  var r, i;
  return t = (t + "").trim().toLowerCase(), (r = P4.exec(t)) ? (i = r[1].length, r = parseInt(r[1], 16), i === 6 ? z2(r) : i === 3 ? new jt(r >> 8 & 15 | r >> 4 & 240, r >> 4 & 15 | r & 240, (r & 15) << 4 | r & 15, 1) : i === 8 ? _o(r >> 24 & 255, r >> 16 & 255, r >> 8 & 255, (r & 255) / 255) : i === 4 ? _o(r >> 12 & 15 | r >> 8 & 240, r >> 8 & 15 | r >> 4 & 240, r >> 4 & 15 | r & 240, ((r & 15) << 4 | r & 15) / 255) : null) : (r = W4.exec(t)) ? new jt(r[1], r[2], r[3], 1) : (r = eD.exec(t)) ? new jt(r[1] * 255 / 100, r[2] * 255 / 100, r[3] * 255 / 100, 1) : (r = tD.exec(t)) ? _o(r[1], r[2], r[3], r[4]) : (r = nD.exec(t)) ? _o(r[1] * 255 / 100, r[2] * 255 / 100, r[3] * 255 / 100, r[4]) : (r = rD.exec(t)) ? L2(r[1], r[2] / 100, r[3] / 100, 1) : (r = aD.exec(t)) ? L2(r[1], r[2] / 100, r[3] / 100, r[4]) : R2.hasOwnProperty(t) ? z2(R2[t]) : t === "transparent" ? new jt(NaN, NaN, NaN, 0) : null;
}
function z2(t) {
  return new jt(t >> 16 & 255, t >> 8 & 255, t & 255, 1);
}
function _o(t, r, i, u) {
  return u <= 0 && (t = r = i = NaN), new jt(t, r, i, u);
}
function lD(t) {
  return t instanceof Xu || (t = ua(t)), t ? (t = t.rgb(), new jt(t.r, t.g, t.b, t.opacity)) : new jt();
}
function bp(t, r, i, u) {
  return arguments.length === 1 ? lD(t) : new jt(t, r, i, u ?? 1);
}
function jt(t, r, i, u) {
  this.r = +t, this.g = +r, this.b = +i, this.opacity = +u;
}
Pp(jt, bp, eM(Xu, {
  brighter(t) {
    return t = t == null ? Ho : Math.pow(Ho, t), new jt(this.r * t, this.g * t, this.b * t, this.opacity);
  },
  darker(t) {
    return t = t == null ? Cu : Math.pow(Cu, t), new jt(this.r * t, this.g * t, this.b * t, this.opacity);
  },
  rgb() {
    return this;
  },
  clamp() {
    return new jt(aa(this.r), aa(this.g), aa(this.b), Lo(this.opacity));
  },
  displayable() {
    return -0.5 <= this.r && this.r < 255.5 && -0.5 <= this.g && this.g < 255.5 && -0.5 <= this.b && this.b < 255.5 && 0 <= this.opacity && this.opacity <= 1;
  },
  hex: D2,
  // Deprecated! Use color.formatHex.
  formatHex: D2,
  formatHex8: oD,
  formatRgb: H2,
  toString: H2
}));
function D2() {
  return `#${ra(this.r)}${ra(this.g)}${ra(this.b)}`;
}
function oD() {
  return `#${ra(this.r)}${ra(this.g)}${ra(this.b)}${ra((isNaN(this.opacity) ? 1 : this.opacity) * 255)}`;
}
function H2() {
  const t = Lo(this.opacity);
  return `${t === 1 ? "rgb(" : "rgba("}${aa(this.r)}, ${aa(this.g)}, ${aa(this.b)}${t === 1 ? ")" : `, ${t})`}`;
}
function Lo(t) {
  return isNaN(t) ? 1 : Math.max(0, Math.min(1, t));
}
function aa(t) {
  return Math.max(0, Math.min(255, Math.round(t) || 0));
}
function ra(t) {
  return t = aa(t), (t < 16 ? "0" : "") + t.toString(16);
}
function L2(t, r, i, u) {
  return u <= 0 ? t = r = i = NaN : i <= 0 || i >= 1 ? t = r = NaN : r <= 0 && (t = NaN), new fn(t, r, i, u);
}
function tM(t) {
  if (t instanceof fn) return new fn(t.h, t.s, t.l, t.opacity);
  if (t instanceof Xu || (t = ua(t)), !t) return new fn();
  if (t instanceof fn) return t;
  t = t.rgb();
  var r = t.r / 255, i = t.g / 255, u = t.b / 255, o = Math.min(r, i, u), s = Math.max(r, i, u), c = NaN, f = s - o, g = (s + o) / 2;
  return f ? (r === s ? c = (i - u) / f + (i < u) * 6 : i === s ? c = (u - r) / f + 2 : c = (r - i) / f + 4, f /= g < 0.5 ? s + o : 2 - s - o, c *= 60) : f = g > 0 && g < 1 ? 0 : c, new fn(c, f, g, t.opacity);
}
function sD(t, r, i, u) {
  return arguments.length === 1 ? tM(t) : new fn(t, r, i, u ?? 1);
}
function fn(t, r, i, u) {
  this.h = +t, this.s = +r, this.l = +i, this.opacity = +u;
}
Pp(fn, sD, eM(Xu, {
  brighter(t) {
    return t = t == null ? Ho : Math.pow(Ho, t), new fn(this.h, this.s, this.l * t, this.opacity);
  },
  darker(t) {
    return t = t == null ? Cu : Math.pow(Cu, t), new fn(this.h, this.s, this.l * t, this.opacity);
  },
  rgb() {
    var t = this.h % 360 + (this.h < 0) * 360, r = isNaN(t) || isNaN(this.s) ? 0 : this.s, i = this.l, u = i + (i < 0.5 ? i : 1 - i) * r, o = 2 * i - u;
    return new jt(
      ip(t >= 240 ? t - 240 : t + 120, o, u),
      ip(t, o, u),
      ip(t < 120 ? t + 240 : t - 120, o, u),
      this.opacity
    );
  },
  clamp() {
    return new fn(B2(this.h), xo(this.s), xo(this.l), Lo(this.opacity));
  },
  displayable() {
    return (0 <= this.s && this.s <= 1 || isNaN(this.s)) && 0 <= this.l && this.l <= 1 && 0 <= this.opacity && this.opacity <= 1;
  },
  formatHsl() {
    const t = Lo(this.opacity);
    return `${t === 1 ? "hsl(" : "hsla("}${B2(this.h)}, ${xo(this.s) * 100}%, ${xo(this.l) * 100}%${t === 1 ? ")" : `, ${t})`}`;
  }
}));
function B2(t) {
  return t = (t || 0) % 360, t < 0 ? t + 360 : t;
}
function xo(t) {
  return Math.max(0, Math.min(1, t || 0));
}
function ip(t, r, i) {
  return (t < 60 ? r + (i - r) * t / 60 : t < 180 ? i : t < 240 ? r + (i - r) * (240 - t) / 60 : r) * 255;
}
const Wp = (t) => () => t;
function cD(t, r) {
  return function(i) {
    return t + i * r;
  };
}
function fD(t, r, i) {
  return t = Math.pow(t, i), r = Math.pow(r, i) - t, i = 1 / i, function(u) {
    return Math.pow(t + u * r, i);
  };
}
function dD(t) {
  return (t = +t) == 1 ? nM : function(r, i) {
    return i - r ? fD(r, i, t) : Wp(isNaN(r) ? i : r);
  };
}
function nM(t, r) {
  var i = r - t;
  return i ? cD(t, i) : Wp(isNaN(t) ? r : t);
}
const Bo = (function t(r) {
  var i = dD(r);
  function u(o, s) {
    var c = i((o = bp(o)).r, (s = bp(s)).r), f = i(o.g, s.g), g = i(o.b, s.b), h = nM(o.opacity, s.opacity);
    return function(v) {
      return o.r = c(v), o.g = f(v), o.b = g(v), o.opacity = h(v), o + "";
    };
  }
  return u.gamma = t, u;
})(1);
function hD(t, r) {
  r || (r = []);
  var i = t ? Math.min(r.length, t.length) : 0, u = r.slice(), o;
  return function(s) {
    for (o = 0; o < i; ++o) u[o] = t[o] * (1 - s) + r[o] * s;
    return u;
  };
}
function gD(t) {
  return ArrayBuffer.isView(t) && !(t instanceof DataView);
}
function vD(t, r) {
  var i = r ? r.length : 0, u = t ? Math.min(i, t.length) : 0, o = new Array(u), s = new Array(i), c;
  for (c = 0; c < u; ++c) o[c] = Tu(t[c], r[c]);
  for (; c < i; ++c) s[c] = r[c];
  return function(f) {
    for (c = 0; c < u; ++c) s[c] = o[c](f);
    return s;
  };
}
function yD(t, r) {
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
function pD(t, r) {
  var i = {}, u = {}, o;
  (t === null || typeof t != "object") && (t = {}), (r === null || typeof r != "object") && (r = {});
  for (o in r)
    o in t ? i[o] = Tu(t[o], r[o]) : u[o] = r[o];
  return function(s) {
    for (o in i) u[o] = i[o](s);
    return u;
  };
}
var _p = /[-+]?(?:\d+\.?\d*|\.?\d+)(?:[eE][-+]?\d+)?/g, up = new RegExp(_p.source, "g");
function mD(t) {
  return function() {
    return t;
  };
}
function bD(t) {
  return function(r) {
    return t(r) + "";
  };
}
function rM(t, r) {
  var i = _p.lastIndex = up.lastIndex = 0, u, o, s, c = -1, f = [], g = [];
  for (t = t + "", r = r + ""; (u = _p.exec(t)) && (o = up.exec(r)); )
    (s = o.index) > i && (s = r.slice(i, s), f[c] ? f[c] += s : f[++c] = s), (u = u[0]) === (o = o[0]) ? f[c] ? f[c] += o : f[++c] = o : (f[++c] = null, g.push({ i: c, x: wn(u, o) })), i = up.lastIndex;
  return i < r.length && (s = r.slice(i), f[c] ? f[c] += s : f[++c] = s), f.length < 2 ? g[0] ? bD(g[0].x) : mD(r) : (r = g.length, function(h) {
    for (var v = 0, p; v < r; ++v) f[(p = g[v]).i] = p.x(h);
    return f.join("");
  });
}
function Tu(t, r) {
  var i = typeof r, u;
  return r == null || i === "boolean" ? Wp(r) : (i === "number" ? wn : i === "string" ? (u = ua(r)) ? (r = u, Bo) : rM : r instanceof ua ? Bo : r instanceof Date ? yD : gD(r) ? hD : Array.isArray(r) ? vD : typeof r.valueOf != "function" && typeof r.toString != "function" || isNaN(r) ? pD : wn)(t, r);
}
var j2 = 180 / Math.PI, xp = {
  translateX: 0,
  translateY: 0,
  rotate: 0,
  skewX: 0,
  scaleX: 1,
  scaleY: 1
};
function aM(t, r, i, u, o, s) {
  var c, f, g;
  return (c = Math.sqrt(t * t + r * r)) && (t /= c, r /= c), (g = t * i + r * u) && (i -= t * g, u -= r * g), (f = Math.sqrt(i * i + u * u)) && (i /= f, u /= f, g /= f), t * u < r * i && (t = -t, r = -r, g = -g, c = -c), {
    translateX: o,
    translateY: s,
    rotate: Math.atan2(r, t) * j2,
    skewX: Math.atan(g) * j2,
    scaleX: c,
    scaleY: f
  };
}
var So;
function _D(t) {
  const r = new (typeof DOMMatrix == "function" ? DOMMatrix : WebKitCSSMatrix)(t + "");
  return r.isIdentity ? xp : aM(r.a, r.b, r.c, r.d, r.e, r.f);
}
function xD(t) {
  return t == null || (So || (So = document.createElementNS("http://www.w3.org/2000/svg", "g")), So.setAttribute("transform", t), !(t = So.transform.baseVal.consolidate())) ? xp : (t = t.matrix, aM(t.a, t.b, t.c, t.d, t.e, t.f));
}
function iM(t, r, i, u) {
  function o(h) {
    return h.length ? h.pop() + " " : "";
  }
  function s(h, v, p, m, b, _) {
    if (h !== p || v !== m) {
      var A = b.push("translate(", null, r, null, i);
      _.push({ i: A - 4, x: wn(h, p) }, { i: A - 2, x: wn(v, m) });
    } else (p || m) && b.push("translate(" + p + r + m + i);
  }
  function c(h, v, p, m) {
    h !== v ? (h - v > 180 ? v += 360 : v - h > 180 && (h += 360), m.push({ i: p.push(o(p) + "rotate(", null, u) - 2, x: wn(h, v) })) : v && p.push(o(p) + "rotate(" + v + u);
  }
  function f(h, v, p, m) {
    h !== v ? m.push({ i: p.push(o(p) + "skewX(", null, u) - 2, x: wn(h, v) }) : v && p.push(o(p) + "skewX(" + v + u);
  }
  function g(h, v, p, m, b, _) {
    if (h !== p || v !== m) {
      var A = b.push(o(b) + "scale(", null, ",", null, ")");
      _.push({ i: A - 4, x: wn(h, p) }, { i: A - 2, x: wn(v, m) });
    } else (p !== 1 || m !== 1) && b.push(o(b) + "scale(" + p + "," + m + ")");
  }
  return function(h, v) {
    var p = [], m = [];
    return h = t(h), v = t(v), s(h.translateX, h.translateY, v.translateX, v.translateY, p, m), c(h.rotate, v.rotate, p, m), f(h.skewX, v.skewX, p, m), g(h.scaleX, h.scaleY, v.scaleX, v.scaleY, p, m), h = v = null, function(b) {
      for (var _ = -1, A = m.length, w; ++_ < A; ) p[(w = m[_]).i] = w.x(b);
      return p.join("");
    };
  };
}
var SD = iM(_D, "px, ", "px)", "deg)"), ED = iM(xD, ", ", ")", ")"), wD = 1e-12;
function U2(t) {
  return ((t = Math.exp(t)) + 1 / t) / 2;
}
function AD(t) {
  return ((t = Math.exp(t)) - 1 / t) / 2;
}
function TD(t) {
  return ((t = Math.exp(2 * t)) - 1) / (t + 1);
}
const Co = (function t(r, i, u) {
  function o(s, c) {
    var f = s[0], g = s[1], h = s[2], v = c[0], p = c[1], m = c[2], b = v - f, _ = p - g, A = b * b + _ * _, w, E;
    if (A < wD)
      E = Math.log(m / h) / r, w = function(R) {
        return [
          f + R * b,
          g + R * _,
          h * Math.exp(r * R * E)
        ];
      };
    else {
      var M = Math.sqrt(A), S = (m * m - h * h + u * A) / (2 * h * i * M), T = (m * m - h * h - u * A) / (2 * m * i * M), O = Math.log(Math.sqrt(S * S + 1) - S), C = Math.log(Math.sqrt(T * T + 1) - T);
      E = (C - O) / r, w = function(R) {
        var H = R * E, B = U2(O), X = h / (i * M) * (B * TD(r * H + O) - AD(O));
        return [
          f + X * b,
          g + X * _,
          h * B / U2(r * H + O)
        ];
      };
    }
    return w.duration = E * 1e3 * r / Math.SQRT2, w;
  }
  return o.rho = function(s) {
    var c = Math.max(1e-3, +s), f = c * c, g = f * f;
    return t(c, f, g);
  }, o;
})(Math.SQRT2, 2, 4);
var si = 0, wu = 0, bu = 0, uM = 1e3, jo, Au, Uo = 0, la = 0, cs = 0, Nu = typeof performance == "object" && performance.now ? performance : Date, lM = typeof window == "object" && window.requestAnimationFrame ? window.requestAnimationFrame.bind(window) : function(t) {
  setTimeout(t, 17);
};
function em() {
  return la || (lM(MD), la = Nu.now() + cs);
}
function MD() {
  la = 0;
}
function Go() {
  this._call = this._time = this._next = null;
}
Go.prototype = oM.prototype = {
  constructor: Go,
  restart: function(t, r, i) {
    if (typeof t != "function") throw new TypeError("callback is not a function");
    i = (i == null ? em() : +i) + (r == null ? 0 : +r), !this._next && Au !== this && (Au ? Au._next = this : jo = this, Au = this), this._call = t, this._time = i, Sp();
  },
  stop: function() {
    this._call && (this._call = null, this._time = 1 / 0, Sp());
  }
};
function oM(t, r, i) {
  var u = new Go();
  return u.restart(t, r, i), u;
}
function qD() {
  em(), ++si;
  for (var t = jo, r; t; )
    (r = la - t._time) >= 0 && t._call.call(void 0, r), t = t._next;
  --si;
}
function G2() {
  la = (Uo = Nu.now()) + cs, si = wu = 0;
  try {
    qD();
  } finally {
    si = 0, RD(), la = 0;
  }
}
function CD() {
  var t = Nu.now(), r = t - Uo;
  r > uM && (cs -= r, Uo = t);
}
function RD() {
  for (var t, r = jo, i, u = 1 / 0; r; )
    r._call ? (u > r._time && (u = r._time), t = r, r = r._next) : (i = r._next, r._next = null, r = t ? t._next = i : jo = i);
  Au = t, Sp(u);
}
function Sp(t) {
  if (!si) {
    wu && (wu = clearTimeout(wu));
    var r = t - la;
    r > 24 ? (t < 1 / 0 && (wu = setTimeout(G2, t - Nu.now() - cs)), bu && (bu = clearInterval(bu))) : (bu || (Uo = Nu.now(), bu = setInterval(CD, uM)), si = 1, lM(G2));
  }
}
function V2(t, r, i) {
  var u = new Go();
  return r = r == null ? 0 : +r, u.restart((o) => {
    u.stop(), t(o + r);
  }, r, i), u;
}
var ND = os("start", "end", "cancel", "interrupt"), OD = [], sM = 0, Y2 = 1, Ep = 2, Ro = 3, k2 = 4, wp = 5, No = 6;
function fs(t, r, i, u, o, s) {
  var c = t.__transition;
  if (!c) t.__transition = {};
  else if (i in c) return;
  zD(t, i, {
    name: r,
    index: u,
    // For context during callback.
    group: o,
    // For context during callback.
    on: ND,
    tween: OD,
    time: s.time,
    delay: s.delay,
    duration: s.duration,
    ease: s.ease,
    timer: null,
    state: sM
  });
}
function tm(t, r) {
  var i = pn(t, r);
  if (i.state > sM) throw new Error("too late; already scheduled");
  return i;
}
function Mn(t, r) {
  var i = pn(t, r);
  if (i.state > Ro) throw new Error("too late; already running");
  return i;
}
function pn(t, r) {
  var i = t.__transition;
  if (!i || !(i = i[r])) throw new Error("transition not found");
  return i;
}
function zD(t, r, i) {
  var u = t.__transition, o;
  u[r] = i, i.timer = oM(s, 0, i.time);
  function s(h) {
    i.state = Y2, i.timer.restart(c, i.delay, i.time), i.delay <= h && c(h - i.delay);
  }
  function c(h) {
    var v, p, m, b;
    if (i.state !== Y2) return g();
    for (v in u)
      if (b = u[v], b.name === i.name) {
        if (b.state === Ro) return V2(c);
        b.state === k2 ? (b.state = No, b.timer.stop(), b.on.call("interrupt", t, t.__data__, b.index, b.group), delete u[v]) : +v < r && (b.state = No, b.timer.stop(), b.on.call("cancel", t, t.__data__, b.index, b.group), delete u[v]);
      }
    if (V2(function() {
      i.state === Ro && (i.state = k2, i.timer.restart(f, i.delay, i.time), f(h));
    }), i.state = Ep, i.on.call("start", t, t.__data__, i.index, i.group), i.state === Ep) {
      for (i.state = Ro, o = new Array(m = i.tween.length), v = 0, p = -1; v < m; ++v)
        (b = i.tween[v].value.call(t, t.__data__, i.index, i.group)) && (o[++p] = b);
      o.length = p + 1;
    }
  }
  function f(h) {
    for (var v = h < i.duration ? i.ease.call(null, h / i.duration) : (i.timer.restart(g), i.state = wp, 1), p = -1, m = o.length; ++p < m; )
      o[p].call(t, v);
    i.state === wp && (i.on.call("end", t, t.__data__, i.index, i.group), g());
  }
  function g() {
    i.state = No, i.timer.stop(), delete u[r];
    for (var h in u) return;
    delete t.__transition;
  }
}
function Oo(t, r) {
  var i = t.__transition, u, o, s = !0, c;
  if (i) {
    r = r == null ? null : r + "";
    for (c in i) {
      if ((u = i[c]).name !== r) {
        s = !1;
        continue;
      }
      o = u.state > Ep && u.state < wp, u.state = No, u.timer.stop(), u.on.call(o ? "interrupt" : "cancel", t, t.__data__, u.index, u.group), delete i[c];
    }
    s && delete t.__transition;
  }
}
function DD(t) {
  return this.each(function() {
    Oo(this, t);
  });
}
function HD(t, r) {
  var i, u;
  return function() {
    var o = Mn(this, t), s = o.tween;
    if (s !== i) {
      u = i = s;
      for (var c = 0, f = u.length; c < f; ++c)
        if (u[c].name === r) {
          u = u.slice(), u.splice(c, 1);
          break;
        }
    }
    o.tween = u;
  };
}
function LD(t, r, i) {
  var u, o;
  if (typeof i != "function") throw new Error();
  return function() {
    var s = Mn(this, t), c = s.tween;
    if (c !== u) {
      o = (u = c).slice();
      for (var f = { name: r, value: i }, g = 0, h = o.length; g < h; ++g)
        if (o[g].name === r) {
          o[g] = f;
          break;
        }
      g === h && o.push(f);
    }
    s.tween = o;
  };
}
function BD(t, r) {
  var i = this._id;
  if (t += "", arguments.length < 2) {
    for (var u = pn(this.node(), i).tween, o = 0, s = u.length, c; o < s; ++o)
      if ((c = u[o]).name === t)
        return c.value;
    return null;
  }
  return this.each((r == null ? HD : LD)(i, t, r));
}
function nm(t, r, i) {
  var u = t._id;
  return t.each(function() {
    var o = Mn(this, u);
    (o.value || (o.value = {}))[r] = i.apply(this, arguments);
  }), function(o) {
    return pn(o, u).value[r];
  };
}
function cM(t, r) {
  var i;
  return (typeof r == "number" ? wn : r instanceof ua ? Bo : (i = ua(r)) ? (r = i, Bo) : rM)(t, r);
}
function jD(t) {
  return function() {
    this.removeAttribute(t);
  };
}
function UD(t) {
  return function() {
    this.removeAttributeNS(t.space, t.local);
  };
}
function GD(t, r, i) {
  var u, o = i + "", s;
  return function() {
    var c = this.getAttribute(t);
    return c === o ? null : c === u ? s : s = r(u = c, i);
  };
}
function VD(t, r, i) {
  var u, o = i + "", s;
  return function() {
    var c = this.getAttributeNS(t.space, t.local);
    return c === o ? null : c === u ? s : s = r(u = c, i);
  };
}
function YD(t, r, i) {
  var u, o, s;
  return function() {
    var c, f = i(this), g;
    return f == null ? void this.removeAttribute(t) : (c = this.getAttribute(t), g = f + "", c === g ? null : c === u && g === o ? s : (o = g, s = r(u = c, f)));
  };
}
function kD(t, r, i) {
  var u, o, s;
  return function() {
    var c, f = i(this), g;
    return f == null ? void this.removeAttributeNS(t.space, t.local) : (c = this.getAttributeNS(t.space, t.local), g = f + "", c === g ? null : c === u && g === o ? s : (o = g, s = r(u = c, f)));
  };
}
function XD(t, r) {
  var i = ss(t), u = i === "transform" ? ED : cM;
  return this.attrTween(t, typeof r == "function" ? (i.local ? kD : YD)(i, u, nm(this, "attr." + t, r)) : r == null ? (i.local ? UD : jD)(i) : (i.local ? VD : GD)(i, u, r));
}
function ID(t, r) {
  return function(i) {
    this.setAttribute(t, r.call(this, i));
  };
}
function QD(t, r) {
  return function(i) {
    this.setAttributeNS(t.space, t.local, r.call(this, i));
  };
}
function ZD(t, r) {
  var i, u;
  function o() {
    var s = r.apply(this, arguments);
    return s !== u && (i = (u = s) && QD(t, s)), i;
  }
  return o._value = r, o;
}
function KD(t, r) {
  var i, u;
  function o() {
    var s = r.apply(this, arguments);
    return s !== u && (i = (u = s) && ID(t, s)), i;
  }
  return o._value = r, o;
}
function $D(t, r) {
  var i = "attr." + t;
  if (arguments.length < 2) return (i = this.tween(i)) && i._value;
  if (r == null) return this.tween(i, null);
  if (typeof r != "function") throw new Error();
  var u = ss(t);
  return this.tween(i, (u.local ? ZD : KD)(u, r));
}
function FD(t, r) {
  return function() {
    tm(this, t).delay = +r.apply(this, arguments);
  };
}
function JD(t, r) {
  return r = +r, function() {
    tm(this, t).delay = r;
  };
}
function PD(t) {
  var r = this._id;
  return arguments.length ? this.each((typeof t == "function" ? FD : JD)(r, t)) : pn(this.node(), r).delay;
}
function WD(t, r) {
  return function() {
    Mn(this, t).duration = +r.apply(this, arguments);
  };
}
function e5(t, r) {
  return r = +r, function() {
    Mn(this, t).duration = r;
  };
}
function t5(t) {
  var r = this._id;
  return arguments.length ? this.each((typeof t == "function" ? WD : e5)(r, t)) : pn(this.node(), r).duration;
}
function n5(t, r) {
  if (typeof r != "function") throw new Error();
  return function() {
    Mn(this, t).ease = r;
  };
}
function r5(t) {
  var r = this._id;
  return arguments.length ? this.each(n5(r, t)) : pn(this.node(), r).ease;
}
function a5(t, r) {
  return function() {
    var i = r.apply(this, arguments);
    if (typeof i != "function") throw new Error();
    Mn(this, t).ease = i;
  };
}
function i5(t) {
  if (typeof t != "function") throw new Error();
  return this.each(a5(this._id, t));
}
function u5(t) {
  typeof t != "function" && (t = VT(t));
  for (var r = this._groups, i = r.length, u = new Array(i), o = 0; o < i; ++o)
    for (var s = r[o], c = s.length, f = u[o] = [], g, h = 0; h < c; ++h)
      (g = s[h]) && t.call(g, g.__data__, h, s) && f.push(g);
  return new Pn(u, this._parents, this._name, this._id);
}
function l5(t) {
  if (t._id !== this._id) throw new Error();
  for (var r = this._groups, i = t._groups, u = r.length, o = i.length, s = Math.min(u, o), c = new Array(u), f = 0; f < s; ++f)
    for (var g = r[f], h = i[f], v = g.length, p = c[f] = new Array(v), m, b = 0; b < v; ++b)
      (m = g[b] || h[b]) && (p[b] = m);
  for (; f < u; ++f)
    c[f] = r[f];
  return new Pn(c, this._parents, this._name, this._id);
}
function o5(t) {
  return (t + "").trim().split(/^|\s+/).every(function(r) {
    var i = r.indexOf(".");
    return i >= 0 && (r = r.slice(0, i)), !r || r === "start";
  });
}
function s5(t, r, i) {
  var u, o, s = o5(r) ? tm : Mn;
  return function() {
    var c = s(this, t), f = c.on;
    f !== u && (o = (u = f).copy()).on(r, i), c.on = o;
  };
}
function c5(t, r) {
  var i = this._id;
  return arguments.length < 2 ? pn(this.node(), i).on.on(t) : this.each(s5(i, t, r));
}
function f5(t) {
  return function() {
    var r = this.parentNode;
    for (var i in this.__transition) if (+i !== t) return;
    r && r.removeChild(this);
  };
}
function d5() {
  return this.on("end.remove", f5(this._id));
}
function h5(t) {
  var r = this._name, i = this._id;
  typeof t != "function" && (t = Fp(t));
  for (var u = this._groups, o = u.length, s = new Array(o), c = 0; c < o; ++c)
    for (var f = u[c], g = f.length, h = s[c] = new Array(g), v, p, m = 0; m < g; ++m)
      (v = f[m]) && (p = t.call(v, v.__data__, m, f)) && ("__data__" in v && (p.__data__ = v.__data__), h[m] = p, fs(h[m], r, i, m, h, pn(v, i)));
  return new Pn(s, this._parents, r, i);
}
function g5(t) {
  var r = this._name, i = this._id;
  typeof t != "function" && (t = GT(t));
  for (var u = this._groups, o = u.length, s = [], c = [], f = 0; f < o; ++f)
    for (var g = u[f], h = g.length, v, p = 0; p < h; ++p)
      if (v = g[p]) {
        for (var m = t.call(v, v.__data__, p, g), b, _ = pn(v, i), A = 0, w = m.length; A < w; ++A)
          (b = m[A]) && fs(b, r, i, A, m, _);
        s.push(m), c.push(v);
      }
  return new Pn(s, c, r, i);
}
var v5 = ku.prototype.constructor;
function y5() {
  return new v5(this._groups, this._parents);
}
function p5(t, r) {
  var i, u, o;
  return function() {
    var s = oi(this, t), c = (this.style.removeProperty(t), oi(this, t));
    return s === c ? null : s === i && c === u ? o : o = r(i = s, u = c);
  };
}
function fM(t) {
  return function() {
    this.style.removeProperty(t);
  };
}
function m5(t, r, i) {
  var u, o = i + "", s;
  return function() {
    var c = oi(this, t);
    return c === o ? null : c === u ? s : s = r(u = c, i);
  };
}
function b5(t, r, i) {
  var u, o, s;
  return function() {
    var c = oi(this, t), f = i(this), g = f + "";
    return f == null && (g = f = (this.style.removeProperty(t), oi(this, t))), c === g ? null : c === u && g === o ? s : (o = g, s = r(u = c, f));
  };
}
function _5(t, r) {
  var i, u, o, s = "style." + r, c = "end." + s, f;
  return function() {
    var g = Mn(this, t), h = g.on, v = g.value[s] == null ? f || (f = fM(r)) : void 0;
    (h !== i || o !== v) && (u = (i = h).copy()).on(c, o = v), g.on = u;
  };
}
function x5(t, r, i) {
  var u = (t += "") == "transform" ? SD : cM;
  return r == null ? this.styleTween(t, p5(t, u)).on("end.style." + t, fM(t)) : typeof r == "function" ? this.styleTween(t, b5(t, u, nm(this, "style." + t, r))).each(_5(this._id, t)) : this.styleTween(t, m5(t, u, r), i).on("end.style." + t, null);
}
function S5(t, r, i) {
  return function(u) {
    this.style.setProperty(t, r.call(this, u), i);
  };
}
function E5(t, r, i) {
  var u, o;
  function s() {
    var c = r.apply(this, arguments);
    return c !== o && (u = (o = c) && S5(t, c, i)), u;
  }
  return s._value = r, s;
}
function w5(t, r, i) {
  var u = "style." + (t += "");
  if (arguments.length < 2) return (u = this.tween(u)) && u._value;
  if (r == null) return this.tween(u, null);
  if (typeof r != "function") throw new Error();
  return this.tween(u, E5(t, r, i ?? ""));
}
function A5(t) {
  return function() {
    this.textContent = t;
  };
}
function T5(t) {
  return function() {
    var r = t(this);
    this.textContent = r ?? "";
  };
}
function M5(t) {
  return this.tween("text", typeof t == "function" ? T5(nm(this, "text", t)) : A5(t == null ? "" : t + ""));
}
function q5(t) {
  return function(r) {
    this.textContent = t.call(this, r);
  };
}
function C5(t) {
  var r, i;
  function u() {
    var o = t.apply(this, arguments);
    return o !== i && (r = (i = o) && q5(o)), r;
  }
  return u._value = t, u;
}
function R5(t) {
  var r = "text";
  if (arguments.length < 1) return (r = this.tween(r)) && r._value;
  if (t == null) return this.tween(r, null);
  if (typeof t != "function") throw new Error();
  return this.tween(r, C5(t));
}
function N5() {
  for (var t = this._name, r = this._id, i = dM(), u = this._groups, o = u.length, s = 0; s < o; ++s)
    for (var c = u[s], f = c.length, g, h = 0; h < f; ++h)
      if (g = c[h]) {
        var v = pn(g, r);
        fs(g, t, i, h, c, {
          time: v.time + v.delay + v.duration,
          delay: 0,
          duration: v.duration,
          ease: v.ease
        });
      }
  return new Pn(u, this._parents, t, i);
}
function O5() {
  var t, r, i = this, u = i._id, o = i.size();
  return new Promise(function(s, c) {
    var f = { value: c }, g = { value: function() {
      --o === 0 && s();
    } };
    i.each(function() {
      var h = Mn(this, u), v = h.on;
      v !== t && (r = (t = v).copy(), r._.cancel.push(f), r._.interrupt.push(f), r._.end.push(g)), h.on = r;
    }), o === 0 && s();
  });
}
var z5 = 0;
function Pn(t, r, i, u) {
  this._groups = t, this._parents = r, this._name = i, this._id = u;
}
function dM() {
  return ++z5;
}
var Fn = ku.prototype;
Pn.prototype = {
  constructor: Pn,
  select: h5,
  selectAll: g5,
  selectChild: Fn.selectChild,
  selectChildren: Fn.selectChildren,
  filter: u5,
  merge: l5,
  selection: y5,
  transition: N5,
  call: Fn.call,
  nodes: Fn.nodes,
  node: Fn.node,
  size: Fn.size,
  empty: Fn.empty,
  each: Fn.each,
  on: c5,
  attr: XD,
  attrTween: $D,
  style: x5,
  styleTween: w5,
  text: M5,
  textTween: R5,
  remove: d5,
  tween: BD,
  delay: PD,
  duration: t5,
  ease: r5,
  easeVarying: i5,
  end: O5,
  [Symbol.iterator]: Fn[Symbol.iterator]
};
function D5(t) {
  return ((t *= 2) <= 1 ? t * t * t : (t -= 2) * t * t + 2) / 2;
}
var H5 = {
  time: null,
  // Set on use.
  delay: 0,
  duration: 250,
  ease: D5
};
function L5(t, r) {
  for (var i; !(i = t.__transition) || !(i = i[r]); )
    if (!(t = t.parentNode))
      throw new Error(`transition ${r} not found`);
  return i;
}
function B5(t) {
  var r, i;
  t instanceof Pn ? (r = t._id, t = t._name) : (r = dM(), (i = H5).time = em(), t = t == null ? null : t + "");
  for (var u = this._groups, o = u.length, s = 0; s < o; ++s)
    for (var c = u[s], f = c.length, g, h = 0; h < f; ++h)
      (g = c[h]) && fs(g, t, r, h, c, i || L5(g, r));
  return new Pn(u, this._parents, t, r);
}
ku.prototype.interrupt = DD;
ku.prototype.transition = B5;
const Eo = (t) => () => t;
function j5(t, {
  sourceEvent: r,
  target: i,
  transform: u,
  dispatch: o
}) {
  Object.defineProperties(this, {
    type: { value: t, enumerable: !0, configurable: !0 },
    sourceEvent: { value: r, enumerable: !0, configurable: !0 },
    target: { value: i, enumerable: !0, configurable: !0 },
    transform: { value: u, enumerable: !0, configurable: !0 },
    _: { value: o }
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
var ds = new Jn(1, 0, 0);
hM.prototype = Jn.prototype;
function hM(t) {
  for (; !t.__zoom; ) if (!(t = t.parentNode)) return ds;
  return t.__zoom;
}
function lp(t) {
  t.stopImmediatePropagation();
}
function _u(t) {
  t.preventDefault(), t.stopImmediatePropagation();
}
function U5(t) {
  return (!t.ctrlKey || t.type === "wheel") && !t.button;
}
function G5() {
  var t = this;
  return t instanceof SVGElement ? (t = t.ownerSVGElement || t, t.hasAttribute("viewBox") ? (t = t.viewBox.baseVal, [[t.x, t.y], [t.x + t.width, t.y + t.height]]) : [[0, 0], [t.width.baseVal.value, t.height.baseVal.value]]) : [[0, 0], [t.clientWidth, t.clientHeight]];
}
function X2() {
  return this.__zoom || ds;
}
function V5(t) {
  return -t.deltaY * (t.deltaMode === 1 ? 0.05 : t.deltaMode ? 1 : 2e-3) * (t.ctrlKey ? 10 : 1);
}
function Y5() {
  return navigator.maxTouchPoints || "ontouchstart" in this;
}
function k5(t, r, i) {
  var u = t.invertX(r[0][0]) - i[0][0], o = t.invertX(r[1][0]) - i[1][0], s = t.invertY(r[0][1]) - i[0][1], c = t.invertY(r[1][1]) - i[1][1];
  return t.translate(
    o > u ? (u + o) / 2 : Math.min(0, u) || Math.max(0, o),
    c > s ? (s + c) / 2 : Math.min(0, s) || Math.max(0, c)
  );
}
function gM() {
  var t = U5, r = G5, i = k5, u = V5, o = Y5, s = [0, 1 / 0], c = [[-1 / 0, -1 / 0], [1 / 0, 1 / 0]], f = 250, g = Co, h = os("start", "zoom", "end"), v, p, m, b = 500, _ = 150, A = 0, w = 10;
  function E(D) {
    D.property("__zoom", X2).on("wheel.zoom", H, { passive: !1 }).on("mousedown.zoom", B).on("dblclick.zoom", X).filter(o).on("touchstart.zoom", Y).on("touchmove.zoom", F).on("touchend.zoom touchcancel.zoom", K).style("-webkit-tap-highlight-color", "rgba(0,0,0,0)");
  }
  E.transform = function(D, G, N, j) {
    var Z = D.selection ? D.selection() : D;
    Z.property("__zoom", X2), D !== Z ? O(D, G, N, j) : Z.interrupt().each(function() {
      C(this, arguments).event(j).start().zoom(null, typeof G == "function" ? G.apply(this, arguments) : G).end();
    });
  }, E.scaleBy = function(D, G, N, j) {
    E.scaleTo(D, function() {
      var Z = this.__zoom.k, Q = typeof G == "function" ? G.apply(this, arguments) : G;
      return Z * Q;
    }, N, j);
  }, E.scaleTo = function(D, G, N, j) {
    E.transform(D, function() {
      var Z = r.apply(this, arguments), Q = this.__zoom, le = N == null ? T(Z) : typeof N == "function" ? N.apply(this, arguments) : N, z = Q.invert(le), V = typeof G == "function" ? G.apply(this, arguments) : G;
      return i(S(M(Q, V), le, z), Z, c);
    }, N, j);
  }, E.translateBy = function(D, G, N, j) {
    E.transform(D, function() {
      return i(this.__zoom.translate(
        typeof G == "function" ? G.apply(this, arguments) : G,
        typeof N == "function" ? N.apply(this, arguments) : N
      ), r.apply(this, arguments), c);
    }, null, j);
  }, E.translateTo = function(D, G, N, j, Z) {
    E.transform(D, function() {
      var Q = r.apply(this, arguments), le = this.__zoom, z = j == null ? T(Q) : typeof j == "function" ? j.apply(this, arguments) : j;
      return i(ds.translate(z[0], z[1]).scale(le.k).translate(
        typeof G == "function" ? -G.apply(this, arguments) : -G,
        typeof N == "function" ? -N.apply(this, arguments) : -N
      ), Q, c);
    }, j, Z);
  };
  function M(D, G) {
    return G = Math.max(s[0], Math.min(s[1], G)), G === D.k ? D : new Jn(G, D.x, D.y);
  }
  function S(D, G, N) {
    var j = G[0] - N[0] * D.k, Z = G[1] - N[1] * D.k;
    return j === D.x && Z === D.y ? D : new Jn(D.k, j, Z);
  }
  function T(D) {
    return [(+D[0][0] + +D[1][0]) / 2, (+D[0][1] + +D[1][1]) / 2];
  }
  function O(D, G, N, j) {
    D.on("start.zoom", function() {
      C(this, arguments).event(j).start();
    }).on("interrupt.zoom end.zoom", function() {
      C(this, arguments).event(j).end();
    }).tween("zoom", function() {
      var Z = this, Q = arguments, le = C(Z, Q).event(j), z = r.apply(Z, Q), V = N == null ? T(z) : typeof N == "function" ? N.apply(Z, Q) : N, ie = Math.max(z[1][0] - z[0][0], z[1][1] - z[0][1]), L = Z.__zoom, I = typeof G == "function" ? G.apply(Z, Q) : G, P = g(L.invert(V).concat(ie / L.k), I.invert(V).concat(ie / I.k));
      return function(ae) {
        if (ae === 1) ae = I;
        else {
          var W = P(ae), se = ie / W[2];
          ae = new Jn(se, V[0] - W[0] * se, V[1] - W[1] * se);
        }
        le.zoom(null, ae);
      };
    });
  }
  function C(D, G, N) {
    return !N && D.__zooming || new R(D, G);
  }
  function R(D, G) {
    this.that = D, this.args = G, this.active = 0, this.sourceEvent = null, this.extent = r.apply(D, G), this.taps = 0;
  }
  R.prototype = {
    event: function(D) {
      return D && (this.sourceEvent = D), this;
    },
    start: function() {
      return ++this.active === 1 && (this.that.__zooming = this, this.emit("start")), this;
    },
    zoom: function(D, G) {
      return this.mouse && D !== "mouse" && (this.mouse[1] = G.invert(this.mouse[0])), this.touch0 && D !== "touch" && (this.touch0[1] = G.invert(this.touch0[0])), this.touch1 && D !== "touch" && (this.touch1[1] = G.invert(this.touch1[0])), this.that.__zoom = G, this.emit("zoom"), this;
    },
    end: function() {
      return --this.active === 0 && (delete this.that.__zooming, this.emit("end")), this;
    },
    emit: function(D) {
      var G = Zt(this.that).datum();
      h.call(
        D,
        this.that,
        new j5(D, {
          sourceEvent: this.sourceEvent,
          target: E,
          transform: this.that.__zoom,
          dispatch: h
        }),
        G
      );
    }
  };
  function H(D, ...G) {
    if (!t.apply(this, arguments)) return;
    var N = C(this, G).event(D), j = this.__zoom, Z = Math.max(s[0], Math.min(s[1], j.k * Math.pow(2, u.apply(this, arguments)))), Q = cn(D);
    if (N.wheel)
      (N.mouse[0][0] !== Q[0] || N.mouse[0][1] !== Q[1]) && (N.mouse[1] = j.invert(N.mouse[0] = Q)), clearTimeout(N.wheel);
    else {
      if (j.k === Z) return;
      N.mouse = [Q, j.invert(Q)], Oo(this), N.start();
    }
    _u(D), N.wheel = setTimeout(le, _), N.zoom("mouse", i(S(M(j, Z), N.mouse[0], N.mouse[1]), N.extent, c));
    function le() {
      N.wheel = null, N.end();
    }
  }
  function B(D, ...G) {
    if (m || !t.apply(this, arguments)) return;
    var N = D.currentTarget, j = C(this, G, !0).event(D), Z = Zt(D.view).on("mousemove.zoom", V, !0).on("mouseup.zoom", ie, !0), Q = cn(D, N), le = D.clientX, z = D.clientY;
    JT(D.view), lp(D), j.mouse = [Q, this.__zoom.invert(Q)], Oo(this), j.start();
    function V(L) {
      if (_u(L), !j.moved) {
        var I = L.clientX - le, P = L.clientY - z;
        j.moved = I * I + P * P > A;
      }
      j.event(L).zoom("mouse", i(S(j.that.__zoom, j.mouse[0] = cn(L, N), j.mouse[1]), j.extent, c));
    }
    function ie(L) {
      Z.on("mousemove.zoom mouseup.zoom", null), PT(L.view, j.moved), _u(L), j.event(L).end();
    }
  }
  function X(D, ...G) {
    if (t.apply(this, arguments)) {
      var N = this.__zoom, j = cn(D.changedTouches ? D.changedTouches[0] : D, this), Z = N.invert(j), Q = N.k * (D.shiftKey ? 0.5 : 2), le = i(S(M(N, Q), j, Z), r.apply(this, G), c);
      _u(D), f > 0 ? Zt(this).transition().duration(f).call(O, le, j, D) : Zt(this).call(E.transform, le, j, D);
    }
  }
  function Y(D, ...G) {
    if (t.apply(this, arguments)) {
      var N = D.touches, j = N.length, Z = C(this, G, D.changedTouches.length === j).event(D), Q, le, z, V;
      for (lp(D), le = 0; le < j; ++le)
        z = N[le], V = cn(z, this), V = [V, this.__zoom.invert(V), z.identifier], Z.touch0 ? !Z.touch1 && Z.touch0[2] !== V[2] && (Z.touch1 = V, Z.taps = 0) : (Z.touch0 = V, Q = !0, Z.taps = 1 + !!v);
      v && (v = clearTimeout(v)), Q && (Z.taps < 2 && (p = V[0], v = setTimeout(function() {
        v = null;
      }, b)), Oo(this), Z.start());
    }
  }
  function F(D, ...G) {
    if (this.__zooming) {
      var N = C(this, G).event(D), j = D.changedTouches, Z = j.length, Q, le, z, V;
      for (_u(D), Q = 0; Q < Z; ++Q)
        le = j[Q], z = cn(le, this), N.touch0 && N.touch0[2] === le.identifier ? N.touch0[0] = z : N.touch1 && N.touch1[2] === le.identifier && (N.touch1[0] = z);
      if (le = N.that.__zoom, N.touch1) {
        var ie = N.touch0[0], L = N.touch0[1], I = N.touch1[0], P = N.touch1[1], ae = (ae = I[0] - ie[0]) * ae + (ae = I[1] - ie[1]) * ae, W = (W = P[0] - L[0]) * W + (W = P[1] - L[1]) * W;
        le = M(le, Math.sqrt(ae / W)), z = [(ie[0] + I[0]) / 2, (ie[1] + I[1]) / 2], V = [(L[0] + P[0]) / 2, (L[1] + P[1]) / 2];
      } else if (N.touch0) z = N.touch0[0], V = N.touch0[1];
      else return;
      N.zoom("touch", i(S(le, z, V), N.extent, c));
    }
  }
  function K(D, ...G) {
    if (this.__zooming) {
      var N = C(this, G).event(D), j = D.changedTouches, Z = j.length, Q, le;
      for (lp(D), m && clearTimeout(m), m = setTimeout(function() {
        m = null;
      }, b), Q = 0; Q < Z; ++Q)
        le = j[Q], N.touch0 && N.touch0[2] === le.identifier ? delete N.touch0 : N.touch1 && N.touch1[2] === le.identifier && delete N.touch1;
      if (N.touch1 && !N.touch0 && (N.touch0 = N.touch1, delete N.touch1), N.touch0) N.touch0[1] = this.__zoom.invert(N.touch0[0]);
      else if (N.end(), N.taps === 2 && (le = cn(le, this), Math.hypot(p[0] - le[0], p[1] - le[1]) < w)) {
        var z = Zt(this).on("dblclick.zoom");
        z && z.apply(this, arguments);
      }
    }
  }
  return E.wheelDelta = function(D) {
    return arguments.length ? (u = typeof D == "function" ? D : Eo(+D), E) : u;
  }, E.filter = function(D) {
    return arguments.length ? (t = typeof D == "function" ? D : Eo(!!D), E) : t;
  }, E.touchable = function(D) {
    return arguments.length ? (o = typeof D == "function" ? D : Eo(!!D), E) : o;
  }, E.extent = function(D) {
    return arguments.length ? (r = typeof D == "function" ? D : Eo([[+D[0][0], +D[0][1]], [+D[1][0], +D[1][1]]]), E) : r;
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
}, Ou = [
  [Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY],
  [Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY]
], vM = ["Enter", " ", "Escape"], yM = {
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
var zu;
(function(t) {
  t.Partial = "partial", t.Full = "full";
})(zu || (zu = {}));
const pM = {
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
var zr;
(function(t) {
  t.Bezier = "default", t.Straight = "straight", t.Step = "step", t.SmoothStep = "smoothstep", t.SimpleBezier = "simplebezier";
})(zr || (zr = {}));
var Vo;
(function(t) {
  t.Arrow = "arrow", t.ArrowClosed = "arrowclosed";
})(Vo || (Vo = {}));
var _e;
(function(t) {
  t.Left = "left", t.Top = "top", t.Right = "right", t.Bottom = "bottom";
})(_e || (_e = {}));
const I2 = {
  [_e.Left]: _e.Right,
  [_e.Right]: _e.Left,
  [_e.Top]: _e.Bottom,
  [_e.Bottom]: _e.Top
};
function mM(t) {
  return t === null ? null : t ? "valid" : "invalid";
}
const bM = (t) => !!t && typeof t == "object" && "id" in t && "source" in t && "target" in t, X5 = (t) => !!t && typeof t == "object" && "id" in t && "position" in t && !("source" in t) && !("target" in t), rm = (t) => !!t && typeof t == "object" && "id" in t && "internals" in t && !("source" in t) && !("target" in t), Iu = (t, r = [0, 0]) => {
  const { width: i, height: u } = qn(t), o = t.origin ?? r, s = i * o[0], c = u * o[1];
  return {
    x: t.position.x - s,
    y: t.position.y - c
  };
}, I5 = (t, r = { nodeOrigin: [0, 0] }) => {
  if (t.length === 0)
    return { x: 0, y: 0, width: 0, height: 0 };
  const i = t.reduce((u, o) => {
    const s = typeof o == "string";
    let c = !r.nodeLookup && !s ? o : void 0;
    r.nodeLookup && (c = s ? r.nodeLookup.get(o) : rm(o) ? o : r.nodeLookup.get(o.id));
    const f = c ? Yo(c, r.nodeOrigin) : { x: 0, y: 0, x2: 0, y2: 0 };
    return hs(u, f);
  }, { x: 1 / 0, y: 1 / 0, x2: -1 / 0, y2: -1 / 0 });
  return gs(i);
}, Qu = (t, r = {}) => {
  let i = { x: 1 / 0, y: 1 / 0, x2: -1 / 0, y2: -1 / 0 }, u = !1;
  return t.forEach((o) => {
    (r.filter === void 0 || r.filter(o)) && (i = hs(i, Yo(o)), u = !0);
  }), u ? gs(i) : { x: 0, y: 0, width: 0, height: 0 };
}, am = (t, r, [i, u, o] = [0, 0, 1], s = !1, c = !1) => {
  const f = (r.x - i) / o, g = (r.y - u) / o, h = r.width / o, v = r.height / o, p = [];
  for (const m of t.values()) {
    const { measured: b, selectable: _ = !0, hidden: A = !1 } = m;
    if (c && !_ || A)
      continue;
    const w = b.width ?? m.width ?? m.initialWidth ?? 0, E = b.height ?? m.height ?? m.initialHeight ?? 0, { x: M, y: S } = m.internals.positionAbsolute, T = EM(f, g, h, v, M, S, w, E), O = w * E, C = s && T > 0;
    (!m.internals.handleBounds || C || T >= O || m.dragging) && p.push(m);
  }
  return p;
}, Q5 = (t, r) => {
  const i = /* @__PURE__ */ new Set();
  return t.forEach((u) => {
    i.add(u.id);
  }), r.filter((u) => i.has(u.source) || i.has(u.target));
};
function Z5(t, r) {
  const i = /* @__PURE__ */ new Map(), u = r != null && r.nodes ? new Set(r.nodes.map((o) => o.id)) : null;
  return t.forEach((o) => {
    let s;
    if (r != null && r.includeHiddenNodes) {
      const { width: c, height: f } = qn(o);
      s = c > 0 && f > 0;
    } else
      s = !!(o.measured.width && o.measured.height && !o.hidden);
    s && (!u || u.has(o.id)) && i.set(o.id, o);
  }), i;
}
async function K5({ nodes: t, width: r, height: i, panZoom: u, minZoom: o, maxZoom: s }, c) {
  if (t.size === 0)
    return !0;
  const f = Z5(t, c), g = Qu(f), h = um(g, r, i, (c == null ? void 0 : c.minZoom) ?? o, (c == null ? void 0 : c.maxZoom) ?? s, (c == null ? void 0 : c.padding) ?? 0.1);
  return await u.setViewport(h, {
    duration: c == null ? void 0 : c.duration,
    ease: c == null ? void 0 : c.ease,
    interpolate: c == null ? void 0 : c.interpolate
  }), !0;
}
function _M({ nodeId: t, nextPosition: r, nodeLookup: i, nodeOrigin: u = [0, 0], nodeExtent: o, onError: s }) {
  const c = i.get(t), f = c.parentId ? i.get(c.parentId) : void 0, { x: g, y: h } = f ? f.internals.positionAbsolute : { x: 0, y: 0 }, v = c.origin ?? u;
  let p = c.extent || o;
  if (c.extent === "parent" && !c.expandParent)
    if (!f)
      s == null || s("005", vn.error005());
    else {
      const b = f.measured.width, _ = f.measured.height;
      b && _ && (p = [
        [g, h],
        [g + b, h + _]
      ]);
    }
  else f && sa(c.extent) && (p = [
    [c.extent[0][0] + g, c.extent[0][1] + h],
    [c.extent[1][0] + g, c.extent[1][1] + h]
  ]);
  const m = sa(p) ? oa(r, p, c.measured) : r;
  return (c.measured.width === void 0 || c.measured.height === void 0) && (s == null || s("015", vn.error015())), {
    position: {
      x: m.x - g + (c.measured.width ?? 0) * v[0],
      y: m.y - h + (c.measured.height ?? 0) * v[1]
    },
    positionAbsolute: m
  };
}
async function $5({ nodesToRemove: t = [], edgesToRemove: r = [], nodes: i, edges: u, onBeforeDelete: o }) {
  const s = new Set(t.map((m) => m.id)), c = [];
  for (const m of i) {
    if (m.deletable === !1)
      continue;
    const b = s.has(m.id), _ = !b && m.parentId && c.find((A) => A.id === m.parentId);
    (b || _) && c.push(m);
  }
  const f = new Set(r.map((m) => m.id)), g = u.filter((m) => m.deletable !== !1), v = Q5(c, g);
  for (const m of g)
    f.has(m.id) && !v.find((_) => _.id === m.id) && v.push(m);
  if (!o)
    return {
      edges: v,
      nodes: c
    };
  const p = await o({
    nodes: c,
    edges: v
  });
  return typeof p == "boolean" ? p ? { edges: v, nodes: c } : { edges: [], nodes: [] } : p;
}
const fi = (t, r = 0, i = 1) => Math.min(Math.max(t, r), i), oa = (t = { x: 0, y: 0 }, r, i) => ({
  x: fi(t.x, r[0][0], r[1][0] - ((i == null ? void 0 : i.width) ?? 0)),
  y: fi(t.y, r[0][1], r[1][1] - ((i == null ? void 0 : i.height) ?? 0))
});
function xM(t, r, i) {
  const { width: u, height: o } = qn(i), { x: s, y: c } = i.internals.positionAbsolute;
  return oa(t, [
    [s, c],
    [s + u, c + o]
  ], r);
}
const Q2 = (t, r, i) => t < r ? fi(Math.abs(t - r), 1, r) / r : t > i ? -fi(Math.abs(t - i), 1, r) / r : 0, im = (t, r, i = 15, u = 40) => {
  const o = Q2(t.x, u, r.width - u) * i, s = Q2(t.y, u, r.height - u) * i;
  return [o, s];
}, hs = (t, r) => ({
  x: Math.min(t.x, r.x),
  y: Math.min(t.y, r.y),
  x2: Math.max(t.x2, r.x2),
  y2: Math.max(t.y2, r.y2)
}), Ap = ({ x: t, y: r, width: i, height: u }) => ({
  x: t,
  y: r,
  x2: t + i,
  y2: r + u
}), gs = ({ x: t, y: r, x2: i, y2: u }) => ({
  x: t,
  y: r,
  width: i - t,
  height: u - r
}), Du = (t, r = [0, 0]) => {
  var o, s;
  const { x: i, y: u } = rm(t) ? t.internals.positionAbsolute : Iu(t, r);
  return {
    x: i,
    y: u,
    width: ((o = t.measured) == null ? void 0 : o.width) ?? t.width ?? t.initialWidth ?? 0,
    height: ((s = t.measured) == null ? void 0 : s.height) ?? t.height ?? t.initialHeight ?? 0
  };
}, Yo = (t, r = [0, 0]) => {
  var o, s;
  const { x: i, y: u } = rm(t) ? t.internals.positionAbsolute : Iu(t, r);
  return {
    x: i,
    y: u,
    x2: i + (((o = t.measured) == null ? void 0 : o.width) ?? t.width ?? t.initialWidth ?? 0),
    y2: u + (((s = t.measured) == null ? void 0 : s.height) ?? t.height ?? t.initialHeight ?? 0)
  };
}, SM = (t, r) => gs(hs(Ap(t), Ap(r))), EM = (t, r, i, u, o, s, c, f) => {
  const g = Math.max(0, Math.min(t + i, o + c) - Math.max(t, o)), h = Math.max(0, Math.min(r + u, s + f) - Math.max(r, s));
  return Math.ceil(g * h);
}, ko = (t, r) => EM(t.x, t.y, t.width, t.height, r.x, r.y, r.width, r.height), Z2 = (t) => dn(t.width) && dn(t.height) && dn(t.x) && dn(t.y), dn = (t) => !isNaN(t) && isFinite(t), wM = (t, r) => (i, u) => {
}, Zu = (t, r = [1, 1]) => ({
  x: r[0] * Math.round(t.x / r[0]),
  y: r[1] * Math.round(t.y / r[1])
}), Ku = ({ x: t, y: r }, [i, u, o], s = !1, c = [1, 1]) => {
  const f = {
    x: (t - i) / o,
    y: (r - u) / o
  };
  return s ? Zu(f, c) : f;
}, di = ({ x: t, y: r }, [i, u, o]) => ({
  x: t * o + i,
  y: r * o + u
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
function F5(t, r, i) {
  if (typeof t == "string" || typeof t == "number") {
    const u = ri(t, i), o = ri(t, r);
    return {
      top: u,
      right: o,
      bottom: u,
      left: o,
      x: o * 2,
      y: u * 2
    };
  }
  if (typeof t == "object") {
    const u = ri(t.top ?? t.y ?? 0, i), o = ri(t.bottom ?? t.y ?? 0, i), s = ri(t.left ?? t.x ?? 0, r), c = ri(t.right ?? t.x ?? 0, r);
    return { top: u, right: c, bottom: o, left: s, x: s + c, y: u + o };
  }
  return { top: 0, right: 0, bottom: 0, left: 0, x: 0, y: 0 };
}
function J5(t, r, i, u, o, s) {
  const { x: c, y: f } = di(t, [r, i, u]), { x: g, y: h } = di({ x: t.x + t.width, y: t.y + t.height }, [r, i, u]), v = o - g, p = s - h;
  return {
    left: Math.floor(c),
    top: Math.floor(f),
    right: Math.floor(v),
    bottom: Math.floor(p)
  };
}
const um = (t, r, i, u, o, s) => {
  const c = F5(s, r, i), f = (r - c.x) / t.width, g = (i - c.y) / t.height, h = Math.min(f, g), v = fi(h, u, o), p = t.x + t.width / 2, m = t.y + t.height / 2, b = r / 2 - p * v, _ = i / 2 - m * v, A = J5(t, b, _, v, r, i), w = {
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
}, Hu = () => {
  var t;
  return typeof navigator < "u" && ((t = navigator == null ? void 0 : navigator.userAgent) == null ? void 0 : t.indexOf("Mac")) >= 0;
};
function sa(t) {
  return t != null && t !== "parent";
}
function qn(t) {
  var r, i;
  return {
    width: ((r = t.measured) == null ? void 0 : r.width) ?? t.width ?? t.initialWidth ?? 0,
    height: ((i = t.measured) == null ? void 0 : i.height) ?? t.height ?? t.initialHeight ?? 0
  };
}
function AM(t) {
  var r, i;
  return (((r = t.measured) == null ? void 0 : r.width) ?? t.width ?? t.initialWidth) !== void 0 && (((i = t.measured) == null ? void 0 : i.height) ?? t.height ?? t.initialHeight) !== void 0;
}
function TM(t, r = { width: 0, height: 0 }, i, u, o) {
  const s = { ...t }, c = u.get(i);
  if (c) {
    const f = c.origin || o;
    s.x += c.internals.positionAbsolute.x - (r.width ?? 0) * f[0], s.y += c.internals.positionAbsolute.y - (r.height ?? 0) * f[1];
  }
  return s;
}
function K2(t, r) {
  if (t.size !== r.size)
    return !1;
  for (const i of t)
    if (!r.has(i))
      return !1;
  return !0;
}
function P5() {
  let t, r;
  return { promise: new Promise((u, o) => {
    t = u, r = o;
  }), resolve: t, reject: r };
}
function W5(t) {
  return { ...yM, ...t || {} };
}
function Mu(t, { snapGrid: r = [0, 0], snapToGrid: i = !1, transform: u, containerBounds: o }) {
  const { x: s, y: c } = hn(t), f = Ku({ x: s - ((o == null ? void 0 : o.left) ?? 0), y: c - ((o == null ? void 0 : o.top) ?? 0) }, u), { x: g, y: h } = i ? Zu(f, r) : f;
  return {
    xSnapped: g,
    ySnapped: h,
    ...f
  };
}
const lm = (t) => ({
  width: t.offsetWidth,
  height: t.offsetHeight
}), MM = (t) => {
  var r;
  return ((r = t == null ? void 0 : t.getRootNode) == null ? void 0 : r.call(t)) || (window == null ? void 0 : window.document);
}, e6 = ["INPUT", "SELECT", "TEXTAREA"];
function qM(t) {
  var u, o;
  const r = ((o = (u = t.composedPath) == null ? void 0 : u.call(t)) == null ? void 0 : o[0]) || t.target;
  return (r == null ? void 0 : r.nodeType) !== 1 ? !1 : e6.includes(r.nodeName) || r.hasAttribute("contenteditable") || !!r.closest(".nokey");
}
const CM = (t) => "clientX" in t, hn = (t, r) => {
  var s, c;
  const i = CM(t), u = i ? t.clientX : (s = t.touches) == null ? void 0 : s[0].clientX, o = i ? t.clientY : (c = t.touches) == null ? void 0 : c[0].clientY;
  return {
    x: u - ((r == null ? void 0 : r.left) ?? 0),
    y: o - ((r == null ? void 0 : r.top) ?? 0)
  };
}, $2 = (t, r, i, u, o) => {
  const s = r.querySelectorAll(`.${t}`);
  return !s || !s.length ? null : Array.from(s).map((c) => {
    const f = c.getBoundingClientRect();
    return {
      id: c.getAttribute("data-handleid"),
      type: t,
      nodeId: o,
      position: c.getAttribute("data-handlepos"),
      x: (f.left - i.left) / u,
      y: (f.top - i.top) / u,
      ...lm(c)
    };
  });
};
function RM({ sourceX: t, sourceY: r, targetX: i, targetY: u, sourceControlX: o, sourceControlY: s, targetControlX: c, targetControlY: f }) {
  const g = t * 0.125 + o * 0.375 + c * 0.375 + i * 0.125, h = r * 0.125 + s * 0.375 + f * 0.375 + u * 0.125, v = Math.abs(g - t), p = Math.abs(h - r);
  return [g, h, v, p];
}
function wo(t, r) {
  return t >= 0 ? 0.5 * t : r * 25 * Math.sqrt(-t);
}
function F2({ pos: t, x1: r, y1: i, x2: u, y2: o, c: s }) {
  switch (t) {
    case _e.Left:
      return [r - wo(r - u, s), i];
    case _e.Right:
      return [r + wo(u - r, s), i];
    case _e.Top:
      return [r, i - wo(i - o, s)];
    case _e.Bottom:
      return [r, i + wo(o - i, s)];
  }
}
function NM({ sourceX: t, sourceY: r, sourcePosition: i = _e.Bottom, targetX: u, targetY: o, targetPosition: s = _e.Top, curvature: c = 0.25 }) {
  const [f, g] = F2({
    pos: i,
    x1: t,
    y1: r,
    x2: u,
    y2: o,
    c
  }), [h, v] = F2({
    pos: s,
    x1: u,
    y1: o,
    x2: t,
    y2: r,
    c
  }), [p, m, b, _] = RM({
    sourceX: t,
    sourceY: r,
    targetX: u,
    targetY: o,
    sourceControlX: f,
    sourceControlY: g,
    targetControlX: h,
    targetControlY: v
  });
  return [
    `M${t},${r} C${f},${g} ${h},${v} ${u},${o}`,
    p,
    m,
    b,
    _
  ];
}
function OM({ sourceX: t, sourceY: r, targetX: i, targetY: u }) {
  const o = Math.abs(i - t) / 2, s = i < t ? i + o : i - o, c = Math.abs(u - r) / 2, f = u < r ? u + c : u - c;
  return [s, f, o, c];
}
function t6({ sourceNode: t, targetNode: r, selected: i = !1, zIndex: u = 0, elevateOnSelect: o = !1, zIndexMode: s = "basic" }) {
  if (s === "manual")
    return u;
  const c = o && i ? u + 1e3 : u, f = Math.max(t.parentId || o && t.selected ? t.internals.z : 0, r.parentId || o && r.selected ? r.internals.z : 0);
  return c + f;
}
function n6({ sourceNode: t, targetNode: r, width: i, height: u, transform: o }) {
  const s = hs(Yo(t), Yo(r));
  s.x === s.x2 && (s.x2 += 1), s.y === s.y2 && (s.y2 += 1);
  const c = {
    x: -o[0] / o[2],
    y: -o[1] / o[2],
    width: i / o[2],
    height: u / o[2]
  };
  return ko(c, gs(s)) > 0;
}
const r6 = ({ source: t, sourceHandle: r, target: i, targetHandle: u }) => `xy-edge__${t}${r || ""}-${i}${u || ""}`, a6 = (t, r) => r.some((i) => i.source === t.source && i.target === t.target && (i.sourceHandle === t.sourceHandle || !i.sourceHandle && !t.sourceHandle) && (i.targetHandle === t.targetHandle || !i.targetHandle && !t.targetHandle)), i6 = (t, r, i = {}) => {
  var s;
  if (!t.source || !t.target)
    return (s = i.onError) == null || s.call(i, "006", vn.error006()), r;
  const u = i.getEdgeId || r6;
  let o;
  return bM(t) ? o = { ...t } : o = {
    ...t,
    id: u(t)
  }, a6(o, r) ? r : (o.sourceHandle === null && delete o.sourceHandle, o.targetHandle === null && delete o.targetHandle, r.concat(o));
};
function zM({ sourceX: t, sourceY: r, targetX: i, targetY: u }) {
  const [o, s, c, f] = OM({
    sourceX: t,
    sourceY: r,
    targetX: i,
    targetY: u
  });
  return [`M ${t},${r}L ${i},${u}`, o, s, c, f];
}
const J2 = {
  [_e.Left]: { x: -1, y: 0 },
  [_e.Right]: { x: 1, y: 0 },
  [_e.Top]: { x: 0, y: -1 },
  [_e.Bottom]: { x: 0, y: 1 }
}, u6 = ({ source: t, sourcePosition: r = _e.Bottom, target: i }) => r === _e.Left || r === _e.Right ? t.x < i.x ? { x: 1, y: 0 } : { x: -1, y: 0 } : t.y < i.y ? { x: 0, y: 1 } : { x: 0, y: -1 }, P2 = (t, r) => Math.sqrt(Math.pow(r.x - t.x, 2) + Math.pow(r.y - t.y, 2));
function l6({ source: t, sourcePosition: r = _e.Bottom, target: i, targetPosition: u = _e.Top, center: o, offset: s, stepPosition: c }) {
  const f = J2[r], g = J2[u], h = { x: t.x + f.x * s, y: t.y + f.y * s }, v = { x: i.x + g.x * s, y: i.y + g.y * s }, p = u6({
    source: h,
    sourcePosition: r,
    target: v
  }), m = p.x !== 0 ? "x" : "y", b = p[m];
  let _ = [], A, w;
  const E = { x: 0, y: 0 }, M = { x: 0, y: 0 }, [, , S, T] = OM({
    sourceX: t.x,
    sourceY: t.y,
    targetX: i.x,
    targetY: i.y
  });
  if (f[m] * g[m] === -1) {
    m === "x" ? (A = o.x ?? h.x + (v.x - h.x) * c, w = o.y ?? (h.y + v.y) / 2) : (A = o.x ?? (h.x + v.x) / 2, w = o.y ?? h.y + (v.y - h.y) * c);
    const H = [
      { x: A, y: h.y },
      { x: A, y: v.y }
    ], B = [
      { x: h.x, y: w },
      { x: v.x, y: w }
    ];
    f[m] === b ? _ = m === "x" ? H : B : _ = m === "x" ? B : H;
  } else {
    const H = [{ x: h.x, y: v.y }], B = [{ x: v.x, y: h.y }];
    if (m === "x" ? _ = f.x === b ? B : H : _ = f.y === b ? H : B, r === u) {
      const D = Math.abs(t[m] - i[m]);
      if (D <= s) {
        const G = Math.min(s - 1, s - D);
        f[m] === b ? E[m] = (h[m] > t[m] ? -1 : 1) * G : M[m] = (v[m] > i[m] ? -1 : 1) * G;
      }
    }
    if (r !== u) {
      const D = m === "x" ? "y" : "x", G = f[m] === g[D], N = h[D] > v[D], j = h[D] < v[D];
      (f[m] === 1 && (!G && N || G && j) || f[m] !== 1 && (!G && j || G && N)) && (_ = m === "x" ? H : B);
    }
    const X = { x: h.x + E.x, y: h.y + E.y }, Y = { x: v.x + M.x, y: v.y + M.y }, F = Math.max(Math.abs(X.x - _[0].x), Math.abs(Y.x - _[0].x)), K = Math.max(Math.abs(X.y - _[0].y), Math.abs(Y.y - _[0].y));
    F >= K ? (A = (X.x + Y.x) / 2, w = _[0].y) : (A = _[0].x, w = (X.y + Y.y) / 2);
  }
  const O = { x: h.x + E.x, y: h.y + E.y }, C = { x: v.x + M.x, y: v.y + M.y };
  return [[
    t,
    // we only want to add the gapped source/target if they are different from the first/last point to avoid duplicates which can cause issues with the bends
    ...O.x !== _[0].x || O.y !== _[0].y ? [O] : [],
    ..._,
    ...C.x !== _[_.length - 1].x || C.y !== _[_.length - 1].y ? [C] : [],
    i
  ], A, w, S, T];
}
function o6(t, r, i, u) {
  const o = Math.min(P2(t, r) / 2, P2(r, i) / 2, u), { x: s, y: c } = r;
  if (t.x === s && s === i.x || t.y === c && c === i.y)
    return `L${s} ${c}`;
  if (t.y === c) {
    const h = t.x < i.x ? -1 : 1, v = t.y < i.y ? 1 : -1;
    return `L ${s + o * h},${c}Q ${s},${c} ${s},${c + o * v}`;
  }
  const f = t.x < i.x ? 1 : -1, g = t.y < i.y ? -1 : 1;
  return `L ${s},${c + o * g}Q ${s},${c} ${s + o * f},${c}`;
}
function Tp({ sourceX: t, sourceY: r, sourcePosition: i = _e.Bottom, targetX: u, targetY: o, targetPosition: s = _e.Top, borderRadius: c = 5, centerX: f, centerY: g, offset: h = 20, stepPosition: v = 0.5 }) {
  const [p, m, b, _, A] = l6({
    source: { x: t, y: r },
    sourcePosition: i,
    target: { x: u, y: o },
    targetPosition: s,
    center: { x: f, y: g },
    offset: h,
    stepPosition: v
  });
  let w = `M${p[0].x} ${p[0].y}`;
  for (let E = 1; E < p.length - 1; E++)
    w += o6(p[E - 1], p[E], p[E + 1], c);
  return w += `L${p[p.length - 1].x} ${p[p.length - 1].y}`, [w, m, b, _, A];
}
function W2(t) {
  var r;
  return t && !!(t.internals.handleBounds || (r = t.handles) != null && r.length) && !!(t.measured.width || t.width || t.initialWidth);
}
function s6(t) {
  var p;
  const { sourceNode: r, targetNode: i } = t;
  if (!W2(r) || !W2(i))
    return null;
  const u = r.internals.handleBounds || eA(r.handles), o = i.internals.handleBounds || eA(i.handles), s = tA((u == null ? void 0 : u.source) ?? [], t.sourceHandle), c = tA(
    // when connection type is loose we can define all handles as sources and connect source -> source
    t.connectionMode === ci.Strict ? (o == null ? void 0 : o.target) ?? [] : ((o == null ? void 0 : o.target) ?? []).concat((o == null ? void 0 : o.source) ?? []),
    t.targetHandle
  );
  if (!s || !c)
    return (p = t.onError) == null || p.call(t, "008", vn.error008(s ? "target" : "source", {
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
function eA(t) {
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
  const o = ((r == null ? void 0 : r.x) ?? 0) + t.internals.positionAbsolute.x, s = ((r == null ? void 0 : r.y) ?? 0) + t.internals.positionAbsolute.y, { width: c, height: f } = r ?? qn(t);
  if (u)
    return { x: o + c / 2, y: s + f / 2 };
  switch ((r == null ? void 0 : r.position) ?? i) {
    case _e.Top:
      return { x: o + c / 2, y: s };
    case _e.Right:
      return { x: o + c, y: s + f / 2 };
    case _e.Bottom:
      return { x: o + c / 2, y: s + f };
    case _e.Left:
      return { x: o, y: s + f / 2 };
  }
}
function tA(t, r) {
  return t && (r ? t.find((i) => i.id === r) : t[0]) || null;
}
function Mp(t, r) {
  return t ? typeof t == "string" ? t : `${r ? `${r}__` : ""}${Object.keys(t).sort().map((u) => `${u}=${t[u]}`).join("&")}` : "";
}
function c6(t, { id: r, defaultColor: i, defaultMarkerStart: u, defaultMarkerEnd: o }) {
  const s = /* @__PURE__ */ new Set();
  return t.reduce((c, f) => ([f.markerStart || u, f.markerEnd || o].forEach((g) => {
    if (g && typeof g == "object") {
      const h = Mp(g, r);
      s.has(h) || (c.push({ id: h, color: g.color || i, ...g }), s.add(h));
    }
  }), c), []).sort((c, f) => c.id.localeCompare(f.id));
}
const DM = 1e3, f6 = 10, om = {
  nodeOrigin: [0, 0],
  nodeExtent: Ou,
  elevateNodesOnSelect: !0,
  zIndexMode: "basic",
  defaults: {}
}, d6 = {
  ...om,
  checkEquality: !0
};
function sm(t, r) {
  const i = { ...t };
  for (const u in r)
    r[u] !== void 0 && (i[u] = r[u]);
  return i;
}
function h6(t, r, i) {
  const u = sm(om, i);
  for (const o of t.values())
    if (o.parentId)
      fm(o, t, r, u);
    else {
      const s = Iu(o, u.nodeOrigin), c = sa(o.extent) ? o.extent : u.nodeExtent, f = oa(s, c, qn(o));
      o.internals.positionAbsolute = f;
    }
}
function g6(t, r) {
  if (!t.handles)
    return t.measured ? r == null ? void 0 : r.internals.handleBounds : void 0;
  const i = [], u = [];
  for (const o of t.handles) {
    const s = {
      id: o.id,
      width: o.width ?? 1,
      height: o.height ?? 1,
      nodeId: t.id,
      x: o.x,
      y: o.y,
      position: o.position,
      type: o.type
    };
    o.type === "source" ? i.push(s) : o.type === "target" && u.push(s);
  }
  return {
    source: i,
    target: u
  };
}
function cm(t) {
  return t === "manual";
}
function qp(t, r, i, u = {}) {
  var v, p;
  const o = sm(d6, u), s = { i: 0 }, c = new Map(r), f = o != null && o.elevateNodesOnSelect && !cm(o.zIndexMode) ? DM : 0;
  let g = t.length > 0, h = !1;
  r.clear(), i.clear();
  for (const m of t) {
    let b = c.get(m.id);
    if (o.checkEquality && m === (b == null ? void 0 : b.internals.userNode))
      r.set(m.id, b);
    else {
      const _ = Iu(m, o.nodeOrigin), A = sa(m.extent) ? m.extent : o.nodeExtent, w = oa(_, A, qn(m));
      b = {
        ...o.defaults,
        ...m,
        measured: {
          width: (v = m.measured) == null ? void 0 : v.width,
          height: (p = m.measured) == null ? void 0 : p.height
        },
        internals: {
          positionAbsolute: w,
          // if user re-initializes the node or removes `measured` for whatever reason, we reset the handleBounds so that the node gets re-measured
          handleBounds: g6(m, b),
          z: HM(m, f, o.zIndexMode),
          userNode: m
        }
      }, r.set(m.id, b);
    }
    (b.measured === void 0 || b.measured.width === void 0 || b.measured.height === void 0) && !b.hidden && (g = !1), m.parentId && fm(b, r, i, u, s), h || (h = m.selected ?? !1);
  }
  return { nodesInitialized: g, hasSelectedNodes: h };
}
function v6(t, r) {
  if (!t.parentId)
    return;
  const i = r.get(t.parentId);
  i ? i.set(t.id, t) : r.set(t.parentId, /* @__PURE__ */ new Map([[t.id, t]]));
}
function fm(t, r, i, u, o) {
  const { elevateNodesOnSelect: s, nodeOrigin: c, nodeExtent: f, zIndexMode: g } = sm(om, u), h = t.parentId, v = r.get(h);
  if (!v) {
    console.warn(`Parent node ${h} not found. Please make sure that parent nodes are in front of their child nodes in the nodes array.`);
    return;
  }
  v6(t, i), o && !v.parentId && v.internals.rootParentIndex === void 0 && g === "auto" && (v.internals.rootParentIndex = ++o.i, v.internals.z = v.internals.z + o.i * f6), o && v.internals.rootParentIndex !== void 0 && (o.i = v.internals.rootParentIndex);
  const p = s && !cm(g) ? DM : 0, { x: m, y: b, z: _ } = y6(t, v, c, f, p, g), { positionAbsolute: A } = t.internals, w = m !== A.x || b !== A.y;
  (w || _ !== t.internals.z) && r.set(t.id, {
    ...t,
    internals: {
      ...t.internals,
      positionAbsolute: w ? { x: m, y: b } : A,
      z: _
    }
  });
}
function HM(t, r, i) {
  const u = dn(t.zIndex) ? t.zIndex : 0;
  return cm(i) ? u : u + (t.selected ? r : 0);
}
function y6(t, r, i, u, o, s) {
  const { x: c, y: f } = r.internals.positionAbsolute, g = qn(t), h = Iu(t, i), v = sa(t.extent) ? oa(h, t.extent, g) : h;
  let p = oa({ x: c + v.x, y: f + v.y }, u, g);
  t.extent === "parent" && (p = xM(p, g, r));
  const m = HM(t, o, s), b = r.internals.z ?? 0;
  return {
    x: p.x,
    y: p.y,
    z: b >= m ? b + 1 : m
  };
}
function dm(t, r, i, u = [0, 0]) {
  var c;
  const o = [], s = /* @__PURE__ */ new Map();
  for (const f of t) {
    const g = r.get(f.parentId);
    if (!g)
      continue;
    const h = ((c = s.get(f.parentId)) == null ? void 0 : c.expandedRect) ?? Du(g), v = SM(h, f.rect);
    s.set(f.parentId, { expandedRect: v, parent: g });
  }
  return s.size > 0 && s.forEach(({ expandedRect: f, parent: g }, h) => {
    var S;
    const v = g.internals.positionAbsolute, p = qn(g), m = g.origin ?? u, b = f.x < v.x ? Math.round(Math.abs(v.x - f.x)) : 0, _ = f.y < v.y ? Math.round(Math.abs(v.y - f.y)) : 0, A = Math.max(p.width, Math.round(f.width)), w = Math.max(p.height, Math.round(f.height)), E = (A - p.width) * m[0], M = (w - p.height) * m[1];
    (b > 0 || _ > 0 || E || M) && (o.push({
      id: h,
      type: "position",
      position: {
        x: g.position.x - b + E,
        y: g.position.y - _ + M
      }
    }), (S = i.get(h)) == null || S.forEach((T) => {
      t.some((O) => O.id === T.id) || o.push({
        id: T.id,
        type: "position",
        position: {
          x: T.position.x + b,
          y: T.position.y + _
        }
      });
    })), (p.width < f.width || p.height < f.height || b || _) && o.push({
      id: h,
      type: "dimensions",
      setAttributes: !0,
      dimensions: {
        width: A + (b ? m[0] * b - E : 0),
        height: w + (_ ? m[1] * _ - M : 0)
      }
    });
  }), o;
}
function p6(t, r, i, u, o, s, c) {
  const f = u == null ? void 0 : u.querySelector(".xyflow__viewport");
  let g = !1;
  if (!f)
    return { changes: [], updatedInternals: g };
  const h = [], v = window.getComputedStyle(f), { m22: p } = new window.DOMMatrixReadOnly(v.transform), m = [];
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
    const A = lm(b.nodeElement), w = _.measured.width !== A.width || _.measured.height !== A.height;
    if (!!(A.width && A.height && (w || !_.internals.handleBounds || b.force))) {
      const M = b.nodeElement.getBoundingClientRect(), S = sa(_.extent) ? _.extent : s;
      let { positionAbsolute: T } = _.internals;
      if (_.parentId && _.extent === "parent") {
        const C = r.get(_.parentId);
        C && (T = xM(T, A, C));
      } else S && (T = oa(T, S, A));
      const O = {
        ..._,
        measured: A,
        internals: {
          ..._.internals,
          positionAbsolute: T,
          handleBounds: {
            source: $2("source", b.nodeElement, M, p, _.id),
            target: $2("target", b.nodeElement, M, p, _.id)
          }
        }
      };
      r.set(_.id, O), _.parentId && fm(O, r, i, { nodeOrigin: o, zIndexMode: c }), g = !0, w && (h.push({
        id: _.id,
        type: "dimensions",
        dimensions: A
      }), _.expandParent && _.parentId && m.push({
        id: _.id,
        parentId: _.parentId,
        rect: Du(O, o)
      }));
    }
  }
  if (m.length > 0) {
    const b = dm(m, r, i, o);
    h.push(...b);
  }
  return { changes: h, updatedInternals: g };
}
async function m6({ delta: t, panZoom: r, transform: i, translateExtent: u, width: o, height: s }) {
  if (!r || !t.x && !t.y)
    return !1;
  const c = await r.setViewportConstrained({
    x: i[0] + t.x,
    y: i[1] + t.y,
    zoom: i[2]
  }, [
    [0, 0],
    [o, s]
  ], u);
  return !!c && (c.x !== i[0] || c.y !== i[1] || c.k !== i[2]);
}
function nA(t, r, i, u, o, s) {
  let c = o;
  const f = u.get(c) || /* @__PURE__ */ new Map();
  u.set(c, f.set(i, r)), c = `${o}-${t}`;
  const g = u.get(c) || /* @__PURE__ */ new Map();
  if (u.set(c, g.set(i, r)), s) {
    c = `${o}-${t}-${s}`;
    const h = u.get(c) || /* @__PURE__ */ new Map();
    u.set(c, h.set(i, r));
  }
}
function LM(t, r, i) {
  t.clear(), r.clear();
  for (const u of i) {
    const { source: o, target: s, sourceHandle: c = null, targetHandle: f = null } = u, g = { edgeId: u.id, source: o, target: s, sourceHandle: c, targetHandle: f }, h = `${o}-${c}--${s}-${f}`, v = `${s}-${f}--${o}-${c}`;
    nA("source", g, v, t, o, c), nA("target", g, h, t, s, f), r.set(u.id, u);
  }
}
function BM(t, r) {
  if (!t.parentId)
    return !1;
  const i = r.get(t.parentId);
  return i ? i.selected ? !0 : BM(i, r) : !1;
}
function rA(t, r, i) {
  var o;
  let u = t;
  do {
    if ((o = u == null ? void 0 : u.matches) != null && o.call(u, r))
      return !0;
    if (u === i)
      return !1;
    u = u == null ? void 0 : u.parentElement;
  } while (u);
  return !1;
}
function b6(t, r, i, u) {
  const o = /* @__PURE__ */ new Map();
  for (const [s, c] of t)
    if ((c.selected || c.id === u) && (!c.parentId || !BM(c, t)) && (c.draggable || r && typeof c.draggable > "u")) {
      const f = t.get(s);
      f && o.set(s, {
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
  return o;
}
function op({ nodeId: t, dragItems: r, nodeLookup: i, dragging: u = !0 }) {
  var c, f, g;
  const o = [];
  for (const [h, v] of r) {
    const p = (c = i.get(h)) == null ? void 0 : c.internals.userNode;
    p && o.push({
      ...p,
      position: v.position,
      dragging: u
    });
  }
  if (!t)
    return [o[0], o];
  const s = (f = i.get(t)) == null ? void 0 : f.internals.userNode;
  return [
    s ? {
      ...s,
      position: ((g = r.get(t)) == null ? void 0 : g.position) || s.position,
      dragging: u
    } : o[0],
    o
  ];
}
function _6({ dragItems: t, snapGrid: r, x: i, y: u }) {
  const o = t.values().next().value;
  if (!o)
    return null;
  const s = {
    x: i - o.distance.x,
    y: u - o.distance.y
  }, c = Zu(s, r);
  return {
    x: c.x - s.x,
    y: c.y - s.y
  };
}
function x6({ onNodeMouseDown: t, getStoreItems: r, onDragStart: i, onDrag: u, onDragStop: o }) {
  let s = { x: null, y: null }, c = 0, f = /* @__PURE__ */ new Map(), g = !1, h = { x: 0, y: 0 }, v = null, p = !1, m = null, b = !1, _ = !1, A = null;
  function w({ noDragClassName: M, handleSelector: S, domNode: T, isSelectable: O, nodeId: C, nodeClickDistance: R = 0 }) {
    m = Zt(T);
    function H({ x: F, y: K }) {
      const { nodeLookup: D, nodeExtent: G, snapGrid: N, snapToGrid: j, nodeOrigin: Z, onNodeDrag: Q, onSelectionDrag: le, onError: z, updateNodePositions: V } = r();
      s = { x: F, y: K };
      let ie = !1;
      const L = f.size > 1, I = L && G ? Ap(Qu(f)) : null, P = L && j ? _6({
        dragItems: f,
        snapGrid: N,
        x: F,
        y: K
      }) : null;
      for (const [ae, W] of f) {
        if (!D.has(ae))
          continue;
        let se = { x: F - W.distance.x, y: K - W.distance.y };
        j && (se = P ? {
          x: Math.round(se.x + P.x),
          y: Math.round(se.y + P.y)
        } : Zu(se, N));
        let de = null;
        if (L && G && !W.extent && I) {
          const { positionAbsolute: he } = W.internals, me = he.x - I.x + G[0][0], ge = he.x + W.measured.width - I.x2 + G[1][0], Ae = he.y - I.y + G[0][1], xe = he.y + W.measured.height - I.y2 + G[1][1];
          de = [
            [me, Ae],
            [ge, xe]
          ];
        }
        const { position: ve, positionAbsolute: pe } = _M({
          nodeId: ae,
          nextPosition: se,
          nodeLookup: D,
          nodeExtent: de || G,
          nodeOrigin: Z,
          onError: z
        });
        ie = ie || W.position.x !== ve.x || W.position.y !== ve.y, W.position = ve, W.internals.positionAbsolute = pe;
      }
      if (_ = _ || ie, !!ie && (V(f, !0), A && (u || Q || !C && le))) {
        const [ae, W] = op({
          nodeId: C,
          dragItems: f,
          nodeLookup: D
        });
        u == null || u(A, f, ae, W), Q == null || Q(A, ae, W), C || le == null || le(A, W);
      }
    }
    async function B() {
      if (!v)
        return;
      const { transform: F, panBy: K, autoPanSpeed: D, autoPanOnNodeDrag: G } = r();
      if (!G) {
        g = !1, cancelAnimationFrame(c);
        return;
      }
      const [N, j] = im(h, v, D);
      (N !== 0 || j !== 0) && (s.x = (s.x ?? 0) - N / F[2], s.y = (s.y ?? 0) - j / F[2], await K({ x: N, y: j }) && H(s)), c = requestAnimationFrame(B);
    }
    function X(F) {
      var L;
      const { nodeLookup: K, multiSelectionActive: D, nodesDraggable: G, transform: N, snapGrid: j, snapToGrid: Z, selectNodesOnDrag: Q, onNodeDragStart: le, onSelectionDragStart: z, unselectNodesAndEdges: V } = r();
      p = !0, (!Q || !O) && !D && C && ((L = K.get(C)) != null && L.selected || V()), O && Q && C && (t == null || t(C));
      const ie = Mu(F.sourceEvent, { transform: N, snapGrid: j, snapToGrid: Z, containerBounds: v });
      if (s = ie, f = b6(K, G, ie, C), f.size > 0 && (i || le || !C && z)) {
        const [I, P] = op({
          nodeId: C,
          dragItems: f,
          nodeLookup: K
        });
        i == null || i(F.sourceEvent, f, I, P), le == null || le(F.sourceEvent, I, P), C || z == null || z(F.sourceEvent, P);
      }
    }
    const Y = WT().clickDistance(R).on("start", (F) => {
      const { domNode: K, nodeDragThreshold: D, transform: G, snapGrid: N, snapToGrid: j } = r();
      v = (K == null ? void 0 : K.getBoundingClientRect()) || null, b = !1, _ = !1, A = F.sourceEvent, D === 0 && X(F), s = Mu(F.sourceEvent, { transform: G, snapGrid: N, snapToGrid: j, containerBounds: v }), h = hn(F.sourceEvent, v);
    }).on("drag", (F) => {
      const { autoPanOnNodeDrag: K, transform: D, snapGrid: G, snapToGrid: N, nodeDragThreshold: j, nodeLookup: Z } = r(), Q = Mu(F.sourceEvent, { transform: D, snapGrid: G, snapToGrid: N, containerBounds: v });
      if (A = F.sourceEvent, (F.sourceEvent.type === "touchmove" && F.sourceEvent.touches.length > 1 || // if user deletes a node while dragging, we need to abort the drag to prevent errors
      C && !Z.has(C)) && (b = !0), !b) {
        if (!g && K && p && (g = !0, B()), !p) {
          const le = hn(F.sourceEvent, v), z = le.x - h.x, V = le.y - h.y;
          Math.sqrt(z * z + V * V) > j && X(F);
        }
        (s.x !== Q.xSnapped || s.y !== Q.ySnapped) && f && p && (h = hn(F.sourceEvent, v), H(Q));
      }
    }).on("end", (F) => {
      if (!p || b) {
        b && f.size > 0 && r().updateNodePositions(f, !1);
        return;
      }
      if (g = !1, p = !1, cancelAnimationFrame(c), f.size > 0) {
        const { nodeLookup: K, updateNodePositions: D, onNodeDragStop: G, onSelectionDragStop: N } = r();
        if (_ && (D(f, !1), _ = !1), o || G || !C && N) {
          const [j, Z] = op({
            nodeId: C,
            dragItems: f,
            nodeLookup: K,
            dragging: !1
          });
          o == null || o(F.sourceEvent, f, j, Z), G == null || G(F.sourceEvent, j, Z), C || N == null || N(F.sourceEvent, Z);
        }
      }
    }).filter((F) => {
      const K = F.target;
      return !F.button && (!M || !rA(K, `.${M}`, T)) && (!S || rA(K, S, T));
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
function S6(t, r, i) {
  const u = [], o = {
    x: t.x - i,
    y: t.y - i,
    width: i * 2,
    height: i * 2
  };
  for (const s of r.values())
    ko(o, Du(s)) > 0 && u.push(s);
  return u;
}
const E6 = 250;
function w6(t, r, i, u) {
  var f, g;
  let o = [], s = 1 / 0;
  const c = S6(t, i, r + E6);
  for (const h of c) {
    const v = [...((f = h.internals.handleBounds) == null ? void 0 : f.source) ?? [], ...((g = h.internals.handleBounds) == null ? void 0 : g.target) ?? []];
    for (const p of v) {
      if (u.nodeId === p.nodeId && u.type === p.type && u.id === p.id)
        continue;
      const { x: m, y: b } = ca(h, p, p.position, !0), _ = Math.sqrt(Math.pow(m - t.x, 2) + Math.pow(b - t.y, 2));
      _ > r || (_ < s ? (o = [{ ...p, x: m, y: b }], s = _) : _ === s && o.push({ ...p, x: m, y: b }));
    }
  }
  if (!o.length)
    return null;
  if (o.length > 1) {
    const h = u.type === "source" ? "target" : "source";
    return o.find((v) => v.type === h) ?? o[0];
  }
  return o[0];
}
function jM(t, r, i, u, o, s = !1) {
  var h, v, p;
  const c = u.get(t);
  if (!c)
    return null;
  const f = o === "strict" ? (h = c.internals.handleBounds) == null ? void 0 : h[r] : [...((v = c.internals.handleBounds) == null ? void 0 : v.source) ?? [], ...((p = c.internals.handleBounds) == null ? void 0 : p.target) ?? []], g = (i ? f == null ? void 0 : f.find((m) => m.id === i) : f == null ? void 0 : f[0]) ?? null;
  return g && s ? { ...g, ...ca(c, g, g.position, !0) } : g;
}
function UM(t, r) {
  return t || (r != null && r.classList.contains("target") ? "target" : r != null && r.classList.contains("source") ? "source" : null);
}
function A6(t, r) {
  let i = null;
  return r ? i = !0 : t && !r && (i = !1), i;
}
const GM = () => !0;
function T6(t, { connectionMode: r, connectionRadius: i, handleId: u, nodeId: o, edgeUpdaterType: s, isTarget: c, domNode: f, nodeLookup: g, lib: h, autoPanOnConnect: v, flowId: p, panBy: m, cancelConnection: b, onConnectStart: _, onConnect: A, onConnectEnd: w, isValidConnection: E = GM, onReconnectEnd: M, updateConnection: S, getTransform: T, getFromHandle: O, autoPanSpeed: C, dragThreshold: R = 1, handleDomNode: H }) {
  const B = MM(t.target);
  let X = 0, Y;
  const { x: F, y: K } = hn(t), D = UM(s, H), G = f == null ? void 0 : f.getBoundingClientRect();
  let N = !1;
  if (!G || !D)
    return;
  const j = jM(o, D, u, g, r);
  if (!j)
    return;
  let Z = hn(t, G), Q = !1, le = null, z = !1, V = null;
  function ie() {
    if (!v || !G)
      return;
    const [ve, pe] = im(Z, G, C);
    m({ x: ve, y: pe }), X = requestAnimationFrame(ie);
  }
  const L = {
    ...j,
    nodeId: o,
    type: D,
    position: j.position
  }, I = g.get(o);
  let ae = {
    inProgress: !0,
    isValid: null,
    from: ca(I, L, _e.Left, !0),
    fromHandle: L,
    fromPosition: L.position,
    fromNode: I,
    to: Z,
    toHandle: null,
    toPosition: I2[L.position],
    toNode: null,
    pointer: Z
  };
  function W() {
    N = !0, S(ae), _ == null || _(t, { nodeId: o, handleId: u, handleType: D });
  }
  R === 0 && W();
  function se(ve) {
    if (!N) {
      const { x: xe, y: Pe } = hn(ve), tt = xe - F, xt = Pe - K;
      if (!(tt * tt + xt * xt > R * R))
        return;
      W();
    }
    if (!O() || !L) {
      de(ve);
      return;
    }
    const pe = T();
    Z = hn(ve, G), Y = w6(Ku(Z, pe, !1, [1, 1]), i, g, L), Q || (ie(), Q = !0);
    const he = VM(ve, {
      handle: Y,
      connectionMode: r,
      fromNodeId: o,
      fromHandleId: u,
      fromType: c ? "target" : "source",
      isValidConnection: E,
      doc: B,
      lib: h,
      flowId: p,
      nodeLookup: g
    });
    V = he.handleDomNode, le = he.connection, z = A6(!!Y, he.isValid);
    const me = g.get(o), ge = me ? ca(me, L, _e.Left, !0) : ae.from, Ae = {
      ...ae,
      from: ge,
      isValid: z,
      to: he.toHandle && z ? di({ x: he.toHandle.x, y: he.toHandle.y }, pe) : Z,
      toHandle: he.toHandle,
      toPosition: z && he.toHandle ? he.toHandle.position : I2[L.position],
      toNode: he.toHandle ? g.get(he.toHandle.nodeId) : null,
      pointer: Z
    };
    S(Ae), ae = Ae;
  }
  function de(ve) {
    if (!("touches" in ve && ve.touches.length > 0)) {
      if (N) {
        (Y || V) && le && z && (A == null || A(le));
        const { inProgress: pe, ...he } = ae, me = {
          ...he,
          toPosition: ae.toHandle ? ae.toPosition : null
        };
        w == null || w(ve, me), s && (M == null || M(ve, me));
      }
      b(), cancelAnimationFrame(X), Q = !1, z = !1, le = null, V = null, B.removeEventListener("mousemove", se), B.removeEventListener("mouseup", de), B.removeEventListener("touchmove", se), B.removeEventListener("touchend", de);
    }
  }
  B.addEventListener("mousemove", se), B.addEventListener("mouseup", de), B.addEventListener("touchmove", se), B.addEventListener("touchend", de);
}
function VM(t, { handle: r, connectionMode: i, fromNodeId: u, fromHandleId: o, fromType: s, doc: c, lib: f, flowId: g, isValidConnection: h = GM, nodeLookup: v }) {
  const p = s === "target", m = r ? c.querySelector(`.${f}-flow__handle[data-id="${g}-${r == null ? void 0 : r.nodeId}-${r == null ? void 0 : r.id}-${r == null ? void 0 : r.type}"]`) : null, { x: b, y: _ } = hn(t), A = c.elementFromPoint(b, _), w = A != null && A.classList.contains(`${f}-flow__handle`) ? A : m, E = {
    handleDomNode: w,
    isValid: !1,
    connection: null,
    toHandle: null
  };
  if (w) {
    const M = UM(void 0, w), S = w.getAttribute("data-nodeid"), T = w.getAttribute("data-handleid"), O = w.classList.contains("connectable"), C = w.classList.contains("connectableend");
    if (!S || !M)
      return E;
    const R = {
      source: p ? S : u,
      sourceHandle: p ? T : o,
      target: p ? u : S,
      targetHandle: p ? o : T
    };
    E.connection = R;
    const B = O && C && (i === ci.Strict ? p && M === "source" || !p && M === "target" : S !== u || T !== o);
    E.isValid = B && h(R), E.toHandle = jM(S, M, T, v, i, !0);
  }
  return E;
}
const Cp = {
  onPointerDown: T6,
  isValid: VM
};
function M6({ domNode: t, panZoom: r, getTransform: i, getViewScale: u }) {
  const o = Zt(t);
  function s({ translateExtent: f, width: g, height: h, zoomStep: v = 1, pannable: p = !0, zoomable: m = !0, inversePan: b = !1 }) {
    const _ = (S) => {
      if (S.sourceEvent.type !== "wheel" || !r)
        return;
      const T = i(), O = S.sourceEvent.ctrlKey && Hu() ? 10 : 1, C = -S.sourceEvent.deltaY * (S.sourceEvent.deltaMode === 1 ? 0.05 : S.sourceEvent.deltaMode ? 1 : 2e-3) * v, R = T[2] * Math.pow(2, C * O);
      r.scaleTo(R);
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
      const O = [
        S.sourceEvent.clientX ?? S.sourceEvent.touches[0].clientX,
        S.sourceEvent.clientY ?? S.sourceEvent.touches[0].clientY
      ], C = [O[0] - A[0], O[1] - A[1]];
      A = O;
      const R = u() * Math.max(T[2], Math.log(T[2])) * (b ? -1 : 1), H = {
        x: T[0] - C[0] * R,
        y: T[1] - C[1] * R
      }, B = [
        [0, 0],
        [g, h]
      ];
      r.setViewportConstrained({
        x: H.x,
        y: H.y,
        zoom: T[2]
      }, B, f);
    }, M = gM().on("start", w).on("zoom", p ? E : null).on("zoom.wheel", m ? _ : null);
    o.call(M, {});
  }
  function c() {
    o.on("zoom", null);
  }
  return {
    update: s,
    destroy: c,
    pointer: cn
  };
}
const vs = (t) => ({
  x: t.x,
  y: t.y,
  zoom: t.k
}), sp = ({ x: t, y: r, zoom: i }) => ds.translate(t, r).scale(i), ai = (t, r) => t.target.closest(`.${r}`), YM = (t, r) => r === 2 && Array.isArray(t) && t.includes(2), q6 = (t) => ((t *= 2) <= 1 ? t * t * t : (t -= 2) * t * t + 2) / 2, cp = (t, r = 0, i = q6, u = () => {
}) => {
  const o = typeof r == "number" && r > 0;
  return o || u(), o ? t.transition().duration(r).ease(i).on("end", u) : t;
}, kM = (t) => {
  const r = t.ctrlKey && Hu() ? 10 : 1;
  return -t.deltaY * (t.deltaMode === 1 ? 0.05 : t.deltaMode ? 1 : 2e-3) * r;
};
function C6({ zoomPanValues: t, noWheelClassName: r, d3Selection: i, d3Zoom: u, panOnScrollMode: o, panOnScrollSpeed: s, zoomOnPinch: c, onPanZoomStart: f, onPanZoom: g, onPanZoomEnd: h }) {
  return (v) => {
    if (ai(v, r))
      return v.ctrlKey && v.preventDefault(), !1;
    v.preventDefault(), v.stopImmediatePropagation();
    const p = i.property("__zoom").k || 1;
    if (v.ctrlKey && c) {
      const w = cn(v), E = kM(v), M = p * Math.pow(2, E);
      u.scaleTo(i, M, w, v);
      return;
    }
    const m = v.deltaMode === 1 ? 20 : 1;
    let b = o === ia.Vertical ? 0 : v.deltaX * m, _ = o === ia.Horizontal ? 0 : v.deltaY * m;
    !Hu() && v.shiftKey && o !== ia.Vertical && (b = v.deltaY * m, _ = 0), u.translateBy(
      i,
      -(b / p) * s,
      -(_ / p) * s,
      // @ts-ignore
      { internal: !0 }
    );
    const A = vs(i.property("__zoom"));
    clearTimeout(t.panScrollTimeout), t.isPanScrolling ? g == null || g(v, A) : (t.isPanScrolling = !0, f == null || f(v, A)), t.panScrollTimeout = setTimeout(() => {
      h == null || h(v, A), t.isPanScrolling = !1;
    }, 150);
  };
}
function R6({ noWheelClassName: t, preventScrolling: r, d3ZoomHandler: i }) {
  return function(u, o) {
    const s = u.type === "wheel", c = !r && s && !u.ctrlKey, f = ai(u, t);
    if (u.ctrlKey && s && f && u.preventDefault(), c || f)
      return null;
    u.preventDefault(), i.call(this, u, o);
  };
}
function N6({ zoomPanValues: t, onDraggingChange: r, onPanZoomStart: i }) {
  return (u) => {
    var s, c, f;
    if ((s = u.sourceEvent) != null && s.internal)
      return;
    const o = vs(u.transform);
    t.mouseButton = ((c = u.sourceEvent) == null ? void 0 : c.button) || 0, t.isZoomingOrPanning = !0, t.prevViewport = o, ((f = u.sourceEvent) == null ? void 0 : f.type) === "mousedown" && r(!0), i && (i == null || i(u.sourceEvent, o));
  };
}
function O6({ zoomPanValues: t, panOnDrag: r, onPaneContextMenu: i, onTransformChange: u, onPanZoom: o }) {
  return (s) => {
    var c, f;
    t.usedRightMouseButton = !!(i && YM(r, t.mouseButton ?? 0)), (c = s.sourceEvent) != null && c.sync || u([s.transform.x, s.transform.y, s.transform.k]), o && !((f = s.sourceEvent) != null && f.internal) && (o == null || o(s.sourceEvent, vs(s.transform)));
  };
}
function z6({ zoomPanValues: t, panOnDrag: r, panOnScroll: i, onDraggingChange: u, onPanZoomEnd: o, onPaneContextMenu: s }) {
  return (c) => {
    var f;
    if (!((f = c.sourceEvent) != null && f.internal) && (t.isZoomingOrPanning = !1, s && YM(r, t.mouseButton ?? 0) && !t.usedRightMouseButton && c.sourceEvent && s(c.sourceEvent), t.usedRightMouseButton = !1, u(!1), o)) {
      const g = vs(c.transform);
      t.prevViewport = g, clearTimeout(t.timerId), t.timerId = setTimeout(
        () => {
          o == null || o(c.sourceEvent, g);
        },
        // we need a setTimeout for panOnScroll to suppress multiple end events fired during scroll
        i ? 150 : 0
      );
    }
  };
}
function D6({ zoomActivationKeyPressed: t, zoomOnScroll: r, zoomOnPinch: i, panOnDrag: u, panOnScroll: o, zoomOnDoubleClick: s, userSelectionActive: c, noWheelClassName: f, noPanClassName: g, lib: h, connectionInProgress: v }) {
  return (p) => {
    var w;
    const m = t || r, b = i && p.ctrlKey, _ = p.type === "wheel";
    if (p.button === 1 && p.type === "mousedown" && (ai(p, `${h}-flow__node`) || ai(p, `${h}-flow__edge`)))
      return !0;
    if (!u && !m && !o && !s && !i || c || v && !_ || ai(p, f) && _ || ai(p, g) && (!_ || o && _ && !t) || !i && p.ctrlKey && _)
      return !1;
    if (!i && p.type === "touchstart" && ((w = p.touches) == null ? void 0 : w.length) > 1)
      return p.preventDefault(), !1;
    if (!m && !o && !b && _ || !u && (p.type === "mousedown" || p.type === "touchstart") || Array.isArray(u) && !u.includes(p.button) && p.type === "mousedown")
      return !1;
    const A = Array.isArray(u) && u.includes(p.button) || !p.button || p.button <= 1;
    return (!p.ctrlKey || _) && A;
  };
}
function H6({ domNode: t, minZoom: r, maxZoom: i, translateExtent: u, viewport: o, onPanZoom: s, onPanZoomStart: c, onPanZoomEnd: f, onDraggingChange: g }) {
  const h = {
    isZoomingOrPanning: !1,
    usedRightMouseButton: !1,
    prevViewport: {},
    mouseButton: 0,
    timerId: void 0,
    panScrollTimeout: void 0,
    isPanScrolling: !1
  }, v = t.getBoundingClientRect();
  let p = [
    [0, 0],
    [v.width, v.height]
  ];
  const m = typeof ResizeObserver < "u" ? new ResizeObserver((K) => {
    const D = K[0];
    D && (p = [
      [0, 0],
      [D.contentRect.width, D.contentRect.height]
    ]);
  }) : null;
  m == null || m.observe(t);
  const b = gM().extent(() => p).scaleExtent([r, i]).translateExtent(u), _ = Zt(t).call(b);
  T({
    x: o.x,
    y: o.y,
    zoom: fi(o.zoom, r, i)
  }, [
    [0, 0],
    [v.width, v.height]
  ], u);
  const A = _.on("wheel.zoom"), w = _.on("dblclick.zoom");
  b.wheelDelta(kM);
  async function E(K, D) {
    return _ ? new Promise((G) => {
      b == null || b.interpolate((D == null ? void 0 : D.interpolate) === "linear" ? Tu : Co).transform(cp(_, D == null ? void 0 : D.duration, D == null ? void 0 : D.ease, () => G(!0)), K);
    }) : !1;
  }
  function M({ noWheelClassName: K, noPanClassName: D, onPaneContextMenu: G, userSelectionActive: N, panOnScroll: j, panOnDrag: Z, panOnScrollMode: Q, panOnScrollSpeed: le, preventScrolling: z, zoomOnPinch: V, zoomOnScroll: ie, zoomOnDoubleClick: L, zoomActivationKeyPressed: I, lib: P, onTransformChange: ae, connectionInProgress: W, paneClickDistance: se, selectionOnDrag: de }) {
    N && !h.isZoomingOrPanning && S();
    const ve = j && !I && !N;
    b.clickDistance(de ? 1 / 0 : !dn(se) || se < 0 ? 0 : se);
    const pe = ve ? C6({
      zoomPanValues: h,
      noWheelClassName: K,
      d3Selection: _,
      d3Zoom: b,
      panOnScrollMode: Q,
      panOnScrollSpeed: le,
      zoomOnPinch: V,
      onPanZoomStart: c,
      onPanZoom: s,
      onPanZoomEnd: f
    }) : R6({
      noWheelClassName: K,
      preventScrolling: z,
      d3ZoomHandler: A
    });
    _.on("wheel.zoom", pe, { passive: !1 });
    const he = N6({
      zoomPanValues: h,
      onDraggingChange: g,
      onPanZoomStart: c
    });
    b.on("start", he);
    const me = O6({
      zoomPanValues: h,
      panOnDrag: Z,
      onPaneContextMenu: !!G,
      onPanZoom: s,
      onTransformChange: ae
    });
    b.on("zoom", me);
    const ge = z6({
      zoomPanValues: h,
      panOnDrag: Z,
      panOnScroll: j,
      onPaneContextMenu: G,
      onPanZoomEnd: f,
      onDraggingChange: g
    });
    b.on("end", ge);
    const Ae = D6({
      zoomActivationKeyPressed: I,
      panOnDrag: Z,
      zoomOnScroll: ie,
      panOnScroll: j,
      zoomOnDoubleClick: L,
      zoomOnPinch: V,
      userSelectionActive: N,
      noPanClassName: D,
      noWheelClassName: K,
      lib: P,
      connectionInProgress: W
    });
    b.filter(Ae), L ? _.on("dblclick.zoom", w) : _.on("dblclick.zoom", null);
  }
  function S() {
    b.on("zoom", null);
  }
  async function T(K, D, G) {
    const N = sp(K), j = b == null ? void 0 : b.constrain()(N, D, G);
    return j && await E(j), j;
  }
  async function O(K, D) {
    const G = sp(K);
    return await E(G, D), G;
  }
  function C(K) {
    if (_) {
      const D = sp(K), G = _.property("__zoom");
      (G.k !== K.zoom || G.x !== K.x || G.y !== K.y) && (b == null || b.transform(_, D, null, { sync: !0 }));
    }
  }
  function R() {
    const K = _ ? hM(_.node()) : { x: 0, y: 0, k: 1 };
    return { x: K.x, y: K.y, zoom: K.k };
  }
  async function H(K, D) {
    return _ ? new Promise((G) => {
      b == null || b.interpolate((D == null ? void 0 : D.interpolate) === "linear" ? Tu : Co).scaleTo(cp(_, D == null ? void 0 : D.duration, D == null ? void 0 : D.ease, () => G(!0)), K);
    }) : !1;
  }
  async function B(K, D) {
    return _ ? new Promise((G) => {
      b == null || b.interpolate((D == null ? void 0 : D.interpolate) === "linear" ? Tu : Co).scaleBy(cp(_, D == null ? void 0 : D.duration, D == null ? void 0 : D.ease, () => G(!0)), K);
    }) : !1;
  }
  function X(K) {
    b == null || b.scaleExtent(K);
  }
  function Y(K) {
    b == null || b.translateExtent(K);
  }
  function F(K) {
    const D = !dn(K) || K < 0 ? 0 : K;
    b == null || b.clickDistance(D);
  }
  return {
    update: M,
    destroy: S,
    setViewport: O,
    setViewportConstrained: T,
    getViewport: R,
    scaleTo: H,
    scaleBy: B,
    setScaleExtent: X,
    setTranslateExtent: Y,
    syncViewport: C,
    setClickDistance: F
  };
}
var hi;
(function(t) {
  t.Line = "line", t.Handle = "handle";
})(hi || (hi = {}));
function L6({ width: t, prevWidth: r, height: i, prevHeight: u, affectsX: o, affectsY: s }) {
  const c = t - r, f = i - u, g = [c > 0 ? 1 : c < 0 ? -1 : 0, f > 0 ? 1 : f < 0 ? -1 : 0];
  return c && o && (g[0] = g[0] * -1), f && s && (g[1] = g[1] * -1), g;
}
function aA(t) {
  const r = t.includes("right") || t.includes("left"), i = t.includes("bottom") || t.includes("top"), u = t.includes("left"), o = t.includes("top");
  return {
    isHorizontal: r,
    isVertical: i,
    affectsX: u,
    affectsY: o
  };
}
function Nr(t, r) {
  return Math.max(0, r - t);
}
function Or(t, r) {
  return Math.max(0, t - r);
}
function Ao(t, r, i) {
  return Math.max(0, r - t, t - i);
}
function iA(t, r) {
  return t ? !r : r;
}
function B6(t, r, i, u, o, s, c, f) {
  let { affectsX: g, affectsY: h } = r;
  const { isHorizontal: v, isVertical: p } = r, m = v && p, { xSnapped: b, ySnapped: _ } = i, { minWidth: A, maxWidth: w, minHeight: E, maxHeight: M } = u, { x: S, y: T, width: O, height: C, aspectRatio: R } = t;
  let H = Math.floor(v ? b - t.pointerX : 0), B = Math.floor(p ? _ - t.pointerY : 0);
  const X = O + (g ? -H : H), Y = C + (h ? -B : B), F = -s[0] * O, K = -s[1] * C;
  let D = Ao(X, A, w), G = Ao(Y, E, M);
  if (c) {
    let Z = 0, Q = 0;
    g && H < 0 ? Z = Nr(S + H + F, c[0][0]) : !g && H > 0 && (Z = Or(S + X + F, c[1][0])), h && B < 0 ? Q = Nr(T + B + K, c[0][1]) : !h && B > 0 && (Q = Or(T + Y + K, c[1][1])), D = Math.max(D, Z), G = Math.max(G, Q);
  }
  if (f) {
    let Z = 0, Q = 0;
    g && H > 0 ? Z = Or(S + H, f[0][0]) : !g && H < 0 && (Z = Nr(S + X, f[1][0])), h && B > 0 ? Q = Or(T + B, f[0][1]) : !h && B < 0 && (Q = Nr(T + Y, f[1][1])), D = Math.max(D, Z), G = Math.max(G, Q);
  }
  if (o) {
    if (v) {
      const Z = Ao(X / R, E, M) * R;
      if (D = Math.max(D, Z), c) {
        let Q = 0;
        !g && !h || g && !h && m ? Q = Or(T + K + X / R, c[1][1]) * R : Q = Nr(T + K + (g ? H : -H) / R, c[0][1]) * R, D = Math.max(D, Q);
      }
      if (f) {
        let Q = 0;
        !g && !h || g && !h && m ? Q = Nr(T + X / R, f[1][1]) * R : Q = Or(T + (g ? H : -H) / R, f[0][1]) * R, D = Math.max(D, Q);
      }
    }
    if (p) {
      const Z = Ao(Y * R, A, w) / R;
      if (G = Math.max(G, Z), c) {
        let Q = 0;
        !g && !h || h && !g && m ? Q = Or(S + Y * R + F, c[1][0]) / R : Q = Nr(S + (h ? B : -B) * R + F, c[0][0]) / R, G = Math.max(G, Q);
      }
      if (f) {
        let Q = 0;
        !g && !h || h && !g && m ? Q = Nr(S + Y * R, f[1][0]) / R : Q = Or(S + (h ? B : -B) * R, f[0][0]) / R, G = Math.max(G, Q);
      }
    }
  }
  B = B + (B < 0 ? G : -G), H = H + (H < 0 ? D : -D), o && (m ? X > Y * R ? B = (iA(g, h) ? -H : H) / R : H = (iA(g, h) ? -B : B) * R : v ? (B = H / R, h = g) : (H = B * R, g = h));
  const N = g ? S + H : S, j = h ? T + B : T;
  return {
    width: O + (g ? -H : H),
    height: C + (h ? -B : B),
    x: s[0] * H * (g ? -1 : 1) + N,
    y: s[1] * B * (h ? -1 : 1) + j
  };
}
const XM = { width: 0, height: 0, x: 0, y: 0 }, j6 = {
  ...XM,
  pointerX: 0,
  pointerY: 0,
  aspectRatio: 1
};
function U6(t, r, i) {
  const u = r.position.x + t.position.x, o = r.position.y + t.position.y, s = t.measured.width ?? 0, c = t.measured.height ?? 0, f = i[0] * s, g = i[1] * c;
  return [
    [u - f, o - g],
    [u + s - f, o + c - g]
  ];
}
function G6({ domNode: t, nodeId: r, getStoreItems: i, onChange: u, onEnd: o }) {
  const s = Zt(t);
  let c = {
    controlDirection: aA("bottom-right"),
    boundaries: {
      minWidth: 0,
      minHeight: 0,
      maxWidth: Number.MAX_VALUE,
      maxHeight: Number.MAX_VALUE
    },
    resizeDirection: void 0,
    keepAspectRatio: !1
  };
  function f({ controlPosition: h, boundaries: v, keepAspectRatio: p, resizeDirection: m, onResizeStart: b, onResize: _, onResizeEnd: A, shouldResize: w }) {
    let E = { ...XM }, M = { ...j6 };
    c = {
      boundaries: v,
      resizeDirection: m,
      keepAspectRatio: p,
      controlDirection: aA(h)
    };
    let S, T = null, O = [], C, R, H, B = !1;
    const X = WT().on("start", (Y) => {
      const { nodeLookup: F, transform: K, snapGrid: D, snapToGrid: G, nodeOrigin: N, paneDomNode: j } = i();
      if (S = F.get(r), !S)
        return;
      T = (j == null ? void 0 : j.getBoundingClientRect()) ?? null;
      const { xSnapped: Z, ySnapped: Q } = Mu(Y.sourceEvent, {
        transform: K,
        snapGrid: D,
        snapToGrid: G,
        containerBounds: T
      });
      E = {
        width: S.measured.width ?? 0,
        height: S.measured.height ?? 0,
        x: S.position.x ?? 0,
        y: S.position.y ?? 0
      }, M = {
        ...E,
        pointerX: Z,
        pointerY: Q,
        aspectRatio: E.width / E.height
      }, C = void 0, R = sa(S.extent) ? S.extent : void 0, S.parentId && (S.extent === "parent" || S.expandParent) && (C = F.get(S.parentId)), C && S.extent === "parent" && (R = [
        [0, 0],
        [C.measured.width, C.measured.height]
      ]), O = [], H = void 0;
      for (const [le, z] of F)
        if (z.parentId === r && (O.push({
          id: le,
          position: { ...z.position },
          extent: z.extent
        }), z.extent === "parent" || z.expandParent)) {
          const V = U6(z, S, z.origin ?? N);
          H ? H = [
            [Math.min(V[0][0], H[0][0]), Math.min(V[0][1], H[0][1])],
            [Math.max(V[1][0], H[1][0]), Math.max(V[1][1], H[1][1])]
          ] : H = V;
        }
      b == null || b(Y, { ...E });
    }).on("drag", (Y) => {
      const { transform: F, snapGrid: K, snapToGrid: D, nodeOrigin: G } = i(), N = Mu(Y.sourceEvent, {
        transform: F,
        snapGrid: K,
        snapToGrid: D,
        containerBounds: T
      }), j = [];
      if (!S)
        return;
      const { x: Z, y: Q, width: le, height: z } = E, V = {}, ie = S.origin ?? G, { width: L, height: I, x: P, y: ae } = B6(M, c.controlDirection, N, c.boundaries, c.keepAspectRatio, ie, R, H), W = L !== le, se = I !== z, de = P !== Z && W, ve = ae !== Q && se;
      if (!de && !ve && !W && !se)
        return;
      if ((de || ve || ie[0] === 1 || ie[1] === 1) && (V.x = de ? P : E.x, V.y = ve ? ae : E.y, E.x = V.x, E.y = V.y, O.length > 0)) {
        const ge = P - Z, Ae = ae - Q;
        for (const xe of O)
          xe.position = {
            x: xe.position.x - ge + ie[0] * (L - le),
            y: xe.position.y - Ae + ie[1] * (I - z)
          }, j.push(xe);
      }
      if ((W || se) && (V.width = W && (!c.resizeDirection || c.resizeDirection === "horizontal") ? L : E.width, V.height = se && (!c.resizeDirection || c.resizeDirection === "vertical") ? I : E.height, E.width = V.width, E.height = V.height), C && S.expandParent) {
        const ge = ie[0] * (V.width ?? 0);
        V.x && V.x < ge && (E.x = ge, M.x = M.x - (V.x - ge));
        const Ae = ie[1] * (V.height ?? 0);
        V.y && V.y < Ae && (E.y = Ae, M.y = M.y - (V.y - Ae));
      }
      const pe = L6({
        width: E.width,
        prevWidth: le,
        height: E.height,
        prevHeight: z,
        affectsX: c.controlDirection.affectsX,
        affectsY: c.controlDirection.affectsY
      }), he = { ...E, direction: pe };
      (w == null ? void 0 : w(Y, he)) !== !1 && (B = !0, _ == null || _(Y, he), u(V, j));
    }).on("end", (Y) => {
      B && (A == null || A(Y, { ...E }), o == null || o({ ...E }), B = !1);
    });
    s.call(X);
  }
  function g() {
    s.on(".drag", null);
  }
  return {
    update: f,
    destroy: g
  };
}
var fp = { exports: {} }, dp = {}, hp = { exports: {} }, gp = {};
/**
 * @license React
 * use-sync-external-store-shim.production.js
 *
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
var uA;
function V6() {
  if (uA) return gp;
  uA = 1;
  var t = Bu();
  function r(p, m) {
    return p === m && (p !== 0 || 1 / p === 1 / m) || p !== p && m !== m;
  }
  var i = typeof Object.is == "function" ? Object.is : r, u = t.useState, o = t.useEffect, s = t.useLayoutEffect, c = t.useDebugValue;
  function f(p, m) {
    var b = m(), _ = u({ inst: { value: b, getSnapshot: m } }), A = _[0].inst, w = _[1];
    return s(
      function() {
        A.value = b, A.getSnapshot = m, g(A) && w({ inst: A });
      },
      [p, b, m]
    ), o(
      function() {
        return g(A) && w({ inst: A }), p(function() {
          g(A) && w({ inst: A });
        });
      },
      [p]
    ), c(b), b;
  }
  function g(p) {
    var m = p.getSnapshot;
    p = p.value;
    try {
      var b = m();
      return !i(p, b);
    } catch {
      return !0;
    }
  }
  function h(p, m) {
    return m();
  }
  var v = typeof window > "u" || typeof window.document > "u" || typeof window.document.createElement > "u" ? h : f;
  return gp.useSyncExternalStore = t.useSyncExternalStore !== void 0 ? t.useSyncExternalStore : v, gp;
}
var lA;
function Y6() {
  return lA || (lA = 1, hp.exports = V6()), hp.exports;
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
var oA;
function k6() {
  if (oA) return dp;
  oA = 1;
  var t = Bu(), r = Y6();
  function i(h, v) {
    return h === v && (h !== 0 || 1 / h === 1 / v) || h !== h && v !== v;
  }
  var u = typeof Object.is == "function" ? Object.is : i, o = r.useSyncExternalStore, s = t.useRef, c = t.useEffect, f = t.useMemo, g = t.useDebugValue;
  return dp.useSyncExternalStoreWithSelector = function(h, v, p, m, b) {
    var _ = s(null);
    if (_.current === null) {
      var A = { hasValue: !1, value: null };
      _.current = A;
    } else A = _.current;
    _ = f(
      function() {
        function E(C) {
          if (!M) {
            if (M = !0, S = C, C = m(C), b !== void 0 && A.hasValue) {
              var R = A.value;
              if (b(R, C))
                return T = R;
            }
            return T = C;
          }
          if (R = T, u(S, C)) return R;
          var H = m(C);
          return b !== void 0 && b(R, H) ? (S = C, R) : (S = C, T = H);
        }
        var M = !1, S, T, O = p === void 0 ? null : p;
        return [
          function() {
            return E(v());
          },
          O === null ? void 0 : function() {
            return E(O());
          }
        ];
      },
      [v, p, m, b]
    );
    var w = o(h, _[0], _[1]);
    return c(
      function() {
        A.hasValue = !0, A.value = w;
      },
      [w]
    ), g(w), w;
  }, dp;
}
var sA;
function X6() {
  return sA || (sA = 1, fp.exports = k6()), fp.exports;
}
var I6 = X6();
const Q6 = /* @__PURE__ */ Np(I6), Z6 = {}, cA = (t) => {
  let r;
  const i = /* @__PURE__ */ new Set(), u = (v, p) => {
    const m = typeof v == "function" ? v(r) : v;
    if (!Object.is(m, r)) {
      const b = r;
      r = p ?? (typeof m != "object" || m === null) ? m : Object.assign({}, r, m), i.forEach((_) => _(r, b));
    }
  }, o = () => r, g = { setState: u, getState: o, getInitialState: () => h, subscribe: (v) => (i.add(v), () => i.delete(v)), destroy: () => {
    (Z6 ? "production" : void 0) !== "production" && console.warn(
      "[DEPRECATED] The `destroy` method will be unsupported in a future version. Instead use unsubscribe function returned by subscribe. Everything will be garbage-collected if store is garbage-collected."
    ), i.clear();
  } }, h = r = t(u, o, g);
  return g;
}, K6 = (t) => t ? cA(t) : cA, { useDebugValue: $6 } = YR, { useSyncExternalStoreWithSelector: F6 } = Q6, J6 = (t) => t;
function IM(t, r = J6, i) {
  const u = F6(
    t.subscribe,
    t.getState,
    t.getServerState || t.getInitialState,
    r,
    i
  );
  return $6(u), u;
}
const fA = (t, r) => {
  const i = K6(t), u = (o, s = r) => IM(i, o, s);
  return Object.assign(u, i), u;
}, P6 = (t, r) => t ? fA(t, r) : fA;
function Je(t, r) {
  if (Object.is(t, r))
    return !0;
  if (typeof t != "object" || t === null || typeof r != "object" || r === null)
    return !1;
  if (t instanceof Map && r instanceof Map) {
    if (t.size !== r.size) return !1;
    for (const [u, o] of t)
      if (!Object.is(o, r.get(u)))
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
UA();
const ys = re.createContext(null), W6 = ys.Provider, QM = vn.error001("react");
function ze(t, r) {
  const i = re.useContext(ys);
  if (i === null)
    throw new Error(QM);
  return IM(i, t, r);
}
function Qe() {
  const t = re.useContext(ys);
  if (t === null)
    throw new Error(QM);
  return re.useMemo(() => ({
    getState: t.getState,
    setState: t.setState,
    subscribe: t.subscribe
  }), [t]);
}
const dA = { display: "none" }, e8 = {
  position: "absolute",
  width: 1,
  height: 1,
  margin: -1,
  border: 0,
  padding: 0,
  overflow: "hidden",
  clip: "rect(0px, 0px, 0px, 0px)",
  clipPath: "inset(100%)"
}, ZM = "react-flow__node-desc", KM = "react-flow__edge-desc", t8 = "react-flow__aria-live", n8 = (t) => t.ariaLiveMessage, r8 = (t) => t.ariaLabelConfig;
function a8({ rfId: t }) {
  const r = ze(n8);
  return J.jsx("div", { id: `${t8}-${t}`, "aria-live": "assertive", "aria-atomic": "true", style: e8, children: r });
}
function i8({ rfId: t, disableKeyboardA11y: r }) {
  const i = ze(r8);
  return J.jsxs(J.Fragment, { children: [J.jsx("div", { id: `${ZM}-${t}`, style: dA, children: r ? i["node.a11yDescription.default"] : i["node.a11yDescription.keyboardDisabled"] }), J.jsx("div", { id: `${KM}-${t}`, style: dA, children: i["edge.a11yDescription.default"] }), !r && J.jsx(a8, { rfId: t })] });
}
const ps = re.forwardRef(({ position: t = "top-left", children: r, className: i, style: u, ...o }, s) => {
  const c = `${t}`.split("-");
  return J.jsx("div", { className: ct(["react-flow__panel", i, ...c]), style: u, ref: s, ...o, children: r });
});
ps.displayName = "Panel";
const hA = "https://reactflow.dev?utm_source=attribution";
function u8({ proOptions: t, position: r = "bottom-right" }) {
  return t != null && t.hideAttribution ? null : J.jsx(ps, { position: r, className: "react-flow__attribution", "data-message": `Please only hide this attribution when you are subscribed to React Flow Pro: ${hA}`, children: J.jsx("a", { href: hA, target: "_blank", rel: "noopener noreferrer", "aria-label": "React Flow attribution", children: "React Flow" }) });
}
const l8 = (t) => {
  const r = [], i = [];
  for (const [, u] of t.nodeLookup)
    u.selected && r.push(u.internals.userNode);
  for (const [, u] of t.edgeLookup)
    u.selected && i.push(u);
  return { selectedNodes: r, selectedEdges: i };
}, To = (t) => t.id;
function o8(t, r) {
  return Je(t.selectedNodes.map(To), r.selectedNodes.map(To)) && Je(t.selectedEdges.map(To), r.selectedEdges.map(To));
}
function s8({ onSelectionChange: t }) {
  const r = Qe(), { selectedNodes: i, selectedEdges: u } = ze(l8, o8);
  return re.useEffect(() => {
    const o = { nodes: i, edges: u };
    t == null || t(o), r.getState().onSelectionChangeHandlers.forEach((s) => s(o));
  }, [i, u, t]), null;
}
const c8 = (t) => !!t.onSelectionChangeHandlers;
function f8({ onSelectionChange: t }) {
  const r = ze(c8);
  return t || r ? J.jsx(s8, { onSelectionChange: t }) : null;
}
const $M = [0, 0], d8 = { x: 0, y: 0, zoom: 1 }, h8 = [
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
], gA = [...h8, "rfId"], g8 = (t) => ({
  setNodes: t.setNodes,
  setEdges: t.setEdges,
  setMinZoom: t.setMinZoom,
  setMaxZoom: t.setMaxZoom,
  setTranslateExtent: t.setTranslateExtent,
  setNodeExtent: t.setNodeExtent,
  reset: t.reset,
  setDefaultNodesAndEdges: t.setDefaultNodesAndEdges
}), vA = {
  /*
   * these are values that are also passed directly to other components
   * than the StoreUpdater. We can reduce the number of setStore calls
   * by setting the same values here as prev fields.
   */
  translateExtent: Ou,
  nodeOrigin: $M,
  minZoom: 0.5,
  maxZoom: 2,
  elementsSelectable: !0,
  noPanClassName: "nopan",
  rfId: "1"
};
function v8(t) {
  const { setNodes: r, setEdges: i, setMinZoom: u, setMaxZoom: o, setTranslateExtent: s, setNodeExtent: c, reset: f, setDefaultNodesAndEdges: g } = ze(g8, Je), h = Qe();
  re.useEffect(() => (g(t.defaultNodes, t.defaultEdges), () => {
    v.current = vA, f();
  }), []);
  const v = re.useRef(vA);
  return re.useEffect(
    () => {
      for (const p of gA) {
        const m = t[p], b = v.current[p];
        m !== b && (typeof t[p] > "u" || (p === "nodes" ? r(m) : p === "edges" ? i(m) : p === "minZoom" ? u(m) : p === "maxZoom" ? o(m) : p === "translateExtent" ? s(m) : p === "nodeExtent" ? c(m) : p === "ariaLabelConfig" ? h.setState({ ariaLabelConfig: W5(m) }) : p === "fitView" ? h.setState({ fitViewQueued: m }) : p === "fitViewOptions" ? h.setState({ fitViewOptions: m }) : h.setState({ [p]: m })));
      }
      v.current = t;
    },
    // Only re-run the effect if one of the fields we track changes
    gA.map((p) => t[p])
  ), null;
}
function yA() {
  return typeof window > "u" || !window.matchMedia ? null : window.matchMedia("(prefers-color-scheme: dark)");
}
function y8(t) {
  var u;
  const [r, i] = re.useState(t === "system" ? null : t);
  return re.useEffect(() => {
    if (t !== "system") {
      i(t);
      return;
    }
    const o = yA(), s = () => i(o != null && o.matches ? "dark" : "light");
    return s(), o == null || o.addEventListener("change", s), () => {
      o == null || o.removeEventListener("change", s);
    };
  }, [t]), r !== null ? r : (u = yA()) != null && u.matches ? "dark" : "light";
}
const pA = typeof document < "u" ? document : null;
function Lu(t = null, r = { target: pA, actInsideInputWithModifier: !0 }) {
  const [i, u] = re.useState(!1), o = re.useRef(!1), s = re.useRef(/* @__PURE__ */ new Set([])), [c, f] = re.useMemo(() => {
    if (t !== null) {
      const h = (Array.isArray(t) ? t : [t]).filter((p) => typeof p == "string").map((p) => p.replace("+", `
`).replace(`

`, `
+`).split(`
`)), v = h.reduce((p, m) => p.concat(...m), []);
      return [h, v];
    }
    return [[], []];
  }, [t]);
  return re.useEffect(() => {
    const g = (r == null ? void 0 : r.target) ?? pA, h = (r == null ? void 0 : r.actInsideInputWithModifier) ?? !0;
    if (t !== null) {
      const v = (b) => {
        var w, E;
        if (o.current = b.ctrlKey || b.metaKey || b.shiftKey || b.altKey, (!o.current || o.current && !h) && qM(b))
          return !1;
        const A = bA(b.code, f);
        if (s.current.add(b[A]), mA(c, s.current, !1)) {
          const M = ((E = (w = b.composedPath) == null ? void 0 : w.call(b)) == null ? void 0 : E[0]) || b.target, S = (M == null ? void 0 : M.nodeName) === "BUTTON" || (M == null ? void 0 : M.nodeName) === "A";
          r.preventDefault !== !1 && (o.current || !S) && b.preventDefault(), u(!0);
        }
      }, p = (b) => {
        const _ = bA(b.code, f);
        mA(c, s.current, !0) ? (u(!1), s.current.clear()) : s.current.delete(b[_]), b.key === "Meta" && s.current.clear(), o.current = !1;
      }, m = () => {
        s.current.clear(), u(!1);
      };
      return g == null || g.addEventListener("keydown", v), g == null || g.addEventListener("keyup", p), window.addEventListener("blur", m), window.addEventListener("contextmenu", m), () => {
        g == null || g.removeEventListener("keydown", v), g == null || g.removeEventListener("keyup", p), window.removeEventListener("blur", m), window.removeEventListener("contextmenu", m);
      };
    }
  }, [t, u]), i;
}
function mA(t, r, i) {
  return t.filter((u) => i || u.length === r.size).some((u) => u.every((o) => r.has(o)));
}
function bA(t, r) {
  return r.includes(t) ? "code" : "key";
}
const p8 = () => {
  const t = Qe();
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
      const { transform: [u, o, s], panZoom: c } = t.getState();
      return c ? (await c.setViewport({
        x: r.x ?? u,
        y: r.y ?? o,
        zoom: r.zoom ?? s
      }, i), !0) : !1;
    },
    getViewport: () => {
      const [r, i, u] = t.getState().transform;
      return { x: r, y: i, zoom: u };
    },
    setCenter: async (r, i, u) => t.getState().setCenter(r, i, u),
    fitBounds: async (r, i) => {
      const { width: u, height: o, minZoom: s, maxZoom: c, panZoom: f } = t.getState(), g = um(r, u, o, s, c, (i == null ? void 0 : i.padding) ?? 0.1);
      return f ? (await f.setViewport(g, {
        duration: i == null ? void 0 : i.duration,
        ease: i == null ? void 0 : i.ease,
        interpolate: i == null ? void 0 : i.interpolate
      }), !0) : !1;
    },
    screenToFlowPosition: (r, i = {}) => {
      const { transform: u, snapGrid: o, snapToGrid: s, domNode: c } = t.getState();
      if (!c)
        return r;
      const { x: f, y: g } = c.getBoundingClientRect(), h = {
        x: r.x - f,
        y: r.y - g
      }, v = i.snapGrid ?? o, p = i.snapToGrid ?? s;
      return Ku(h, u, p, v);
    },
    flowToScreenPosition: (r) => {
      const { transform: i, domNode: u } = t.getState();
      if (!u)
        return r;
      const { x: o, y: s } = u.getBoundingClientRect(), c = di(r, i);
      return {
        x: c.x + o,
        y: c.y + s
      };
    }
  }), []);
};
function FM(t, r) {
  const i = [], u = /* @__PURE__ */ new Map(), o = [];
  for (const s of t)
    if (s.type === "add") {
      o.push(s);
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
      m8(g, f);
    i.push(f);
  }
  return o.length && o.forEach((s) => {
    s.index !== void 0 ? i.splice(s.index, 0, { ...s.item }) : i.push({ ...s.item });
  }), i;
}
function m8(t, r) {
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
function b8(t, r) {
  return FM(t, r);
}
function _8(t, r) {
  return FM(t, r);
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
  for (const [o, s] of t) {
    const c = r.has(o);
    !(s.selected === void 0 && !c) && s.selected !== c && (i && (s.selected = c), u.push(na(s.id, c)));
  }
  return u;
}
function _A({ items: t = [], lookup: r }) {
  var o;
  const i = [], u = new Map(t.map((s) => [s.id, s]));
  for (const [s, c] of t.entries()) {
    const f = r.get(c.id), g = ((o = f == null ? void 0 : f.internals) == null ? void 0 : o.userNode) ?? f;
    g !== void 0 && g !== c && i.push({ id: c.id, item: c, type: "replace" }), g === void 0 && i.push({ item: c, type: "add", index: s });
  }
  for (const [s] of r)
    u.get(s) === void 0 && i.push({ id: s, type: "remove" });
  return i;
}
function xA(t) {
  return {
    id: t.id,
    type: "remove"
  };
}
const x8 = wM();
function S8(t, r, i = {}) {
  return i6(t, r, {
    ...i,
    onError: i.onError ?? x8
  });
}
const SA = (t) => X5(t), E8 = (t) => bM(t);
function JM(t) {
  return re.forwardRef(t);
}
const PM = typeof window < "u" ? re.useLayoutEffect : re.useEffect;
function EA(t) {
  const [r, i] = re.useState(BigInt(0)), [u] = re.useState(() => w8(() => i((o) => o + BigInt(1))));
  return PM(() => {
    const o = u.get();
    o.length && (t(o), u.reset());
  }, [r]), u;
}
function w8(t) {
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
const WM = re.createContext(null);
function A8({ children: t }) {
  const r = Qe(), i = re.useCallback((f) => {
    const { nodes: g = [], setNodes: h, hasDefaultNodes: v, onNodesChange: p, nodeLookup: m, fitViewQueued: b, onNodesChangeMiddlewareMap: _ } = r.getState();
    let A = g;
    for (const E of f)
      A = typeof E == "function" ? E(A) : E;
    let w = _A({
      items: A,
      lookup: m
    });
    for (const E of _.values())
      w = E(w);
    v && h(A), w.length > 0 ? p == null || p(w) : b && window.requestAnimationFrame(() => {
      const { fitViewQueued: E, nodes: M, setNodes: S } = r.getState();
      E && S(M);
    });
  }, []), u = EA(i), o = re.useCallback((f) => {
    const { edges: g = [], setEdges: h, hasDefaultEdges: v, onEdgesChange: p, edgeLookup: m } = r.getState();
    let b = g;
    for (const _ of f)
      b = typeof _ == "function" ? _(b) : _;
    v ? h(b) : p && p(_A({
      items: b,
      lookup: m
    }));
  }, []), s = EA(o), c = re.useMemo(() => ({ nodeQueue: u, edgeQueue: s }), []);
  return J.jsx(WM.Provider, { value: c, children: t });
}
function T8() {
  const t = re.useContext(WM);
  if (!t)
    throw new Error("useBatchContext must be used within a BatchProvider");
  return t;
}
const M8 = (t) => !!t.panZoom;
function hm() {
  const t = p8(), r = Qe(), i = T8(), u = ze(M8), o = re.useMemo(() => {
    const s = (p) => r.getState().nodeLookup.get(p), c = (p) => {
      i.nodeQueue.push(p);
    }, f = (p) => {
      i.edgeQueue.push(p);
    }, g = (p) => {
      var E, M;
      const { nodeLookup: m, nodeOrigin: b } = r.getState(), _ = SA(p) ? p : m.get(p.id), A = _.parentId ? TM(_.position, _.measured, _.parentId, m, b) : _.position, w = {
        ..._,
        position: A,
        width: ((E = _.measured) == null ? void 0 : E.width) ?? _.width,
        height: ((M = _.measured) == null ? void 0 : M.height) ?? _.height
      };
      return Du(w);
    }, h = (p, m, b = { replace: !1 }) => {
      c((_) => _.map((A) => {
        if (A.id === p) {
          const w = typeof m == "function" ? m(A) : m;
          return b.replace && SA(w) ? w : { ...A, ...w };
        }
        return A;
      }));
    }, v = (p, m, b = { replace: !1 }) => {
      f((_) => _.map((A) => {
        if (A.id === p) {
          const w = typeof m == "function" ? m(A) : m;
          return b.replace && E8(w) ? w : { ...A, ...w };
        }
        return A;
      }));
    };
    return {
      getNodes: () => r.getState().nodes.map((p) => ({ ...p })),
      getNode: (p) => {
        var m;
        return (m = s(p)) == null ? void 0 : m.internals.userNode;
      },
      getInternalNode: s,
      getEdges: () => {
        const { edges: p = [] } = r.getState();
        return p.map((m) => ({ ...m }));
      },
      getEdge: (p) => r.getState().edgeLookup.get(p),
      setNodes: c,
      setEdges: f,
      addNodes: (p) => {
        const m = Array.isArray(p) ? p : [p];
        i.nodeQueue.push((b) => [...b, ...m]);
      },
      addEdges: (p) => {
        const m = Array.isArray(p) ? p : [p];
        i.edgeQueue.push((b) => [...b, ...m]);
      },
      toObject: () => {
        const { nodes: p = [], edges: m = [], transform: b } = r.getState(), [_, A, w] = b;
        return {
          nodes: p.map((E) => ({ ...E })),
          edges: m.map((E) => ({ ...E })),
          viewport: {
            x: _,
            y: A,
            zoom: w
          }
        };
      },
      deleteElements: async ({ nodes: p = [], edges: m = [] }) => {
        const { nodes: b, edges: _, onNodesDelete: A, onEdgesDelete: w, triggerNodeChanges: E, triggerEdgeChanges: M, onDelete: S, onBeforeDelete: T } = r.getState(), { nodes: O, edges: C } = await $5({
          nodesToRemove: p,
          edgesToRemove: m,
          nodes: b,
          edges: _,
          onBeforeDelete: T
        }), R = C.length > 0, H = O.length > 0;
        if (R) {
          const B = C.map(xA);
          w == null || w(C), M(B);
        }
        if (H) {
          const B = O.map(xA);
          A == null || A(O), E(B);
        }
        return (H || R) && (S == null || S({ nodes: O, edges: C })), { deletedNodes: O, deletedEdges: C };
      },
      /**
       * Partial is defined as "the 2 nodes/areas are intersecting partially".
       * If a is contained in b or b is contained in a, they are both
       * considered fully intersecting.
       */
      getIntersectingNodes: (p, m = !0, b) => {
        const _ = Z2(p), A = _ ? p : g(p), w = b !== void 0;
        return A ? (b || r.getState().nodes).filter((E) => {
          const M = r.getState().nodeLookup.get(E.id);
          if (M && !_ && (E.id === p.id || !M.internals.positionAbsolute))
            return !1;
          const S = Du(w ? E : M), T = ko(S, A);
          return m && T > 0 || T >= S.width * S.height || T >= A.width * A.height;
        }) : [];
      },
      isNodeIntersecting: (p, m, b = !0) => {
        const A = Z2(p) ? p : g(p);
        if (!A)
          return !1;
        const w = ko(A, m);
        return b && w > 0 || w >= m.width * m.height || w >= A.width * A.height;
      },
      updateNode: h,
      updateNodeData: (p, m, b = { replace: !1 }) => {
        h(p, (_) => {
          const A = typeof m == "function" ? m(_) : m;
          return b.replace ? { ..._, data: A } : { ..._, data: { ..._.data, ...A } };
        }, b);
      },
      updateEdge: v,
      updateEdgeData: (p, m, b = { replace: !1 }) => {
        v(p, (_) => {
          const A = typeof m == "function" ? m(_) : m;
          return b.replace ? { ..._, data: A } : { ..._, data: { ..._.data, ...A } };
        }, b);
      },
      getNodesBounds: (p) => {
        const { nodeLookup: m, nodeOrigin: b } = r.getState();
        return I5(p, { nodeLookup: m, nodeOrigin: b });
      },
      getHandleConnections: ({ type: p, id: m, nodeId: b }) => {
        var _;
        return Array.from(((_ = r.getState().connectionLookup.get(`${b}-${p}${m ? `-${m}` : ""}`)) == null ? void 0 : _.values()) ?? []);
      },
      getNodeConnections: ({ type: p, handleId: m, nodeId: b }) => {
        var _;
        return Array.from(((_ = r.getState().connectionLookup.get(`${b}${p ? m ? `-${p}-${m}` : `-${p}` : ""}`)) == null ? void 0 : _.values()) ?? []);
      },
      fitView: async (p) => {
        const m = r.getState().fitViewResolver ?? P5();
        return r.setState({ fitViewQueued: !0, fitViewOptions: p, fitViewResolver: m }), i.nodeQueue.push((b) => [...b]), m.promise;
      }
    };
  }, []);
  return re.useMemo(() => ({
    ...o,
    ...t,
    viewportInitialized: u
  }), [u]);
}
const wA = (t) => t.selected, q8 = typeof window < "u" ? window : void 0;
function C8({ deleteKeyCode: t, multiSelectionKeyCode: r }) {
  const i = Qe(), { deleteElements: u } = hm(), o = Lu(t, { actInsideInputWithModifier: !1 }), s = Lu(r, { target: q8 });
  re.useEffect(() => {
    if (o) {
      const { edges: c, nodes: f } = i.getState();
      u({ nodes: f.filter(wA), edges: c.filter(wA) }), i.setState({ nodesSelectionActive: !1 });
    }
  }, [o]), re.useEffect(() => {
    i.setState({ multiSelectionActive: s });
  }, [s]);
}
function R8(t) {
  const r = Qe();
  re.useEffect(() => {
    const i = () => {
      var o, s, c, f;
      if (!t.current || !(((s = (o = t.current).checkVisibility) == null ? void 0 : s.call(o)) ?? !0))
        return !1;
      const u = lm(t.current);
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
const ms = {
  position: "absolute",
  width: "100%",
  height: "100%",
  top: 0,
  left: 0
}, N8 = (t) => ({
  userSelectionActive: t.userSelectionActive,
  lib: t.lib,
  connectionInProgress: t.connection.inProgress
});
function O8({ onPaneContextMenu: t, zoomOnScroll: r = !0, zoomOnPinch: i = !0, panOnScroll: u = !1, panOnScrollSpeed: o = 0.5, panOnScrollMode: s = ia.Free, zoomOnDoubleClick: c = !0, panOnDrag: f = !0, defaultViewport: g, translateExtent: h, minZoom: v, maxZoom: p, zoomActivationKeyCode: m, preventScrolling: b = !0, children: _, noWheelClassName: A, noPanClassName: w, onViewportChange: E, isControlledViewport: M, paneClickDistance: S, selectionOnDrag: T }) {
  const O = Qe(), C = re.useRef(null), { userSelectionActive: R, lib: H, connectionInProgress: B } = ze(N8, Je), X = Lu(m), Y = re.useRef();
  R8(C);
  const F = re.useCallback((K) => {
    E == null || E({ x: K[0], y: K[1], zoom: K[2] }), M || O.setState({ transform: K });
  }, [E, M]);
  return re.useEffect(() => {
    if (C.current) {
      Y.current = H6({
        domNode: C.current,
        minZoom: v,
        maxZoom: p,
        translateExtent: h,
        viewport: g,
        onDraggingChange: (N) => O.setState((j) => j.paneDragging === N ? j : { paneDragging: N }),
        onPanZoomStart: (N, j) => {
          const { onViewportChangeStart: Z, onMoveStart: Q } = O.getState();
          Q == null || Q(N, j), Z == null || Z(j);
        },
        onPanZoom: (N, j) => {
          const { onViewportChange: Z, onMove: Q } = O.getState();
          Q == null || Q(N, j), Z == null || Z(j);
        },
        onPanZoomEnd: (N, j) => {
          const { onViewportChangeEnd: Z, onMoveEnd: Q } = O.getState();
          Q == null || Q(N, j), Z == null || Z(j);
        }
      });
      const { x: K, y: D, zoom: G } = Y.current.getViewport();
      return O.setState({
        panZoom: Y.current,
        transform: [K, D, G],
        domNode: C.current.closest(".react-flow")
      }), () => {
        var N;
        (N = Y.current) == null || N.destroy();
      };
    }
  }, []), re.useEffect(() => {
    var K;
    (K = Y.current) == null || K.update({
      onPaneContextMenu: t,
      zoomOnScroll: r,
      zoomOnPinch: i,
      panOnScroll: u,
      panOnScrollSpeed: o,
      panOnScrollMode: s,
      zoomOnDoubleClick: c,
      panOnDrag: f,
      zoomActivationKeyPressed: X,
      preventScrolling: b,
      noPanClassName: w,
      userSelectionActive: R,
      noWheelClassName: A,
      lib: H,
      onTransformChange: F,
      connectionInProgress: B,
      selectionOnDrag: T,
      paneClickDistance: S
    });
  }, [
    t,
    r,
    i,
    u,
    o,
    s,
    c,
    f,
    X,
    b,
    w,
    R,
    A,
    H,
    F,
    B,
    T,
    S
  ]), J.jsx("div", { className: "react-flow__renderer", ref: C, style: ms, children: _ });
}
const z8 = (t) => ({
  userSelectionActive: t.userSelectionActive,
  userSelectionRect: t.userSelectionRect
});
function D8() {
  const { userSelectionActive: t, userSelectionRect: r } = ze(z8, Je);
  return t && r ? J.jsx("div", { className: "react-flow__selection react-flow__container", style: {
    width: r.width,
    height: r.height,
    transform: `translate(${r.x}px, ${r.y}px)`
  } }) : null;
}
const vp = (t, r) => (i) => {
  i.target === r.current && (t == null || t(i));
}, H8 = (t) => ({
  userSelectionActive: t.userSelectionActive,
  elementsSelectable: t.elementsSelectable,
  dragging: t.paneDragging,
  panBy: t.panBy,
  autoPanSpeed: t.autoPanSpeed
});
function L8({ isSelecting: t, selectionKeyPressed: r, selectionMode: i = zu.Full, panOnDrag: u, autoPanOnSelection: o, paneClickDistance: s, selectionOnDrag: c, onSelectionStart: f, onSelectionEnd: g, onPaneClick: h, onPaneContextMenu: v, onPaneScroll: p, onPaneMouseEnter: m, onPaneMouseMove: b, onPaneMouseLeave: _, children: A }) {
  const w = re.useRef(0), E = Qe(), { userSelectionActive: M, elementsSelectable: S, dragging: T, panBy: O, autoPanSpeed: C } = ze(H8, Je), R = S && (t || M), H = re.useRef(null), B = re.useRef(), X = re.useRef(/* @__PURE__ */ new Set()), Y = re.useRef(/* @__PURE__ */ new Set()), F = re.useRef(!1), K = re.useRef(!1), D = re.useRef({ x: 0, y: 0 }), G = re.useRef(!1), N = (W) => {
    if (K.current || F.current || E.getState().connection.inProgress) {
      K.current = !1, F.current = !1;
      return;
    }
    h == null || h(W), E.getState().resetSelectedElements(), E.setState({ nodesSelectionActive: !1 });
  }, j = (W) => {
    if (Array.isArray(u) && (u != null && u.includes(2))) {
      W.preventDefault();
      return;
    }
    v == null || v(W);
  }, Z = p ? (W) => p(W) : void 0, Q = (W) => {
    K.current && (W.stopPropagation(), K.current = !1);
  }, le = (W) => {
    var xe, Pe;
    const { domNode: se, transform: de } = E.getState();
    if (B.current = se == null ? void 0 : se.getBoundingClientRect(), !B.current)
      return;
    const ve = W.target === H.current;
    if (!ve && !!W.target.closest(".nokey") || !t || !(c && ve || r) || W.button !== 0 || !W.isPrimary)
      return;
    (Pe = (xe = W.target) == null ? void 0 : xe.setPointerCapture) == null || Pe.call(xe, W.pointerId), K.current = !1;
    const { x: me, y: ge } = hn(W.nativeEvent, B.current), Ae = Ku({ x: me, y: ge }, de);
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
  function z(W, se) {
    const { userSelectionRect: de } = E.getState();
    if (!de)
      return;
    const { transform: ve, nodeLookup: pe, edgeLookup: he, connectionLookup: me, triggerNodeChanges: ge, triggerEdgeChanges: Ae, defaultEdgeOptions: xe } = E.getState(), Pe = { x: de.startX, y: de.startY }, { x: tt, y: xt } = di(Pe, ve), gt = {
      startX: Pe.x,
      startY: Pe.y,
      x: W < tt ? W : tt,
      y: se < xt ? se : xt,
      width: Math.abs(W - tt),
      height: Math.abs(se - xt)
    }, St = X.current, Ze = Y.current;
    X.current = new Set(am(pe, gt, ve, i === zu.Partial, !0).map((vt) => vt.id)), Y.current = /* @__PURE__ */ new Set();
    const ke = (xe == null ? void 0 : xe.selectable) ?? !0;
    for (const vt of X.current) {
      const Ct = me.get(vt);
      if (Ct)
        for (const { edgeId: Tt } of Ct.values()) {
          const ft = he.get(Tt);
          ft && (ft.selectable ?? ke) && Y.current.add(Tt);
        }
    }
    if (!K2(St, X.current)) {
      const vt = ii(pe, X.current, !0);
      ge(vt);
    }
    if (!K2(Ze, Y.current)) {
      const vt = ii(he, Y.current);
      Ae(vt);
    }
    E.setState({
      userSelectionRect: gt,
      userSelectionActive: !0,
      nodesSelectionActive: !1
    });
  }
  function V() {
    if (!o || !B.current)
      return;
    const [W, se] = im(D.current, B.current, C);
    O({ x: W, y: se }).then((de) => {
      if (!K.current || !de) {
        w.current = requestAnimationFrame(V);
        return;
      }
      const { x: ve, y: pe } = D.current;
      z(ve, pe), w.current = requestAnimationFrame(V);
    });
  }
  const ie = () => {
    cancelAnimationFrame(w.current), w.current = 0, G.current = !1;
  };
  re.useEffect(() => () => ie(), []);
  const L = (W) => {
    const { userSelectionRect: se, transform: de, resetSelectedElements: ve } = E.getState();
    if (!B.current || !se)
      return;
    const { x: pe, y: he } = hn(W.nativeEvent, B.current);
    D.current = { x: pe, y: he };
    const me = di({ x: se.startX, y: se.startY }, de);
    if (!K.current) {
      const ge = r ? 0 : s;
      if (Math.hypot(pe - me.x, he - me.y) <= ge)
        return;
      ve(), f == null || f(W);
    }
    K.current = !0, G.current || (V(), G.current = !0), z(pe, he);
  }, I = (W) => {
    var se, de;
    if (!R) {
      W.target === H.current && E.getState().connection.inProgress && (F.current = !0);
      return;
    }
    W.button === 0 && ((de = (se = W.target) == null ? void 0 : se.releasePointerCapture) == null || de.call(se, W.pointerId), !M && W.target === H.current && E.getState().userSelectionRect && (N == null || N(W)), E.setState({
      userSelectionActive: !1,
      userSelectionRect: null
    }), K.current && (g == null || g(W), E.setState({
      nodesSelectionActive: X.current.size > 0
    })), ie());
  }, P = (W) => {
    var se, de;
    (de = (se = W.target) == null ? void 0 : se.releasePointerCapture) == null || de.call(se, W.pointerId), ie();
  }, ae = u === !0 || Array.isArray(u) && u.includes(0);
  return J.jsxs("div", { className: ct(["react-flow__pane", { draggable: ae, dragging: T, selection: t }]), onClick: R ? void 0 : vp(N, H), onContextMenu: vp(j, H), onWheel: vp(Z, H), onPointerEnter: R ? void 0 : m, onPointerMove: R ? L : b, onPointerUp: I, onPointerCancel: R ? P : void 0, onPointerDownCapture: R ? le : void 0, onClickCapture: R ? Q : void 0, onPointerLeave: _, ref: H, style: ms, children: [A, J.jsx(D8, {})] });
}
function Rp({ id: t, store: r, unselect: i = !1, nodeRef: u }) {
  const { addSelectedNodes: o, unselectNodesAndEdges: s, multiSelectionActive: c, nodeLookup: f, onError: g } = r.getState(), h = f.get(t);
  if (!h) {
    g == null || g("012", vn.error012(t));
    return;
  }
  r.setState({ nodesSelectionActive: !1 }), h.selected ? (i || h.selected && c) && (s({ nodes: [h], edges: [] }), requestAnimationFrame(() => {
    var v;
    return (v = u == null ? void 0 : u.current) == null ? void 0 : v.blur();
  })) : o([t]);
}
function eq({ nodeRef: t, disabled: r = !1, noDragClassName: i, handleSelector: u, nodeId: o, isSelectable: s, nodeClickDistance: c }) {
  const f = Qe(), [g, h] = re.useState(!1), v = re.useRef();
  return re.useEffect(() => {
    if (!r)
      return v.current = x6({
        getStoreItems: () => f.getState(),
        onNodeMouseDown: (p) => {
          Rp({
            id: p,
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
        var p;
        (p = v.current) == null || p.destroy(), v.current = void 0;
      };
  }, [r, f, t]), re.useEffect(() => {
    r || !t.current || !v.current || v.current.update({
      noDragClassName: i,
      handleSelector: u,
      domNode: t.current,
      isSelectable: s,
      nodeId: o,
      nodeClickDistance: c
    });
  }, [i, u, r, s, t, o, c]), g;
}
const B8 = (t) => (r) => r.selected && (r.draggable || t && typeof r.draggable > "u");
function tq() {
  const t = Qe();
  return re.useCallback((i) => {
    const { nodeExtent: u, snapToGrid: o, snapGrid: s, nodesDraggable: c, onError: f, updateNodePositions: g, nodeLookup: h, nodeOrigin: v } = t.getState(), p = /* @__PURE__ */ new Map(), m = B8(c), b = o ? s[0] : 5, _ = o ? s[1] : 5, A = i.direction.x * b * i.factor, w = i.direction.y * _ * i.factor;
    for (const [, E] of h) {
      if (!m(E))
        continue;
      let M = {
        x: E.internals.positionAbsolute.x + A,
        y: E.internals.positionAbsolute.y + w
      };
      o && (M = Zu(M, s));
      const { position: S, positionAbsolute: T } = _M({
        nodeId: E.id,
        nextPosition: M,
        nodeLookup: h,
        nodeExtent: u,
        nodeOrigin: v,
        onError: f
      });
      E.position = S, E.internals.positionAbsolute = T, p.set(E.id, E);
    }
    g(p);
  }, []);
}
const gm = re.createContext(null), j8 = gm.Provider;
gm.Consumer;
const nq = () => re.useContext(gm), U8 = (t) => ({
  connectOnClick: t.connectOnClick,
  noPanClassName: t.noPanClassName,
  rfId: t.rfId
}), rq = re.createContext(null);
function G8({ children: t }) {
  const r = ze(U8, Je);
  return J.jsx(rq.Provider, { value: r, children: t });
}
function V8() {
  const t = re.useContext(rq);
  if (!t)
    throw new Error("useHandleConfig must be used within a HandleConfigProvider");
  return t;
}
const Y8 = {
  connectingFrom: !1,
  connectingTo: !1,
  clickConnecting: !1,
  isPossibleEndHandle: !0,
  connectionInProcess: !1,
  clickConnectionInProcess: !1,
  valid: !1
}, k8 = (t, r, i) => (u) => {
  const { connectionClickStartHandle: o, connectionMode: s, connection: c } = u, { fromHandle: f, toHandle: g, isValid: h } = c;
  if (!f && !o)
    return Y8;
  const v = (g == null ? void 0 : g.nodeId) === t && (g == null ? void 0 : g.id) === r && (g == null ? void 0 : g.type) === i;
  return {
    connectingFrom: (f == null ? void 0 : f.nodeId) === t && (f == null ? void 0 : f.id) === r && (f == null ? void 0 : f.type) === i,
    connectingTo: v,
    clickConnecting: (o == null ? void 0 : o.nodeId) === t && (o == null ? void 0 : o.id) === r && (o == null ? void 0 : o.type) === i,
    isPossibleEndHandle: s === ci.Strict ? (f == null ? void 0 : f.type) !== i : t !== (f == null ? void 0 : f.nodeId) || r !== (f == null ? void 0 : f.id),
    connectionInProcess: !!f,
    clickConnectionInProcess: !!o,
    valid: v && h
  };
};
function X8({ type: t = "source", position: r = _e.Top, isValidConnection: i, isConnectable: u = !0, isConnectableStart: o = !0, isConnectableEnd: s = !0, id: c, onConnect: f, children: g, className: h, onMouseDown: v, onTouchStart: p, ...m }, b) {
  var G, N;
  const _ = c || null, A = t === "target", w = Qe(), E = nq(), { connectOnClick: M, noPanClassName: S, rfId: T } = V8(), { connectingFrom: O, connectingTo: C, clickConnecting: R, isPossibleEndHandle: H, connectionInProcess: B, clickConnectionInProcess: X, valid: Y } = ze(k8(E, _, t), Je);
  E || (N = (G = w.getState()).onError) == null || N.call(G, "010", vn.error010());
  const F = (j) => {
    const { defaultEdgeOptions: Z, onConnect: Q, hasDefaultEdges: le } = w.getState(), z = {
      ...Z,
      ...j
    };
    if (le) {
      const { edges: V, setEdges: ie, onError: L } = w.getState();
      ie(S8(z, V, { onError: L }));
    }
    Q == null || Q(z), f == null || f(z);
  }, K = (j) => {
    if (!E)
      return;
    const Z = CM(j.nativeEvent);
    if (o && (Z && j.button === 0 || !Z)) {
      const Q = w.getState();
      Cp.onPointerDown(j.nativeEvent, {
        handleDomNode: j.currentTarget,
        autoPanOnConnect: Q.autoPanOnConnect,
        connectionMode: Q.connectionMode,
        connectionRadius: Q.connectionRadius,
        domNode: Q.domNode,
        nodeLookup: Q.nodeLookup,
        lib: Q.lib,
        isTarget: A,
        handleId: _,
        nodeId: E,
        flowId: Q.rfId,
        panBy: Q.panBy,
        cancelConnection: Q.cancelConnection,
        onConnectStart: Q.onConnectStart,
        onConnectEnd: (...le) => {
          var z, V;
          return (V = (z = w.getState()).onConnectEnd) == null ? void 0 : V.call(z, ...le);
        },
        updateConnection: Q.updateConnection,
        onConnect: F,
        isValidConnection: i || ((...le) => {
          var z, V;
          return ((V = (z = w.getState()).isValidConnection) == null ? void 0 : V.call(z, ...le)) ?? !0;
        }),
        getTransform: () => w.getState().transform,
        getFromHandle: () => w.getState().connection.fromHandle,
        autoPanSpeed: Q.autoPanSpeed,
        dragThreshold: Q.connectionDragThreshold
      });
    }
    Z ? v == null || v(j) : p == null || p(j);
  }, D = (j) => {
    const { onClickConnectStart: Z, onClickConnectEnd: Q, connectionClickStartHandle: le, connectionMode: z, isValidConnection: V, lib: ie, rfId: L, nodeLookup: I, connection: P } = w.getState();
    if (!E || !le && !o)
      return;
    if (!le) {
      Z == null || Z(j.nativeEvent, { nodeId: E, handleId: _, handleType: t }), w.setState({ connectionClickStartHandle: { nodeId: E, type: t, id: _ } });
      return;
    }
    const ae = MM(j.target), W = i || V, { connection: se, isValid: de } = Cp.isValid(j.nativeEvent, {
      handle: {
        nodeId: E,
        id: _,
        type: t
      },
      connectionMode: z,
      fromNodeId: le.nodeId,
      fromHandleId: le.id || null,
      fromType: le.type,
      isValidConnection: W,
      flowId: L,
      doc: ae,
      lib: ie,
      nodeLookup: I
    });
    de && se && F(se);
    const ve = structuredClone(P);
    delete ve.inProgress, ve.toPosition = ve.toHandle ? ve.toHandle.position : null, Q == null || Q(j, ve), w.setState({ connectionClickStartHandle: null });
  };
  return J.jsx("div", { "data-handleid": _, "data-nodeid": E, "data-handlepos": r, "data-id": `${T}-${E}-${_}-${t}`, className: ct([
    "react-flow__handle",
    `react-flow__handle-${r}`,
    "nodrag",
    S,
    h,
    {
      source: !A,
      target: A,
      connectable: u,
      connectablestart: o,
      connectableend: s,
      clickconnecting: R,
      connectingfrom: O,
      connectingto: C,
      valid: Y,
      /*
       * shows where you can start a connection from
       * and where you can end it while connecting
       */
      connectionindicator: u && (!B || H) && (B || X ? s : o)
    }
  ]), onMouseDown: K, onTouchStart: K, onClick: M ? D : void 0, ref: b, ...m, children: g });
}
const gi = re.memo(JM(X8));
function I8({ data: t, isConnectable: r, sourcePosition: i = _e.Bottom }) {
  return J.jsxs(J.Fragment, { children: [t == null ? void 0 : t.label, J.jsx(gi, { type: "source", position: i, isConnectable: r })] });
}
function Q8({ data: t, isConnectable: r, targetPosition: i = _e.Top, sourcePosition: u = _e.Bottom }) {
  return J.jsxs(J.Fragment, { children: [J.jsx(gi, { type: "target", position: i, isConnectable: r }), t == null ? void 0 : t.label, J.jsx(gi, { type: "source", position: u, isConnectable: r })] });
}
function Z8() {
  return null;
}
function K8({ data: t, isConnectable: r, targetPosition: i = _e.Top }) {
  return J.jsxs(J.Fragment, { children: [J.jsx(gi, { type: "target", position: i, isConnectable: r }), t == null ? void 0 : t.label] });
}
const Xo = {
  ArrowUp: { x: 0, y: -1 },
  ArrowDown: { x: 0, y: 1 },
  ArrowLeft: { x: -1, y: 0 },
  ArrowRight: { x: 1, y: 0 }
}, AA = {
  input: I8,
  default: Q8,
  output: K8,
  group: Z8
};
function $8(t) {
  var r, i, u, o;
  return t.internals.handleBounds === void 0 ? {
    width: t.width ?? t.initialWidth ?? ((r = t.style) == null ? void 0 : r.width),
    height: t.height ?? t.initialHeight ?? ((i = t.style) == null ? void 0 : i.height)
  } : {
    width: t.width ?? ((u = t.style) == null ? void 0 : u.width),
    height: t.height ?? ((o = t.style) == null ? void 0 : o.height)
  };
}
const F8 = (t) => {
  const { width: r, height: i, x: u, y: o } = Qu(t.nodeLookup, {
    filter: (s) => !!s.selected
  });
  return {
    width: dn(r) ? r : null,
    height: dn(i) ? i : null,
    userSelectionActive: t.userSelectionActive,
    transformString: `translate(${t.transform[0]}px,${t.transform[1]}px) scale(${t.transform[2]}) translate(${u}px,${o}px)`
  };
};
function J8({ onSelectionContextMenu: t, noPanClassName: r, disableKeyboardA11y: i }) {
  const u = Qe(), { width: o, height: s, transformString: c, userSelectionActive: f } = ze(F8, Je), g = tq(), h = re.useRef(null);
  re.useEffect(() => {
    var b;
    i || (b = h.current) == null || b.focus({
      preventScroll: !0
    });
  }, [i]);
  const v = !f && o !== null && s !== null;
  if (eq({
    nodeRef: h,
    disabled: !v
  }), !v)
    return null;
  const p = t ? (b) => {
    const _ = u.getState().nodes.filter((A) => A.selected);
    t(b, _);
  } : void 0, m = (b) => {
    Object.prototype.hasOwnProperty.call(Xo, b.key) && (b.preventDefault(), g({
      direction: Xo[b.key],
      factor: b.shiftKey ? 4 : 1
    }));
  };
  return J.jsx("div", { className: ct(["react-flow__nodesselection", "react-flow__container", r]), style: {
    transform: c
  }, children: J.jsx("div", { ref: h, className: "react-flow__nodesselection-rect", onContextMenu: p, tabIndex: i ? void 0 : -1, onKeyDown: i ? void 0 : m, style: {
    width: o,
    height: s
  } }) });
}
const TA = typeof window < "u" ? window : void 0, P8 = (t) => ({ nodesSelectionActive: t.nodesSelectionActive, userSelectionActive: t.userSelectionActive });
function aq({ children: t, onPaneClick: r, onPaneMouseEnter: i, onPaneMouseMove: u, onPaneMouseLeave: o, onPaneContextMenu: s, onPaneScroll: c, paneClickDistance: f, deleteKeyCode: g, selectionKeyCode: h, selectionOnDrag: v, selectionMode: p, onSelectionStart: m, onSelectionEnd: b, multiSelectionKeyCode: _, panActivationKeyCode: A, zoomActivationKeyCode: w, elementsSelectable: E, zoomOnScroll: M, zoomOnPinch: S, panOnScroll: T, panOnScrollSpeed: O, panOnScrollMode: C, zoomOnDoubleClick: R, panOnDrag: H, autoPanOnSelection: B, defaultViewport: X, translateExtent: Y, minZoom: F, maxZoom: K, preventScrolling: D, onSelectionContextMenu: G, noWheelClassName: N, noPanClassName: j, disableKeyboardA11y: Z, onViewportChange: Q, isControlledViewport: le }) {
  const { nodesSelectionActive: z, userSelectionActive: V } = ze(P8, Je), ie = Lu(h, { target: TA }), L = Lu(A, { target: TA }), I = L || H, P = L || T, ae = v && I !== !0, W = ie || V || ae;
  return C8({ deleteKeyCode: g, multiSelectionKeyCode: _ }), J.jsx(O8, { onPaneContextMenu: s, elementsSelectable: E, zoomOnScroll: M, zoomOnPinch: S, panOnScroll: P, panOnScrollSpeed: O, panOnScrollMode: C, zoomOnDoubleClick: R, panOnDrag: !ie && I, defaultViewport: X, translateExtent: Y, minZoom: F, maxZoom: K, zoomActivationKeyCode: w, preventScrolling: D, noWheelClassName: N, noPanClassName: j, onViewportChange: Q, isControlledViewport: le, paneClickDistance: f, selectionOnDrag: ae, children: J.jsxs(L8, { onSelectionStart: m, onSelectionEnd: b, onPaneClick: r, onPaneMouseEnter: i, onPaneMouseMove: u, onPaneMouseLeave: o, onPaneContextMenu: s, onPaneScroll: c, panOnDrag: I, autoPanOnSelection: B, isSelecting: !!W, selectionMode: p, selectionKeyPressed: ie, paneClickDistance: f, selectionOnDrag: ae, children: [t, z && J.jsx(J8, { onSelectionContextMenu: G, noPanClassName: j, disableKeyboardA11y: Z })] }) });
}
aq.displayName = "FlowRenderer";
const W8 = re.memo(aq), eH = (t) => (r) => t ? am(r.nodeLookup, { x: 0, y: 0, width: r.width, height: r.height }, r.transform, !0).map((i) => i.id) : Array.from(r.nodeLookup.keys());
function tH(t) {
  return ze(re.useCallback(eH(t), [t]), Je);
}
const nH = (t) => t.updateNodeInternals;
function rH() {
  const t = ze(nH), [r] = re.useState(() => typeof ResizeObserver > "u" ? null : new ResizeObserver((i) => {
    const u = /* @__PURE__ */ new Map();
    i.forEach((o) => {
      const s = o.target.getAttribute("data-id");
      u.set(s, {
        id: s,
        nodeElement: o.target,
        force: !0
      });
    }), t(u);
  }));
  return re.useEffect(() => () => {
    r == null || r.disconnect();
  }, [r]), r;
}
function aH({ node: t, nodeType: r, hasDimensions: i, resizeObserver: u }) {
  const o = Qe(), s = re.useRef(null), c = re.useRef(null), f = re.useRef(t.sourcePosition), g = re.useRef(t.targetPosition), h = re.useRef(r), v = i && !!t.internals.handleBounds;
  return re.useEffect(() => {
    s.current && !t.hidden && (!v || c.current !== s.current) && (c.current && (u == null || u.unobserve(c.current)), u == null || u.observe(s.current), c.current = s.current);
  }, [v, t.hidden]), re.useEffect(() => () => {
    c.current && (u == null || u.unobserve(c.current), c.current = null);
  }, []), re.useEffect(() => {
    if (s.current) {
      const p = h.current !== r, m = f.current !== t.sourcePosition, b = g.current !== t.targetPosition;
      (p || m || b) && (h.current = r, f.current = t.sourcePosition, g.current = t.targetPosition, o.getState().updateNodeInternals(/* @__PURE__ */ new Map([[t.id, { id: t.id, nodeElement: s.current, force: !0 }]])));
    }
  }, [t.id, r, t.sourcePosition, t.targetPosition]), s;
}
function iH({ id: t, onClick: r, onMouseEnter: i, onMouseMove: u, onMouseLeave: o, onContextMenu: s, onDoubleClick: c, nodesDraggable: f, elementsSelectable: g, nodesConnectable: h, nodesFocusable: v, resizeObserver: p, noDragClassName: m, noPanClassName: b, disableKeyboardA11y: _, rfId: A, nodeTypes: w, nodeClickDistance: E, onError: M }) {
  const { node: S, internals: T, isParent: O } = ze((W) => {
    const se = W.nodeLookup.get(t), de = W.parentLookup.has(t);
    return {
      node: se,
      internals: se.internals,
      isParent: de
    };
  }, Je);
  let C = S.type || "default", R = (w == null ? void 0 : w[C]) || AA[C];
  R === void 0 && (M == null || M("003", vn.error003(C)), C = "default", R = (w == null ? void 0 : w.default) || AA.default);
  const H = !!(S.draggable || f && typeof S.draggable > "u"), B = !!(S.selectable || g && typeof S.selectable > "u"), X = !!(S.connectable || h && typeof S.connectable > "u"), Y = !!(S.focusable || v && typeof S.focusable > "u"), F = Qe(), K = AM(S), D = aH({ node: S, nodeType: C, hasDimensions: K, resizeObserver: p }), G = eq({
    nodeRef: D,
    disabled: S.hidden || !H,
    noDragClassName: m,
    handleSelector: S.dragHandle,
    nodeId: t,
    isSelectable: B,
    nodeClickDistance: E
  }), N = tq();
  if (S.hidden)
    return null;
  const j = qn(S), Z = $8(S), Q = B || H || r || i || u || o, le = i ? (W) => i(W, { ...T.userNode }) : void 0, z = u ? (W) => u(W, { ...T.userNode }) : void 0, V = o ? (W) => o(W, { ...T.userNode }) : void 0, ie = s ? (W) => s(W, { ...T.userNode }) : void 0, L = c ? (W) => c(W, { ...T.userNode }) : void 0, I = (W) => {
    const { selectNodesOnDrag: se, nodeDragThreshold: de } = F.getState();
    B && (!se || !H || de > 0) && Rp({
      id: t,
      store: F,
      nodeRef: D
    }), r && r(W, { ...T.userNode });
  }, P = (W) => {
    if (!(qM(W.nativeEvent) || _)) {
      if (vM.includes(W.key) && B) {
        const se = W.key === "Escape";
        Rp({
          id: t,
          store: F,
          unselect: se,
          nodeRef: D
        });
      } else if (H && S.selected && Object.prototype.hasOwnProperty.call(Xo, W.key)) {
        W.preventDefault();
        const { ariaLabelConfig: se } = F.getState();
        F.setState({
          ariaLiveMessage: se["node.a11yDescription.ariaLiveMessage"]({
            direction: W.key.replace("Arrow", "").toLowerCase(),
            x: ~~T.positionAbsolute.x,
            y: ~~T.positionAbsolute.y
          })
        }), N({
          direction: Xo[W.key],
          factor: W.shiftKey ? 4 : 1
        });
      }
    }
  }, ae = () => {
    var me;
    if (_ || !((me = D.current) != null && me.matches(":focus-visible")))
      return;
    const { transform: W, width: se, height: de, autoPanOnNodeFocus: ve, setCenter: pe } = F.getState();
    if (!ve)
      return;
    am(/* @__PURE__ */ new Map([[t, S]]), { x: 0, y: 0, width: se, height: de }, W, !0).length > 0 || pe(S.position.x + j.width / 2, S.position.y + j.height / 2, {
      zoom: W[2]
    });
  };
  return J.jsx("div", { className: ct([
    "react-flow__node",
    `react-flow__node-${C}`,
    {
      // this is overwritable by passing `nopan` as a class name
      [b]: H
    },
    S.className,
    {
      selected: S.selected,
      selectable: B,
      parent: O,
      draggable: H,
      dragging: G
    }
  ]), ref: D, style: {
    zIndex: T.z,
    transform: `translate(${T.positionAbsolute.x}px,${T.positionAbsolute.y}px)`,
    pointerEvents: Q ? "all" : "none",
    visibility: K ? "visible" : "hidden",
    ...S.style,
    ...Z
  }, "data-id": t, "data-testid": `rf__node-${t}`, onMouseEnter: le, onMouseMove: z, onMouseLeave: V, onContextMenu: ie, onClick: I, onDoubleClick: L, onKeyDown: Y ? P : void 0, tabIndex: Y ? 0 : void 0, onFocus: Y ? ae : void 0, role: S.ariaRole ?? (Y ? "group" : void 0), "aria-roledescription": "node", "aria-describedby": _ ? void 0 : `${ZM}-${A}`, "aria-label": S.ariaLabel, ...S.domAttributes, children: J.jsx(j8, { value: t, children: J.jsx(R, { id: t, data: S.data, type: C, positionAbsoluteX: T.positionAbsolute.x, positionAbsoluteY: T.positionAbsolute.y, selected: S.selected ?? !1, selectable: B, draggable: H, deletable: S.deletable ?? !0, isConnectable: X, sourcePosition: S.sourcePosition, targetPosition: S.targetPosition, dragging: G, dragHandle: S.dragHandle, zIndex: T.z, parentId: S.parentId, ...j }) }) });
}
var uH = re.memo(iH);
const lH = (t) => ({
  nodesConnectable: t.nodesConnectable,
  nodesFocusable: t.nodesFocusable,
  elementsSelectable: t.elementsSelectable,
  onError: t.onError
});
function iq(t) {
  const { nodesConnectable: r, nodesFocusable: i, elementsSelectable: u, onError: o } = ze(lH, Je), s = tH(t.onlyRenderVisibleElements), c = rH();
  return J.jsx("div", { className: "react-flow__nodes", style: ms, children: s.map((f) => (
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
    J.jsx(uH, { id: f, nodeTypes: t.nodeTypes, nodeExtent: t.nodeExtent, onClick: t.onNodeClick, onMouseEnter: t.onNodeMouseEnter, onMouseMove: t.onNodeMouseMove, onMouseLeave: t.onNodeMouseLeave, onContextMenu: t.onNodeContextMenu, onDoubleClick: t.onNodeDoubleClick, noDragClassName: t.noDragClassName, noPanClassName: t.noPanClassName, rfId: t.rfId, disableKeyboardA11y: t.disableKeyboardA11y, resizeObserver: c, nodesDraggable: t.nodesDraggable ?? !0, nodesConnectable: r, nodesFocusable: i, elementsSelectable: u, nodeClickDistance: t.nodeClickDistance, onError: o }, f)
  )) });
}
iq.displayName = "NodeRenderer";
const oH = re.memo(iq);
function sH(t) {
  return ze(re.useCallback((i) => {
    if (!t)
      return i.edges.map((o) => o.id);
    const u = [];
    if (i.width && i.height)
      for (const o of i.edges) {
        const s = i.nodeLookup.get(o.source), c = i.nodeLookup.get(o.target);
        s && c && n6({
          sourceNode: s,
          targetNode: c,
          width: i.width,
          height: i.height,
          transform: i.transform
        }) && u.push(o.id);
      }
    return u;
  }, [t]), Je);
}
const cH = ({ color: t = "none", strokeWidth: r = 1 }) => {
  const i = {
    strokeWidth: r,
    ...t && { stroke: t }
  };
  return J.jsx("polyline", { className: "arrow", style: i, strokeLinecap: "round", fill: "none", strokeLinejoin: "round", points: "-5,-4 0,0 -5,4" });
}, fH = ({ color: t = "none", strokeWidth: r = 1 }) => {
  const i = {
    strokeWidth: r,
    ...t && { stroke: t, fill: t }
  };
  return J.jsx("polyline", { className: "arrowclosed", style: i, strokeLinecap: "round", strokeLinejoin: "round", points: "-5,-4 0,0 -5,4 -5,-4" });
}, MA = {
  [Vo.Arrow]: cH,
  [Vo.ArrowClosed]: fH
};
function dH(t) {
  const r = Qe();
  return re.useMemo(() => {
    var o, s;
    return Object.prototype.hasOwnProperty.call(MA, t) ? MA[t] : ((s = (o = r.getState()).onError) == null || s.call(o, "009", vn.error009(t)), null);
  }, [t]);
}
const hH = ({ id: t, type: r, color: i, width: u = 12.5, height: o = 12.5, markerUnits: s = "strokeWidth", strokeWidth: c, orient: f = "auto-start-reverse" }) => {
  const g = dH(r);
  return g ? J.jsx("marker", { className: "react-flow__arrowhead", id: t, markerWidth: `${u}`, markerHeight: `${o}`, viewBox: "-10 -10 20 20", markerUnits: s, orient: f, refX: "0", refY: "0", children: J.jsx(g, { color: i, strokeWidth: c }) }) : null;
}, uq = ({ defaultColor: t, rfId: r }) => {
  const i = ze((s) => s.edges), u = ze((s) => s.defaultEdgeOptions), o = re.useMemo(() => c6(i, {
    id: r,
    defaultColor: t,
    defaultMarkerStart: u == null ? void 0 : u.markerStart,
    defaultMarkerEnd: u == null ? void 0 : u.markerEnd
  }), [i, u, r, t]);
  return o.length ? J.jsx("svg", { className: "react-flow__marker", "aria-hidden": "true", children: J.jsx("defs", { children: o.map((s) => J.jsx(hH, { id: s.id, type: s.type, color: s.color, width: s.width, height: s.height, markerUnits: s.markerUnits, strokeWidth: s.strokeWidth, orient: s.orient }, s.id)) }) }) : null;
};
uq.displayName = "MarkerDefinitions";
var gH = re.memo(uq);
function lq({ x: t, y: r, label: i, labelStyle: u, labelShowBg: o = !0, labelBgStyle: s, labelBgPadding: c = [2, 4], labelBgBorderRadius: f = 2, children: g, className: h, ...v }) {
  const [p, m] = re.useState({ x: 1, y: 0, width: 0, height: 0 }), b = ct(["react-flow__edge-textwrapper", h]), _ = re.useRef(null);
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
  }, [i]), i ? J.jsxs("g", { transform: `translate(${t - p.width / 2} ${r - p.height / 2})`, className: b, visibility: p.width ? "visible" : "hidden", ...v, children: [o && J.jsx("rect", { width: p.width + 2 * c[0], x: -c[0], y: -c[1], height: p.height + 2 * c[1], className: "react-flow__edge-textbg", style: s, rx: f, ry: f }), J.jsx("text", { className: "react-flow__edge-text", y: p.height / 2, dy: "0.3em", ref: _, style: u, children: i }), g] }) : null;
}
lq.displayName = "EdgeText";
const vH = re.memo(lq);
function bs({ path: t, labelX: r, labelY: i, label: u, labelStyle: o, labelShowBg: s, labelBgStyle: c, labelBgPadding: f, labelBgBorderRadius: g, interactionWidth: h = 20, ...v }) {
  return J.jsxs(J.Fragment, { children: [J.jsx("path", { ...v, d: t, fill: "none", className: ct(["react-flow__edge-path", v.className]) }), h ? J.jsx("path", { d: t, fill: "none", strokeOpacity: 0, strokeWidth: h, className: "react-flow__edge-interaction" }) : null, u && dn(r) && dn(i) ? J.jsx(vH, { x: r, y: i, label: u, labelStyle: o, labelShowBg: s, labelBgStyle: c, labelBgPadding: f, labelBgBorderRadius: g }) : null] });
}
function qA({ pos: t, x1: r, y1: i, x2: u, y2: o }) {
  return t === _e.Left || t === _e.Right ? [0.5 * (r + u), i] : [r, 0.5 * (i + o)];
}
function oq({ sourceX: t, sourceY: r, sourcePosition: i = _e.Bottom, targetX: u, targetY: o, targetPosition: s = _e.Top }) {
  const [c, f] = qA({
    pos: i,
    x1: t,
    y1: r,
    x2: u,
    y2: o
  }), [g, h] = qA({
    pos: s,
    x1: u,
    y1: o,
    x2: t,
    y2: r
  }), [v, p, m, b] = RM({
    sourceX: t,
    sourceY: r,
    targetX: u,
    targetY: o,
    sourceControlX: c,
    sourceControlY: f,
    targetControlX: g,
    targetControlY: h
  });
  return [
    `M${t},${r} C${c},${f} ${g},${h} ${u},${o}`,
    v,
    p,
    m,
    b
  ];
}
function sq(t) {
  return re.memo(({ id: r, sourceX: i, sourceY: u, targetX: o, targetY: s, sourcePosition: c, targetPosition: f, label: g, labelStyle: h, labelShowBg: v, labelBgStyle: p, labelBgPadding: m, labelBgBorderRadius: b, style: _, markerEnd: A, markerStart: w, interactionWidth: E }) => {
    const [M, S, T] = oq({
      sourceX: i,
      sourceY: u,
      sourcePosition: c,
      targetX: o,
      targetY: s,
      targetPosition: f
    }), O = t.isInternal ? void 0 : r;
    return J.jsx(bs, { id: O, path: M, labelX: S, labelY: T, label: g, labelStyle: h, labelShowBg: v, labelBgStyle: p, labelBgPadding: m, labelBgBorderRadius: b, style: _, markerEnd: A, markerStart: w, interactionWidth: E });
  });
}
const yH = sq({ isInternal: !1 }), cq = sq({ isInternal: !0 });
yH.displayName = "SimpleBezierEdge";
cq.displayName = "SimpleBezierEdgeInternal";
function fq(t) {
  return re.memo(({ id: r, sourceX: i, sourceY: u, targetX: o, targetY: s, label: c, labelStyle: f, labelShowBg: g, labelBgStyle: h, labelBgPadding: v, labelBgBorderRadius: p, style: m, sourcePosition: b = _e.Bottom, targetPosition: _ = _e.Top, markerEnd: A, markerStart: w, pathOptions: E, interactionWidth: M }) => {
    const [S, T, O] = Tp({
      sourceX: i,
      sourceY: u,
      sourcePosition: b,
      targetX: o,
      targetY: s,
      targetPosition: _,
      borderRadius: E == null ? void 0 : E.borderRadius,
      offset: E == null ? void 0 : E.offset,
      stepPosition: E == null ? void 0 : E.stepPosition
    }), C = t.isInternal ? void 0 : r;
    return J.jsx(bs, { id: C, path: S, labelX: T, labelY: O, label: c, labelStyle: f, labelShowBg: g, labelBgStyle: h, labelBgPadding: v, labelBgBorderRadius: p, style: m, markerEnd: A, markerStart: w, interactionWidth: M });
  });
}
const dq = fq({ isInternal: !1 }), hq = fq({ isInternal: !0 });
dq.displayName = "SmoothStepEdge";
hq.displayName = "SmoothStepEdgeInternal";
function gq(t) {
  return re.memo(({ id: r, ...i }) => {
    var o;
    const u = t.isInternal ? void 0 : r;
    return J.jsx(dq, { ...i, id: u, pathOptions: re.useMemo(() => {
      var s;
      return { borderRadius: 0, offset: (s = i.pathOptions) == null ? void 0 : s.offset };
    }, [(o = i.pathOptions) == null ? void 0 : o.offset]) });
  });
}
const pH = gq({ isInternal: !1 }), vq = gq({ isInternal: !0 });
pH.displayName = "StepEdge";
vq.displayName = "StepEdgeInternal";
function yq(t) {
  return re.memo(({ id: r, sourceX: i, sourceY: u, targetX: o, targetY: s, label: c, labelStyle: f, labelShowBg: g, labelBgStyle: h, labelBgPadding: v, labelBgBorderRadius: p, style: m, markerEnd: b, markerStart: _, interactionWidth: A }) => {
    const [w, E, M] = zM({ sourceX: i, sourceY: u, targetX: o, targetY: s }), S = t.isInternal ? void 0 : r;
    return J.jsx(bs, { id: S, path: w, labelX: E, labelY: M, label: c, labelStyle: f, labelShowBg: g, labelBgStyle: h, labelBgPadding: v, labelBgBorderRadius: p, style: m, markerEnd: b, markerStart: _, interactionWidth: A });
  });
}
const mH = yq({ isInternal: !1 }), pq = yq({ isInternal: !0 });
mH.displayName = "StraightEdge";
pq.displayName = "StraightEdgeInternal";
function mq(t) {
  return re.memo(({ id: r, sourceX: i, sourceY: u, targetX: o, targetY: s, sourcePosition: c = _e.Bottom, targetPosition: f = _e.Top, label: g, labelStyle: h, labelShowBg: v, labelBgStyle: p, labelBgPadding: m, labelBgBorderRadius: b, style: _, markerEnd: A, markerStart: w, pathOptions: E, interactionWidth: M }) => {
    const [S, T, O] = NM({
      sourceX: i,
      sourceY: u,
      sourcePosition: c,
      targetX: o,
      targetY: s,
      targetPosition: f,
      curvature: E == null ? void 0 : E.curvature
    }), C = t.isInternal ? void 0 : r;
    return J.jsx(bs, { id: C, path: S, labelX: T, labelY: O, label: g, labelStyle: h, labelShowBg: v, labelBgStyle: p, labelBgPadding: m, labelBgBorderRadius: b, style: _, markerEnd: A, markerStart: w, interactionWidth: M });
  });
}
const bH = mq({ isInternal: !1 }), bq = mq({ isInternal: !0 });
bH.displayName = "BezierEdge";
bq.displayName = "BezierEdgeInternal";
const CA = {
  default: bq,
  straight: pq,
  step: vq,
  smoothstep: hq,
  simplebezier: cq
}, RA = {
  sourceX: null,
  sourceY: null,
  targetX: null,
  targetY: null,
  sourcePosition: null,
  targetPosition: null,
  zIndex: void 0
}, _H = (t, r, i) => i === _e.Left ? t - r : i === _e.Right ? t + r : t, xH = (t, r, i) => i === _e.Top ? t - r : i === _e.Bottom ? t + r : t, NA = "react-flow__edgeupdater";
function OA({ position: t, centerX: r, centerY: i, radius: u = 10, onMouseDown: o, onMouseEnter: s, onMouseOut: c, type: f }) {
  return J.jsx("circle", { onMouseDown: o, onMouseEnter: s, onMouseOut: c, className: ct([NA, `${NA}-${f}`]), cx: _H(r, u, t), cy: xH(i, u, t), r: u, stroke: "transparent", fill: "transparent" });
}
function SH({ isReconnectable: t, reconnectRadius: r, edge: i, sourceX: u, sourceY: o, targetX: s, targetY: c, sourcePosition: f, targetPosition: g, onReconnect: h, onReconnectStart: v, onReconnectEnd: p, setReconnecting: m, setUpdateHover: b }) {
  const _ = Qe(), A = (T, O) => {
    if (T.button !== 0)
      return;
    const { autoPanOnConnect: C, domNode: R, connectionMode: H, connectionRadius: B, lib: X, onConnectStart: Y, cancelConnection: F, nodeLookup: K, rfId: D, panBy: G, updateConnection: N } = _.getState(), j = O.type === "target", Z = (z, V) => {
      m(!1), p == null || p(z, i, O.type, V);
    }, Q = (z) => h == null ? void 0 : h(i, z), le = (z, V) => {
      m(!0), v == null || v(T, i, O.type), Y == null || Y(z, V);
    };
    Cp.onPointerDown(T.nativeEvent, {
      autoPanOnConnect: C,
      connectionMode: H,
      connectionRadius: B,
      domNode: R,
      handleId: O.id,
      nodeId: O.nodeId,
      nodeLookup: K,
      isTarget: j,
      edgeUpdaterType: O.type,
      lib: X,
      flowId: D,
      cancelConnection: F,
      panBy: G,
      isValidConnection: (...z) => {
        var V, ie;
        return ((ie = (V = _.getState()).isValidConnection) == null ? void 0 : ie.call(V, ...z)) ?? !0;
      },
      onConnect: Q,
      onConnectStart: le,
      onConnectEnd: (...z) => {
        var V, ie;
        return (ie = (V = _.getState()).onConnectEnd) == null ? void 0 : ie.call(V, ...z);
      },
      onReconnectEnd: Z,
      updateConnection: N,
      getTransform: () => _.getState().transform,
      getFromHandle: () => _.getState().connection.fromHandle,
      dragThreshold: _.getState().connectionDragThreshold,
      handleDomNode: T.currentTarget
    });
  }, w = (T) => A(T, { nodeId: i.target, id: i.targetHandle ?? null, type: "target" }), E = (T) => A(T, { nodeId: i.source, id: i.sourceHandle ?? null, type: "source" }), M = () => b(!0), S = () => b(!1);
  return J.jsxs(J.Fragment, { children: [(t === !0 || t === "source") && J.jsx(OA, { position: f, centerX: u, centerY: o, radius: r, onMouseDown: w, onMouseEnter: M, onMouseOut: S, type: "source" }), (t === !0 || t === "target") && J.jsx(OA, { position: g, centerX: s, centerY: c, radius: r, onMouseDown: E, onMouseEnter: M, onMouseOut: S, type: "target" })] });
}
function EH({ id: t, edgesFocusable: r, edgesReconnectable: i, elementsSelectable: u, onClick: o, onDoubleClick: s, onContextMenu: c, onMouseEnter: f, onMouseMove: g, onMouseLeave: h, reconnectRadius: v, onReconnect: p, onReconnectStart: m, onReconnectEnd: b, rfId: _, edgeTypes: A, noPanClassName: w, onError: E, disableKeyboardA11y: M }) {
  let S = ze((pe) => pe.edgeLookup.get(t));
  const T = ze((pe) => pe.defaultEdgeOptions);
  S = T ? { ...T, ...S } : S;
  let O = S.type || "default", C = (A == null ? void 0 : A[O]) || CA[O];
  C === void 0 && (E == null || E("011", vn.error011(O)), O = "default", C = (A == null ? void 0 : A.default) || CA.default);
  const R = !!(S.focusable || r && typeof S.focusable > "u"), H = typeof p < "u" && (S.reconnectable || i && typeof S.reconnectable > "u"), B = !!(S.selectable || u && typeof S.selectable > "u"), X = re.useRef(null), [Y, F] = re.useState(!1), [K, D] = re.useState(!1), G = Qe(), { zIndex: N = S.zIndex, sourceX: j, sourceY: Z, targetX: Q, targetY: le, sourcePosition: z, targetPosition: V } = ze(re.useCallback((pe) => {
    const he = pe.nodeLookup.get(S.source), me = pe.nodeLookup.get(S.target);
    if (!he || !me)
      return RA;
    const ge = s6({
      id: t,
      sourceNode: he,
      targetNode: me,
      sourceHandle: S.sourceHandle || null,
      targetHandle: S.targetHandle || null,
      connectionMode: pe.connectionMode,
      onError: E
    }), Ae = t6({
      selected: S.selected,
      zIndex: S.zIndex,
      sourceNode: he,
      targetNode: me,
      elevateOnSelect: pe.elevateEdgesOnSelect,
      zIndexMode: pe.zIndexMode
    });
    return {
      ...ge || RA,
      zIndex: Ae
    };
  }, [S.source, S.target, S.sourceHandle, S.targetHandle, S.selected, S.zIndex]), Je), ie = re.useMemo(() => S.markerStart ? `url('#${Mp(S.markerStart, _)}')` : void 0, [S.markerStart, _]), L = re.useMemo(() => S.markerEnd ? `url('#${Mp(S.markerEnd, _)}')` : void 0, [S.markerEnd, _]);
  if (S.hidden || j === null || Z === null || Q === null || le === null)
    return null;
  const I = (pe) => {
    var Ae;
    const { addSelectedEdges: he, unselectNodesAndEdges: me, multiSelectionActive: ge } = G.getState();
    B && (G.setState({ nodesSelectionActive: !1 }), S.selected && ge ? (me({ nodes: [], edges: [S] }), (Ae = X.current) == null || Ae.blur()) : he([t])), o && o(pe, S);
  }, P = s ? (pe) => {
    s(pe, { ...S });
  } : void 0, ae = c ? (pe) => {
    c(pe, { ...S });
  } : void 0, W = f ? (pe) => {
    f(pe, { ...S });
  } : void 0, se = g ? (pe) => {
    g(pe, { ...S });
  } : void 0, de = h ? (pe) => {
    h(pe, { ...S });
  } : void 0, ve = (pe) => {
    var he;
    if (!M && vM.includes(pe.key) && B) {
      const { unselectNodesAndEdges: me, addSelectedEdges: ge } = G.getState();
      pe.key === "Escape" ? ((he = X.current) == null || he.blur(), me({ edges: [S] })) : ge([t]);
    }
  };
  return J.jsx("svg", { style: { zIndex: N }, children: J.jsxs("g", { className: ct([
    "react-flow__edge",
    `react-flow__edge-${O}`,
    S.className,
    w,
    {
      selected: S.selected,
      animated: S.animated,
      inactive: !B && !o,
      updating: Y,
      selectable: B
    }
  ]), onClick: I, onDoubleClick: P, onContextMenu: ae, onMouseEnter: W, onMouseMove: se, onMouseLeave: de, onKeyDown: R ? ve : void 0, tabIndex: R ? 0 : void 0, role: S.ariaRole ?? (R ? "group" : "img"), "aria-roledescription": "edge", "data-id": t, "data-testid": `rf__edge-${t}`, "aria-label": S.ariaLabel === null ? void 0 : S.ariaLabel || `Edge from ${S.source} to ${S.target}`, "aria-describedby": R ? `${KM}-${_}` : void 0, ref: X, ...S.domAttributes, children: [!K && J.jsx(C, { id: t, source: S.source, target: S.target, type: S.type, selected: S.selected, animated: S.animated, selectable: B, deletable: S.deletable ?? !0, label: S.label, labelStyle: S.labelStyle, labelShowBg: S.labelShowBg, labelBgStyle: S.labelBgStyle, labelBgPadding: S.labelBgPadding, labelBgBorderRadius: S.labelBgBorderRadius, sourceX: j, sourceY: Z, targetX: Q, targetY: le, sourcePosition: z, targetPosition: V, data: S.data, style: S.style, sourceHandleId: S.sourceHandle, targetHandleId: S.targetHandle, markerStart: ie, markerEnd: L, pathOptions: "pathOptions" in S ? S.pathOptions : void 0, interactionWidth: S.interactionWidth }), H && J.jsx(SH, { edge: S, isReconnectable: H, reconnectRadius: v, onReconnect: p, onReconnectStart: m, onReconnectEnd: b, sourceX: j, sourceY: Z, targetX: Q, targetY: le, sourcePosition: z, targetPosition: V, setUpdateHover: F, setReconnecting: D })] }) });
}
var wH = re.memo(EH);
const AH = (t) => ({
  edgesFocusable: t.edgesFocusable,
  edgesReconnectable: t.edgesReconnectable,
  elementsSelectable: t.elementsSelectable,
  connectionMode: t.connectionMode,
  onError: t.onError
});
function _q({ defaultMarkerColor: t, onlyRenderVisibleElements: r, rfId: i, edgeTypes: u, noPanClassName: o, onReconnect: s, onEdgeContextMenu: c, onEdgeMouseEnter: f, onEdgeMouseMove: g, onEdgeMouseLeave: h, onEdgeClick: v, reconnectRadius: p, onEdgeDoubleClick: m, onReconnectStart: b, onReconnectEnd: _, disableKeyboardA11y: A }) {
  const { edgesFocusable: w, edgesReconnectable: E, elementsSelectable: M, onError: S } = ze(AH, Je), T = sH(r);
  return J.jsxs("div", { className: "react-flow__edges", children: [J.jsx(gH, { defaultColor: t, rfId: i }), T.map((O) => J.jsx(wH, { id: O, edgesFocusable: w, edgesReconnectable: E, elementsSelectable: M, noPanClassName: o, onReconnect: s, onContextMenu: c, onMouseEnter: f, onMouseMove: g, onMouseLeave: h, onClick: v, reconnectRadius: p, onDoubleClick: m, onReconnectStart: b, onReconnectEnd: _, rfId: i, onError: S, edgeTypes: u, disableKeyboardA11y: A }, O))] });
}
_q.displayName = "EdgeRenderer";
const TH = re.memo(_q), zA = (t) => `translate(${t[0]}px,${t[1]}px) scale(${t[2]})`;
function MH({ children: t }) {
  const r = Qe(), i = re.useRef(null), [u] = re.useState(() => r.getState().transform);
  return PM(() => {
    let o = null;
    const s = () => {
      const c = r.getState().transform;
      o && c[0] === o[0] && c[1] === o[1] && c[2] === o[2] || (o = c, i.current && (i.current.style.transform = zA(c)));
    };
    return s(), r.subscribe(s);
  }, [r]), J.jsx("div", { ref: i, className: "react-flow__viewport xyflow__viewport react-flow__container", style: { transform: zA(u) }, children: t });
}
function qH(t) {
  const r = hm(), i = re.useRef(!1);
  re.useEffect(() => {
    !i.current && r.viewportInitialized && t && (setTimeout(() => t(r), 1), i.current = !0);
  }, [t, r.viewportInitialized]);
}
const CH = (t) => {
  var r;
  return (r = t.panZoom) == null ? void 0 : r.syncViewport;
};
function RH(t) {
  const r = ze(CH), i = Qe();
  return re.useEffect(() => {
    t && (r == null || r(t), i.setState({ transform: [t.x, t.y, t.zoom] }));
  }, [t, r]), null;
}
function NH(t) {
  return t.connection.inProgress ? { ...t.connection, to: Ku(t.connection.to, t.transform) } : { ...t.connection };
}
function OH(t) {
  return NH;
}
function zH(t) {
  const r = OH();
  return ze(r, Je);
}
const DH = (t) => ({
  nodesConnectable: t.nodesConnectable,
  isValid: t.connection.isValid,
  inProgress: t.connection.inProgress,
  width: t.width,
  height: t.height
});
function HH({ containerStyle: t, style: r, type: i, component: u }) {
  const { nodesConnectable: o, width: s, height: c, isValid: f, inProgress: g } = ze(DH, Je);
  return !(s && o && g) ? null : J.jsx("svg", { style: t, width: s, height: c, className: "react-flow__connectionline react-flow__container", children: J.jsx("g", { className: ct(["react-flow__connection", mM(f)]), children: J.jsx(xq, { style: r, type: i, CustomComponent: u, isValid: f }) }) });
}
const xq = ({ style: t, type: r = zr.Bezier, CustomComponent: i, isValid: u }) => {
  const { inProgress: o, from: s, fromNode: c, fromHandle: f, fromPosition: g, to: h, toNode: v, toHandle: p, toPosition: m, pointer: b } = zH();
  if (!o)
    return;
  if (i)
    return J.jsx(i, { connectionLineType: r, connectionLineStyle: t, fromNode: c, fromHandle: f, fromX: s.x, fromY: s.y, toX: h.x, toY: h.y, fromPosition: g, toPosition: m, connectionStatus: mM(u), toNode: v, toHandle: p, pointer: b });
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
    case zr.Bezier:
      [_] = NM(A);
      break;
    case zr.SimpleBezier:
      [_] = oq(A);
      break;
    case zr.Step:
      [_] = Tp({
        ...A,
        borderRadius: 0
      });
      break;
    case zr.SmoothStep:
      [_] = Tp(A);
      break;
    default:
      [_] = zM(A);
  }
  return J.jsx("path", { d: _, fill: "none", className: "react-flow__connection-path", style: t });
};
xq.displayName = "ConnectionLine";
const LH = {};
function DA(t = LH) {
  re.useRef(t), Qe(), re.useEffect(() => {
  }, [t]);
}
function BH() {
  Qe(), re.useRef(!1), re.useEffect(() => {
  }, []);
}
function Sq({ nodeTypes: t, edgeTypes: r, onInit: i, onNodeClick: u, onEdgeClick: o, onNodeDoubleClick: s, onEdgeDoubleClick: c, onNodeMouseEnter: f, onNodeMouseMove: g, onNodeMouseLeave: h, onNodeContextMenu: v, onSelectionContextMenu: p, onSelectionStart: m, onSelectionEnd: b, connectionLineType: _, connectionLineStyle: A, connectionLineComponent: w, connectionLineContainerStyle: E, selectionKeyCode: M, selectionOnDrag: S, selectionMode: T, multiSelectionKeyCode: O, panActivationKeyCode: C, zoomActivationKeyCode: R, deleteKeyCode: H, onlyRenderVisibleElements: B, elementsSelectable: X, defaultViewport: Y, translateExtent: F, minZoom: K, maxZoom: D, preventScrolling: G, defaultMarkerColor: N, zoomOnScroll: j, zoomOnPinch: Z, panOnScroll: Q, panOnScrollSpeed: le, panOnScrollMode: z, zoomOnDoubleClick: V, panOnDrag: ie, autoPanOnSelection: L, onPaneClick: I, onPaneMouseEnter: P, onPaneMouseMove: ae, onPaneMouseLeave: W, onPaneScroll: se, onPaneContextMenu: de, paneClickDistance: ve, nodeClickDistance: pe, onEdgeContextMenu: he, onEdgeMouseEnter: me, onEdgeMouseMove: ge, onEdgeMouseLeave: Ae, reconnectRadius: xe, onReconnect: Pe, onReconnectStart: tt, onReconnectEnd: xt, noDragClassName: gt, noWheelClassName: St, noPanClassName: Ze, disableKeyboardA11y: ke, nodeExtent: vt, rfId: Ct, viewport: Tt, onViewportChange: ft, nodesDraggable: tr }) {
  return DA(t), DA(r), BH(), qH(i), RH(Tt), J.jsx(W8, { onPaneClick: I, onPaneMouseEnter: P, onPaneMouseMove: ae, onPaneMouseLeave: W, onPaneContextMenu: de, onPaneScroll: se, paneClickDistance: ve, deleteKeyCode: H, selectionKeyCode: M, selectionOnDrag: S, selectionMode: T, onSelectionStart: m, onSelectionEnd: b, multiSelectionKeyCode: O, panActivationKeyCode: C, zoomActivationKeyCode: R, elementsSelectable: X, zoomOnScroll: j, zoomOnPinch: Z, zoomOnDoubleClick: V, panOnScroll: Q, panOnScrollSpeed: le, panOnScrollMode: z, panOnDrag: ie, autoPanOnSelection: L, defaultViewport: Y, translateExtent: F, minZoom: K, maxZoom: D, onSelectionContextMenu: p, preventScrolling: G, noDragClassName: gt, noWheelClassName: St, noPanClassName: Ze, disableKeyboardA11y: ke, onViewportChange: ft, isControlledViewport: !!Tt, children: J.jsxs(MH, { children: [J.jsx(TH, { edgeTypes: r, onEdgeClick: o, onEdgeDoubleClick: c, onReconnect: Pe, onReconnectStart: tt, onReconnectEnd: xt, onlyRenderVisibleElements: B, onEdgeContextMenu: he, onEdgeMouseEnter: me, onEdgeMouseMove: ge, onEdgeMouseLeave: Ae, reconnectRadius: xe, defaultMarkerColor: N, noPanClassName: Ze, disableKeyboardA11y: ke, rfId: Ct }), J.jsx(HH, { style: A, type: _, component: w, containerStyle: E }), J.jsx("div", { className: "react-flow__edgelabel-renderer" }), J.jsx(oH, { nodeTypes: t, onNodeClick: u, onNodeDoubleClick: s, onNodeMouseEnter: f, onNodeMouseMove: g, onNodeMouseLeave: h, onNodeContextMenu: v, nodeClickDistance: pe, onlyRenderVisibleElements: B, noPanClassName: Ze, noDragClassName: gt, disableKeyboardA11y: ke, nodeExtent: vt, rfId: Ct, nodesDraggable: tr }), J.jsx("div", { className: "react-flow__viewport-portal" })] }) });
}
Sq.displayName = "GraphView";
const jH = re.memo(Sq), UH = wM(), HA = ({ nodes: t, edges: r, defaultNodes: i, defaultEdges: u, width: o, height: s, fitView: c, fitViewOptions: f, minZoom: g = 0.5, maxZoom: h = 2, nodeOrigin: v, nodeExtent: p, zIndexMode: m = "basic" } = {}) => {
  const b = /* @__PURE__ */ new Map(), _ = /* @__PURE__ */ new Map(), A = /* @__PURE__ */ new Map(), w = /* @__PURE__ */ new Map(), E = u ?? r ?? [], M = i ?? t ?? [], S = v ?? [0, 0], T = p ?? Ou;
  LM(A, w, E);
  const { nodesInitialized: O } = qp(M, b, _, {
    nodeOrigin: S,
    nodeExtent: T,
    zIndexMode: m
  });
  let C = [0, 0, 1];
  if (c && o && s) {
    const R = Qu(b, {
      filter: (Y) => !!((Y.width || Y.initialWidth) && (Y.height || Y.initialHeight))
    }), { x: H, y: B, zoom: X } = um(R, o, s, g, h, (f == null ? void 0 : f.padding) ?? 0.1);
    C = [H, B, X];
  }
  return {
    rfId: "1",
    width: o ?? 0,
    height: s ?? 0,
    transform: C,
    nodes: M,
    nodesInitialized: O,
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
    translateExtent: Ou,
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
    connection: { ...pM },
    connectionClickStartHandle: null,
    connectOnClick: !0,
    ariaLiveMessage: "",
    autoPanOnConnect: !0,
    autoPanOnNodeDrag: !0,
    autoPanOnNodeFocus: !0,
    autoPanSpeed: 15,
    connectionRadius: 20,
    onError: UH,
    isValidConnection: void 0,
    onSelectionChangeHandlers: [],
    lib: "react",
    debug: !1,
    ariaLabelConfig: yM,
    zIndexMode: m,
    onNodesChangeMiddlewareMap: /* @__PURE__ */ new Map(),
    onEdgesChangeMiddlewareMap: /* @__PURE__ */ new Map()
  };
}, GH = ({ nodes: t, edges: r, defaultNodes: i, defaultEdges: u, width: o, height: s, fitView: c, fitViewOptions: f, minZoom: g, maxZoom: h, nodeOrigin: v, nodeExtent: p, zIndexMode: m }) => P6((b, _) => {
  async function A() {
    const { nodeLookup: w, panZoom: E, fitViewOptions: M, fitViewResolver: S, width: T, height: O, minZoom: C, maxZoom: R } = _();
    E && (await K5({
      nodes: w,
      width: T,
      height: O,
      panZoom: E,
      minZoom: C,
      maxZoom: R
    }, M), S == null || S.resolve(!0), b({ fitViewResolver: null }));
  }
  return {
    ...HA({
      nodes: t,
      edges: r,
      width: o,
      height: s,
      fitView: c,
      fitViewOptions: f,
      minZoom: g,
      maxZoom: h,
      nodeOrigin: v,
      nodeExtent: p,
      defaultNodes: i,
      defaultEdges: u,
      zIndexMode: m
    }),
    setNodes: (w) => {
      const { nodeLookup: E, parentLookup: M, nodeOrigin: S, elevateNodesOnSelect: T, fitViewQueued: O, zIndexMode: C, nodesSelectionActive: R } = _(), { nodesInitialized: H, hasSelectedNodes: B } = qp(w, E, M, {
        nodeOrigin: S,
        nodeExtent: p,
        elevateNodesOnSelect: T,
        checkEquality: !0,
        zIndexMode: C
      }), X = R && B;
      O && H ? (A(), b({
        nodes: w,
        nodesInitialized: H,
        fitViewQueued: !1,
        fitViewOptions: void 0,
        nodesSelectionActive: X
      })) : b({ nodes: w, nodesInitialized: H, nodesSelectionActive: X });
    },
    setEdges: (w) => {
      const { connectionLookup: E, edgeLookup: M } = _();
      LM(E, M, w), b({ edges: w });
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
      const { triggerNodeChanges: E, nodeLookup: M, parentLookup: S, domNode: T, nodeOrigin: O, nodeExtent: C, debug: R, fitViewQueued: H, zIndexMode: B } = _(), { changes: X, updatedInternals: Y } = p6(w, M, S, T, O, C, B);
      Y && (h6(M, S, { nodeOrigin: O, nodeExtent: C, zIndexMode: B }), H ? (A(), b({ fitViewQueued: !1, fitViewOptions: void 0 })) : b({}), (X == null ? void 0 : X.length) > 0 && (R && console.log("React Flow: trigger node changes", X), E == null || E(X)));
    },
    updateNodePositions: (w, E = !1) => {
      const M = [];
      let S = [];
      const { nodeLookup: T, triggerNodeChanges: O, connection: C, updateConnection: R, onNodesChangeMiddlewareMap: H } = _();
      for (const [B, X] of w) {
        const Y = T.get(B), F = !!(Y != null && Y.expandParent && (Y != null && Y.parentId) && (X != null && X.position)), K = {
          id: B,
          type: "position",
          position: F ? {
            x: Math.max(0, X.position.x),
            y: Math.max(0, X.position.y)
          } : X.position,
          dragging: E
        };
        if (Y && C.inProgress && C.fromNode.id === Y.id) {
          const D = ca(Y, C.fromHandle, _e.Left, !0);
          R({ ...C, from: D });
        }
        F && Y.parentId && M.push({
          id: B,
          parentId: Y.parentId,
          rect: {
            ...X.internals.positionAbsolute,
            width: X.measured.width ?? 0,
            height: X.measured.height ?? 0
          }
        }), S.push(K);
      }
      if (M.length > 0) {
        const { parentLookup: B, nodeOrigin: X } = _(), Y = dm(M, T, B, X);
        S.push(...Y);
      }
      for (const B of H.values())
        S = B(S);
      O(S);
    },
    triggerNodeChanges: (w) => {
      const { onNodesChange: E, setNodes: M, nodes: S, hasDefaultNodes: T, debug: O } = _();
      if (w != null && w.length) {
        if (T) {
          const C = b8(w, S);
          M(C);
        }
        O && console.log("React Flow: trigger node changes", w), E == null || E(w);
      }
    },
    triggerEdgeChanges: (w) => {
      const { onEdgesChange: E, setEdges: M, edges: S, hasDefaultEdges: T, debug: O } = _();
      if (w != null && w.length) {
        if (T) {
          const C = _8(w, S);
          M(C);
        }
        O && console.log("React Flow: trigger edge changes", w), E == null || E(w);
      }
    },
    addSelectedNodes: (w) => {
      const { multiSelectionActive: E, edgeLookup: M, nodeLookup: S, triggerNodeChanges: T, triggerEdgeChanges: O } = _();
      if (E) {
        const C = w.map((R) => na(R, !0));
        T(C);
        return;
      }
      T(ii(S, /* @__PURE__ */ new Set([...w]), !0)), O(ii(M));
    },
    addSelectedEdges: (w) => {
      const { multiSelectionActive: E, edgeLookup: M, nodeLookup: S, triggerNodeChanges: T, triggerEdgeChanges: O } = _();
      if (E) {
        const C = w.map((R) => na(R, !0));
        O(C);
        return;
      }
      O(ii(M, /* @__PURE__ */ new Set([...w]))), T(ii(S, /* @__PURE__ */ new Set(), !0));
    },
    unselectNodesAndEdges: ({ nodes: w, edges: E } = {}) => {
      const { edges: M, nodes: S, nodeLookup: T, triggerNodeChanges: O, triggerEdgeChanges: C } = _(), R = w || S, H = E || M, B = [];
      for (const Y of R) {
        if (!Y.selected)
          continue;
        const F = T.get(Y.id);
        F && (F.selected = !1), B.push(na(Y.id, !1));
      }
      const X = [];
      for (const Y of H)
        Y.selected && X.push(na(Y.id, !1));
      O(B), C(X);
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
      const O = E.reduce((R, H) => H.selected ? [...R, na(H.id, !1)] : R, []), C = w.reduce((R, H) => H.selected ? [...R, na(H.id, !1)] : R, []);
      M(O), S(C);
    },
    setNodeExtent: (w) => {
      const { nodes: E, nodeLookup: M, parentLookup: S, nodeOrigin: T, elevateNodesOnSelect: O, nodeExtent: C, zIndexMode: R } = _();
      w[0][0] === C[0][0] && w[0][1] === C[0][1] && w[1][0] === C[1][0] && w[1][1] === C[1][1] || (qp(E, M, S, {
        nodeOrigin: T,
        nodeExtent: w,
        elevateNodesOnSelect: O,
        checkEquality: !1,
        zIndexMode: R
      }), b({ nodeExtent: w }));
    },
    panBy: (w) => {
      const { transform: E, width: M, height: S, panZoom: T, translateExtent: O } = _();
      return m6({ delta: w, panZoom: T, transform: E, translateExtent: O, width: M, height: S });
    },
    setCenter: async (w, E, M) => {
      const { width: S, height: T, maxZoom: O, panZoom: C } = _();
      if (!C)
        return !1;
      const R = typeof (M == null ? void 0 : M.zoom) < "u" ? M.zoom : O;
      return await C.setViewport({
        x: S / 2 - w * R,
        y: T / 2 - E * R,
        zoom: R
      }, { duration: M == null ? void 0 : M.duration, ease: M == null ? void 0 : M.ease, interpolate: M == null ? void 0 : M.interpolate }), !0;
    },
    cancelConnection: () => {
      b({
        connection: { ...pM }
      });
    },
    updateConnection: (w) => {
      b({ connection: w });
    },
    reset: () => b({ ...HA() })
  };
}, Object.is);
function Eq({ initialNodes: t, initialEdges: r, defaultNodes: i, defaultEdges: u, initialWidth: o, initialHeight: s, initialMinZoom: c, initialMaxZoom: f, initialFitViewOptions: g, fitView: h, nodeOrigin: v, nodeExtent: p, zIndexMode: m, children: b }) {
  const [_] = re.useState(() => GH({
    nodes: t,
    edges: r,
    defaultNodes: i,
    defaultEdges: u,
    width: o,
    height: s,
    fitView: h,
    minZoom: c,
    maxZoom: f,
    fitViewOptions: g,
    nodeOrigin: v,
    nodeExtent: p,
    zIndexMode: m
  }));
  return J.jsx(W6, { value: _, children: J.jsx(A8, { children: J.jsx(G8, { children: b }) }) });
}
function VH({ children: t, nodes: r, edges: i, defaultNodes: u, defaultEdges: o, width: s, height: c, fitView: f, fitViewOptions: g, minZoom: h, maxZoom: v, nodeOrigin: p, nodeExtent: m, zIndexMode: b }) {
  return re.useContext(ys) ? J.jsx(J.Fragment, { children: t }) : J.jsx(Eq, { initialNodes: r, initialEdges: i, defaultNodes: u, defaultEdges: o, initialWidth: s, initialHeight: c, fitView: f, initialFitViewOptions: g, initialMinZoom: h, initialMaxZoom: v, nodeOrigin: p, nodeExtent: m, zIndexMode: b, children: t });
}
const YH = {
  width: "100%",
  height: "100%",
  overflow: "hidden",
  position: "relative",
  zIndex: 0
};
function kH({ nodes: t, edges: r, defaultNodes: i, defaultEdges: u, className: o, nodeTypes: s, edgeTypes: c, onNodeClick: f, onEdgeClick: g, onInit: h, onMove: v, onMoveStart: p, onMoveEnd: m, onConnect: b, onConnectStart: _, onConnectEnd: A, onClickConnectStart: w, onClickConnectEnd: E, onNodeMouseEnter: M, onNodeMouseMove: S, onNodeMouseLeave: T, onNodeContextMenu: O, onNodeDoubleClick: C, onNodeDragStart: R, onNodeDrag: H, onNodeDragStop: B, onNodesDelete: X, onEdgesDelete: Y, onDelete: F, onSelectionChange: K, onSelectionDragStart: D, onSelectionDrag: G, onSelectionDragStop: N, onSelectionContextMenu: j, onSelectionStart: Z, onSelectionEnd: Q, onBeforeDelete: le, connectionMode: z, connectionLineType: V = zr.Bezier, connectionLineStyle: ie, connectionLineComponent: L, connectionLineContainerStyle: I, deleteKeyCode: P = "Backspace", selectionKeyCode: ae = "Shift", selectionOnDrag: W = !1, selectionMode: se = zu.Full, panActivationKeyCode: de = "Space", multiSelectionKeyCode: ve = Hu() ? "Meta" : "Control", zoomActivationKeyCode: pe = Hu() ? "Meta" : "Control", snapToGrid: he, snapGrid: me, onlyRenderVisibleElements: ge = !1, selectNodesOnDrag: Ae, nodesDraggable: xe, autoPanOnNodeFocus: Pe, nodesConnectable: tt, nodesFocusable: xt, nodeOrigin: gt = $M, edgesFocusable: St, edgesReconnectable: Ze, elementsSelectable: ke = !0, defaultViewport: vt = d8, minZoom: Ct = 0.5, maxZoom: Tt = 2, translateExtent: ft = Ou, preventScrolling: tr = !0, nodeExtent: Cn, defaultMarkerColor: _i = "#b1b1b7", zoomOnScroll: va = !0, zoomOnPinch: Et = !0, panOnScroll: Fe = !1, panOnScrollSpeed: mn = 0.5, panOnScrollMode: Mt = ia.Free, zoomOnDoubleClick: _s = !0, panOnDrag: xs = !0, onPaneClick: Ss, onPaneMouseEnter: ya, onPaneMouseMove: pa, onPaneMouseLeave: ma, onPaneScroll: Rn, onPaneContextMenu: ba, paneClickDistance: Lr = 1, nodeClickDistance: Es = 0, children: $u, onReconnect: xi, onReconnectStart: Br, onReconnectEnd: ws, onEdgeContextMenu: Fu, onEdgeDoubleClick: Ju, onEdgeMouseEnter: Pu, onEdgeMouseMove: Si, onEdgeMouseLeave: Ei, reconnectRadius: Wu = 10, onNodesChange: el, onEdgesChange: bn, noDragClassName: dt = "nodrag", noWheelClassName: wt = "nowheel", noPanClassName: Nn = "nopan", fitView: _a, fitViewOptions: tl, connectOnClick: As, attributionPosition: nl, proOptions: jr, defaultEdgeOptions: wi, elevateNodesOnSelect: nr = !0, elevateEdgesOnSelect: rr = !1, disableKeyboardA11y: ar = !1, autoPanOnConnect: ir, autoPanOnNodeDrag: at, autoPanOnSelection: rl = !0, autoPanSpeed: al, connectionRadius: On, isValidConnection: ur, onError: Ts, style: il, id: Ai, nodeDragThreshold: Ms, connectionDragThreshold: xa, viewport: Sa, onViewportChange: un, width: Rt, height: ul, colorMode: qs = "light", debug: Ti, onScroll: Ur, ariaLabelConfig: Ea, zIndexMode: ll = "basic", ...Nt }, Mi) {
  const wa = Ai || "1", qi = y8(qs), lr = re.useCallback((Ci) => {
    Ci.currentTarget.scrollTo({ top: 0, left: 0, behavior: "instant" }), Ur == null || Ur(Ci);
  }, [Ur]);
  return J.jsx("div", { "data-testid": "rf__wrapper", ...Nt, onScroll: lr, style: { ...il, ...YH }, ref: Mi, className: ct(["react-flow", o, qi]), id: Ai, role: "application", children: J.jsxs(VH, { nodes: t, edges: r, width: Rt, height: ul, fitView: _a, fitViewOptions: tl, minZoom: Ct, maxZoom: Tt, nodeOrigin: gt, nodeExtent: Cn, zIndexMode: ll, children: [J.jsx(v8, { nodes: t, edges: r, defaultNodes: i, defaultEdges: u, onConnect: b, onConnectStart: _, onConnectEnd: A, onClickConnectStart: w, onClickConnectEnd: E, nodesDraggable: xe, autoPanOnNodeFocus: Pe, nodesConnectable: tt, nodesFocusable: xt, edgesFocusable: St, edgesReconnectable: Ze, elementsSelectable: ke, elevateNodesOnSelect: nr, elevateEdgesOnSelect: rr, minZoom: Ct, maxZoom: Tt, nodeExtent: Cn, onNodesChange: el, onEdgesChange: bn, snapToGrid: he, snapGrid: me, connectionMode: z, translateExtent: ft, connectOnClick: As, defaultEdgeOptions: wi, fitView: _a, fitViewOptions: tl, onNodesDelete: X, onEdgesDelete: Y, onDelete: F, onNodeDragStart: R, onNodeDrag: H, onNodeDragStop: B, onSelectionDrag: G, onSelectionDragStart: D, onSelectionDragStop: N, onMove: v, onMoveStart: p, onMoveEnd: m, noPanClassName: Nn, nodeOrigin: gt, rfId: wa, autoPanOnConnect: ir, autoPanOnNodeDrag: at, autoPanSpeed: al, onError: Ts, connectionRadius: On, isValidConnection: ur, selectNodesOnDrag: Ae, nodeDragThreshold: Ms, connectionDragThreshold: xa, onBeforeDelete: le, debug: Ti, ariaLabelConfig: Ea, zIndexMode: ll }), J.jsx(jH, { onInit: h, onNodeClick: f, onEdgeClick: g, onNodeMouseEnter: M, onNodeMouseMove: S, onNodeMouseLeave: T, onNodeContextMenu: O, onNodeDoubleClick: C, nodeTypes: s, edgeTypes: c, connectionLineType: V, connectionLineStyle: ie, connectionLineComponent: L, connectionLineContainerStyle: I, selectionKeyCode: ae, selectionOnDrag: W, selectionMode: se, deleteKeyCode: P, multiSelectionKeyCode: ve, panActivationKeyCode: de, zoomActivationKeyCode: pe, onlyRenderVisibleElements: ge, defaultViewport: vt, translateExtent: ft, minZoom: Ct, maxZoom: Tt, preventScrolling: tr, zoomOnScroll: va, zoomOnPinch: Et, zoomOnDoubleClick: _s, panOnScroll: Fe, panOnScrollSpeed: mn, panOnScrollMode: Mt, panOnDrag: xs, autoPanOnSelection: rl, onPaneClick: Ss, onPaneMouseEnter: ya, onPaneMouseMove: pa, onPaneMouseLeave: ma, onPaneScroll: Rn, onPaneContextMenu: ba, paneClickDistance: Lr, nodeClickDistance: Es, onSelectionContextMenu: j, onSelectionStart: Z, onSelectionEnd: Q, onReconnect: xi, onReconnectStart: Br, onReconnectEnd: ws, onEdgeContextMenu: Fu, onEdgeDoubleClick: Ju, onEdgeMouseEnter: Pu, onEdgeMouseMove: Si, onEdgeMouseLeave: Ei, reconnectRadius: Wu, defaultMarkerColor: _i, noDragClassName: dt, noWheelClassName: wt, noPanClassName: Nn, rfId: wa, disableKeyboardA11y: ar, nodeExtent: Cn, viewport: Sa, onViewportChange: un, nodesDraggable: xe }), J.jsx(f8, { onSelectionChange: K }), $u, J.jsx(u8, { proOptions: jr, position: nl }), J.jsx(i8, { rfId: wa, disableKeyboardA11y: ar })] }) });
}
var XH = JM(kH);
function IH({ dimensions: t, lineWidth: r, variant: i, className: u }) {
  return J.jsx("path", { strokeWidth: r, d: `M${t[0] / 2} 0 V${t[1]} M0 ${t[1] / 2} H${t[0]}`, className: ct(["react-flow__background-pattern", i, u]) });
}
function QH({ radius: t, className: r }) {
  return J.jsx("circle", { cx: t, cy: t, r: t, className: ct(["react-flow__background-pattern", "dots", r]) });
}
var Dr;
(function(t) {
  t.Lines = "lines", t.Dots = "dots", t.Cross = "cross";
})(Dr || (Dr = {}));
const ZH = {
  [Dr.Dots]: 1,
  [Dr.Lines]: 1,
  [Dr.Cross]: 6
}, KH = (t) => ({ transform: t.transform, patternId: `pattern-${t.rfId}` });
function wq({
  id: t,
  variant: r = Dr.Dots,
  // only used for dots and cross
  gap: i = 20,
  // only used for lines and cross
  size: u,
  lineWidth: o = 1,
  offset: s = 0,
  color: c,
  bgColor: f,
  style: g,
  className: h,
  patternClassName: v
}) {
  const p = re.useRef(null), { transform: m, patternId: b } = ze(KH, Je), _ = u || ZH[r], A = r === Dr.Dots, w = r === Dr.Cross, E = Array.isArray(i) ? i : [i, i], M = [E[0] * m[2] || 1, E[1] * m[2] || 1], S = _ * m[2], T = Array.isArray(s) ? s : [s, s], O = w ? [S, S] : M, C = [
    T[0] * m[2] || 1 + O[0] / 2,
    T[1] * m[2] || 1 + O[1] / 2
  ], R = `${b}${t || ""}`;
  return J.jsxs("svg", { className: ct(["react-flow__background", h]), style: {
    ...g,
    ...ms,
    "--xy-background-color-props": f,
    "--xy-background-pattern-color-props": c
  }, ref: p, "data-testid": "rf__background", children: [J.jsx("pattern", { id: R, x: m[0] % M[0], y: m[1] % M[1], width: M[0], height: M[1], patternUnits: "userSpaceOnUse", patternTransform: `translate(-${C[0]},-${C[1]})`, children: A ? J.jsx(QH, { radius: S / 2, className: v }) : J.jsx(IH, { dimensions: O, lineWidth: o, variant: r, className: v }) }), J.jsx("rect", { x: "0", y: "0", width: "100%", height: "100%", fill: `url(#${R})` })] });
}
wq.displayName = "Background";
const $H = re.memo(wq);
function FH() {
  return J.jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 32 32", children: J.jsx("path", { d: "M32 18.133H18.133V32h-4.266V18.133H0v-4.266h13.867V0h4.266v13.867H32z" }) });
}
function JH() {
  return J.jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 32 5", children: J.jsx("path", { d: "M0 0h32v4.2H0z" }) });
}
function PH() {
  return J.jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 32 30", children: J.jsx("path", { d: "M3.692 4.63c0-.53.4-.938.939-.938h5.215V0H4.708C2.13 0 0 2.054 0 4.63v5.216h3.692V4.631zM27.354 0h-5.2v3.692h5.17c.53 0 .984.4.984.939v5.215H32V4.631A4.624 4.624 0 0027.354 0zm.954 24.83c0 .532-.4.94-.939.94h-5.215v3.768h5.215c2.577 0 4.631-2.13 4.631-4.707v-5.139h-3.692v5.139zm-23.677.94c-.531 0-.939-.4-.939-.94v-5.138H0v5.139c0 2.577 2.13 4.707 4.708 4.707h5.138V25.77H4.631z" }) });
}
function WH() {
  return J.jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 25 32", children: J.jsx("path", { d: "M21.333 10.667H19.81V7.619C19.81 3.429 16.38 0 12.19 0 8 0 4.571 3.429 4.571 7.619v3.048H3.048A3.056 3.056 0 000 13.714v15.238A3.056 3.056 0 003.048 32h18.285a3.056 3.056 0 003.048-3.048V13.714a3.056 3.056 0 00-3.048-3.047zM12.19 24.533a3.056 3.056 0 01-3.047-3.047 3.056 3.056 0 013.047-3.048 3.056 3.056 0 013.048 3.048 3.056 3.056 0 01-3.048 3.047zm4.724-13.866H7.467V7.619c0-2.59 2.133-4.724 4.723-4.724 2.591 0 4.724 2.133 4.724 4.724v3.048z" }) });
}
function eL() {
  return J.jsx("svg", { xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 25 32", children: J.jsx("path", { d: "M21.333 10.667H19.81V7.619C19.81 3.429 16.38 0 12.19 0c-4.114 1.828-1.37 2.133.305 2.438 1.676.305 4.42 2.59 4.42 5.181v3.048H3.047A3.056 3.056 0 000 13.714v15.238A3.056 3.056 0 003.048 32h18.285a3.056 3.056 0 003.048-3.048V13.714a3.056 3.056 0 00-3.048-3.047zM12.19 24.533a3.056 3.056 0 01-3.047-3.047 3.056 3.056 0 013.047-3.048 3.056 3.056 0 013.048 3.048 3.056 3.056 0 01-3.048 3.047z" }) });
}
function Mo({ children: t, className: r, ...i }) {
  return J.jsx("button", { type: "button", className: ct(["react-flow__controls-button", r]), ...i, children: t });
}
const tL = (t) => ({
  isInteractive: t.nodesDraggable || t.nodesConnectable || t.elementsSelectable,
  minZoomReached: t.transform[2] <= t.minZoom,
  maxZoomReached: t.transform[2] >= t.maxZoom,
  ariaLabelConfig: t.ariaLabelConfig
});
function Aq({ style: t, showZoom: r = !0, showFitView: i = !0, showInteractive: u = !0, fitViewOptions: o, onZoomIn: s, onZoomOut: c, onFitView: f, onInteractiveChange: g, className: h, children: v, position: p = "bottom-left", orientation: m = "vertical", "aria-label": b }) {
  const _ = Qe(), { isInteractive: A, minZoomReached: w, maxZoomReached: E, ariaLabelConfig: M } = ze(tL, Je), { zoomIn: S, zoomOut: T, fitView: O } = hm(), C = () => {
    S(), s == null || s();
  }, R = () => {
    T(), c == null || c();
  }, H = () => {
    O(o), f == null || f();
  }, B = () => {
    _.setState({
      nodesDraggable: !A,
      nodesConnectable: !A,
      elementsSelectable: !A
    }), g == null || g(!A);
  }, X = m === "horizontal" ? "horizontal" : "vertical";
  return J.jsxs(ps, { className: ct(["react-flow__controls", X, h]), position: p, style: t, "data-testid": "rf__controls", "aria-label": b ?? M["controls.ariaLabel"], children: [r && J.jsxs(J.Fragment, { children: [J.jsx(Mo, { onClick: C, className: "react-flow__controls-zoomin", title: M["controls.zoomIn.ariaLabel"], "aria-label": M["controls.zoomIn.ariaLabel"], disabled: E, children: J.jsx(FH, {}) }), J.jsx(Mo, { onClick: R, className: "react-flow__controls-zoomout", title: M["controls.zoomOut.ariaLabel"], "aria-label": M["controls.zoomOut.ariaLabel"], disabled: w, children: J.jsx(JH, {}) })] }), i && J.jsx(Mo, { className: "react-flow__controls-fitview", onClick: H, title: M["controls.fitView.ariaLabel"], "aria-label": M["controls.fitView.ariaLabel"], children: J.jsx(PH, {}) }), u && J.jsx(Mo, { className: "react-flow__controls-interactive", onClick: B, title: M["controls.interactive.ariaLabel"], "aria-label": M["controls.interactive.ariaLabel"], children: A ? J.jsx(eL, {}) : J.jsx(WH, {}) }), v] });
}
Aq.displayName = "Controls";
re.memo(Aq);
function nL({ id: t, x: r, y: i, width: u, height: o, style: s, color: c, strokeColor: f, strokeWidth: g, className: h, borderRadius: v, shapeRendering: p, selected: m, onClick: b }) {
  const { background: _, backgroundColor: A } = s || {}, w = c || _ || A;
  return J.jsx("rect", { className: ct(["react-flow__minimap-node", { selected: m }, h]), x: r, y: i, rx: v, ry: v, width: u, height: o, style: {
    fill: w,
    stroke: f,
    strokeWidth: g
  }, shapeRendering: p, onClick: b ? (E) => b(E, t) : void 0 });
}
const rL = re.memo(nL), aL = (t) => t.nodes.map((r) => r.id), yp = (t) => t instanceof Function ? t : () => t;
function iL({
  nodeStrokeColor: t,
  nodeColor: r,
  nodeClassName: i = "",
  nodeBorderRadius: u = 5,
  nodeStrokeWidth: o,
  /*
   * We need to rename the prop to be `CapitalCase` so that JSX will render it as
   * a component properly.
   */
  nodeComponent: s = rL,
  onClick: c
}) {
  const f = ze(aL, Je), g = yp(r), h = yp(t), v = yp(i), p = typeof window > "u" || window.chrome ? "crispEdges" : "geometricPrecision";
  return J.jsx(J.Fragment, { children: f.map((m) => (
    /*
     * The split of responsibilities between MiniMapNodes and
     * NodeComponentWrapper may appear weird. However, it’s designed to
     * minimize the cost of updates when individual nodes change.
     *
     * For more details, see a similar commit in `NodeRenderer/index.tsx`.
     */
    J.jsx(lL, { id: m, nodeColorFunc: g, nodeStrokeColorFunc: h, nodeClassNameFunc: v, nodeBorderRadius: u, nodeStrokeWidth: o, NodeComponent: s, onClick: c, shapeRendering: p }, m)
  )) });
}
function uL({ id: t, nodeColorFunc: r, nodeStrokeColorFunc: i, nodeClassNameFunc: u, nodeBorderRadius: o, nodeStrokeWidth: s, shapeRendering: c, NodeComponent: f, onClick: g }) {
  const { node: h, x: v, y: p, width: m, height: b } = ze((_) => {
    const A = _.nodeLookup.get(t);
    if (!A)
      return { node: void 0, x: 0, y: 0, width: 0, height: 0 };
    const w = A.internals.userNode, { x: E, y: M } = A.internals.positionAbsolute, { width: S, height: T } = qn(w);
    return {
      node: w,
      x: E,
      y: M,
      width: S,
      height: T
    };
  }, Je);
  return !h || h.hidden || !AM(h) ? null : J.jsx(f, { x: v, y: p, width: m, height: b, style: h.style, selected: !!h.selected, className: u(h), color: r(h), borderRadius: o, strokeColor: i(h), strokeWidth: s, shapeRendering: c, onClick: g, id: h.id });
}
const lL = re.memo(uL);
var oL = re.memo(iL);
const sL = 200, cL = 150, fL = (t) => !t.hidden, dL = (t) => {
  const r = {
    x: -t.transform[0] / t.transform[2],
    y: -t.transform[1] / t.transform[2],
    width: t.width / t.transform[2],
    height: t.height / t.transform[2]
  };
  return {
    viewBB: r,
    boundingRect: t.nodeLookup.size > 0 ? SM(Qu(t.nodeLookup, { filter: fL }), r) : r,
    rfId: t.rfId,
    panZoom: t.panZoom,
    translateExtent: t.translateExtent,
    flowWidth: t.width,
    flowHeight: t.height,
    ariaLabelConfig: t.ariaLabelConfig
  };
}, LA = (t, r) => t.x === r.x && t.y === r.y && t.width === r.width && t.height === r.height, hL = (t, r) => LA(t.viewBB, r.viewBB) && LA(t.boundingRect, r.boundingRect) && t.rfId === r.rfId && t.panZoom === r.panZoom && t.translateExtent === r.translateExtent && t.flowWidth === r.flowWidth && t.flowHeight === r.flowHeight && t.ariaLabelConfig === r.ariaLabelConfig, gL = "react-flow__minimap-desc";
function Tq({
  style: t,
  className: r,
  nodeStrokeColor: i,
  nodeColor: u,
  nodeClassName: o = "",
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
  maskStrokeWidth: p,
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
  const O = Qe(), C = re.useRef(null), { boundingRect: R, viewBB: H, rfId: B, panZoom: X, translateExtent: Y, flowWidth: F, flowHeight: K, ariaLabelConfig: D } = ze(dL, hL), G = (t == null ? void 0 : t.width) ?? sL, N = (t == null ? void 0 : t.height) ?? cL, j = R.width / G, Z = R.height / N, Q = Math.max(j, Z), le = Q * G, z = Q * N, V = T * Q, ie = R.x - (le - R.width) / 2 - V, L = R.y - (z - R.height) / 2 - V, I = le + V * 2, P = z + V * 2, ae = `${gL}-${B}`, W = re.useRef(0), se = re.useRef();
  W.current = Q, re.useEffect(() => {
    if (C.current && X)
      return se.current = M6({
        domNode: C.current,
        panZoom: X,
        getTransform: () => O.getState().transform,
        getViewScale: () => W.current
      }), () => {
        var he;
        (he = se.current) == null || he.destroy();
      };
  }, [X]), re.useEffect(() => {
    var he;
    (he = se.current) == null || he.update({
      translateExtent: Y,
      width: F,
      height: K,
      inversePan: M,
      pannable: A,
      zoomStep: S,
      zoomable: w
    });
  }, [A, w, M, S, Y, F, K]);
  const de = b ? (he) => {
    var Ae;
    const [me, ge] = ((Ae = se.current) == null ? void 0 : Ae.pointer(he)) || [0, 0];
    b(he, { x: me, y: ge });
  } : void 0, ve = _ ? re.useCallback((he, me) => {
    const ge = O.getState().nodeLookup.get(me).internals.userNode;
    _(he, ge);
  }, []) : void 0, pe = E ?? D["minimap.ariaLabel"];
  return J.jsx(ps, { position: m, style: {
    ...t,
    "--xy-minimap-background-color-props": typeof g == "string" ? g : void 0,
    "--xy-minimap-mask-background-color-props": typeof h == "string" ? h : void 0,
    "--xy-minimap-mask-stroke-color-props": typeof v == "string" ? v : void 0,
    "--xy-minimap-mask-stroke-width-props": typeof p == "number" ? p * Q : void 0,
    "--xy-minimap-node-background-color-props": typeof u == "string" ? u : void 0,
    "--xy-minimap-node-stroke-color-props": typeof i == "string" ? i : void 0,
    "--xy-minimap-node-stroke-width-props": typeof c == "number" ? c : void 0
  }, className: ct(["react-flow__minimap", r]), "data-testid": "rf__minimap", children: J.jsxs("svg", { width: G, height: N, viewBox: `${ie} ${L} ${I} ${P}`, className: "react-flow__minimap-svg", role: "img", "aria-labelledby": ae, ref: C, onClick: de, children: [pe && J.jsx("title", { id: ae, children: pe }), J.jsx(oL, { onClick: ve, nodeColor: u, nodeStrokeColor: i, nodeBorderRadius: s, nodeClassName: o, nodeStrokeWidth: c, nodeComponent: f }), J.jsx("path", { className: "react-flow__minimap-mask", d: `M${ie - V},${L - V}h${I + V * 2}v${P + V * 2}h${-I - V * 2}z
        M${H.x},${H.y}h${H.width}v${H.height}h${-H.width}z`, fillRule: "evenodd", pointerEvents: "none" })] }) });
}
Tq.displayName = "MiniMap";
re.memo(Tq);
const vL = (t) => (r) => t ? `${Math.max(1 / r.transform[2], 1)}` : void 0, yL = {
  [hi.Line]: "right",
  [hi.Handle]: "bottom-right"
};
function pL({ nodeId: t, position: r, variant: i = hi.Handle, className: u, style: o = void 0, children: s, color: c, minWidth: f = 10, minHeight: g = 10, maxWidth: h = Number.MAX_VALUE, maxHeight: v = Number.MAX_VALUE, keepAspectRatio: p = !1, resizeDirection: m, autoScale: b = !0, shouldResize: _, onResizeStart: A, onResize: w, onResizeEnd: E }) {
  const M = nq(), S = typeof t == "string" ? t : M, T = Qe(), O = re.useRef(null), C = i === hi.Handle, R = ze(re.useCallback(vL(C && b), [C, b]), Je), H = re.useRef(null), B = r ?? yL[i];
  re.useEffect(() => {
    if (!(!O.current || !S))
      return H.current || (H.current = G6({
        domNode: O.current,
        nodeId: S,
        getStoreItems: () => {
          const { nodeLookup: Y, transform: F, snapGrid: K, snapToGrid: D, nodeOrigin: G, domNode: N } = T.getState();
          return {
            nodeLookup: Y,
            transform: F,
            snapGrid: K,
            snapToGrid: D,
            nodeOrigin: G,
            paneDomNode: N
          };
        },
        onChange: (Y, F) => {
          const { triggerNodeChanges: K, nodeLookup: D, parentLookup: G, nodeOrigin: N } = T.getState(), j = [], Z = { x: Y.x, y: Y.y }, Q = D.get(S);
          if (Q && Q.expandParent && Q.parentId) {
            const le = Q.origin ?? N, z = Y.width ?? Q.measured.width ?? 0, V = Y.height ?? Q.measured.height ?? 0, ie = {
              id: Q.id,
              parentId: Q.parentId,
              rect: {
                width: z,
                height: V,
                ...TM({
                  x: Y.x ?? Q.position.x,
                  y: Y.y ?? Q.position.y
                }, { width: z, height: V }, Q.parentId, D, le)
              }
            }, L = dm([ie], D, G, N);
            j.push(...L), Z.x = Y.x ? Math.max(le[0] * z, Y.x) : void 0, Z.y = Y.y ? Math.max(le[1] * V, Y.y) : void 0;
          }
          if (Z.x !== void 0 && Z.y !== void 0) {
            const le = {
              id: S,
              type: "position",
              position: { ...Z }
            };
            j.push(le);
          }
          if (Y.width !== void 0 && Y.height !== void 0) {
            const z = {
              id: S,
              type: "dimensions",
              resizing: !0,
              setAttributes: m ? m === "horizontal" ? "width" : "height" : !0,
              dimensions: {
                width: Y.width,
                height: Y.height
              }
            };
            j.push(z);
          }
          for (const le of F) {
            const z = {
              ...le,
              type: "position"
            };
            j.push(z);
          }
          K(j);
        },
        onEnd: ({ width: Y, height: F }) => {
          const K = {
            id: S,
            type: "dimensions",
            resizing: !1,
            dimensions: {
              width: Y,
              height: F
            }
          };
          T.getState().triggerNodeChanges([K]);
        }
      })), H.current.update({
        controlPosition: B,
        boundaries: {
          minWidth: f,
          minHeight: g,
          maxWidth: h,
          maxHeight: v
        },
        keepAspectRatio: p,
        resizeDirection: m,
        onResizeStart: A,
        onResize: w,
        onResizeEnd: E,
        shouldResize: _
      }), () => {
        var Y;
        (Y = H.current) == null || Y.destroy();
      };
  }, [
    B,
    f,
    g,
    h,
    v,
    p,
    A,
    w,
    E,
    _
  ]);
  const X = B.split("-");
  return J.jsx("div", { className: ct(["react-flow__resize-control", "nodrag", ...X, i, u]), ref: O, style: {
    ...o,
    scale: R,
    ...c && { [C ? "backgroundColor" : "borderColor"]: c }
  }, children: s });
}
re.memo(pL);
function mL({ flat: t, selected: r, color: i, state: u }) {
  const o = [t.agentType, t.model, t.badge].filter(Boolean), s = [
    "nwf-node",
    `nwf-kind-${t.kind}`,
    r ? "nwf-selected" : "",
    u ? `nwf-${u}` : ""
  ].filter(Boolean).join(" ");
  return /* @__PURE__ */ J.jsxs(
    "div",
    {
      className: s,
      style: i ? { "--nwf-phase-color": i } : void 0,
      title: t.label,
      children: [
        t.phase ? /* @__PURE__ */ J.jsx("div", { className: "nwf-node-phase", children: t.phase }) : null,
        /* @__PURE__ */ J.jsx("div", { className: "nwf-node-label", children: t.label }),
        o.length > 0 ? /* @__PURE__ */ J.jsx("div", { className: "nwf-node-badges", children: o.map((c) => /* @__PURE__ */ J.jsx("span", { className: "nwf-node-badge", children: c }, c)) }) : null
      ]
    }
  );
}
var bL = {
  nodeWidth: 230,
  nodeHeight: 84,
  direction: "LR",
  nodeSep: 28,
  rankSep: 110
};
function _L(t, r, i = {}) {
  const { nodeWidth: u, nodeHeight: o, direction: s, nodeSep: c, rankSep: f } = { ...bL, ...i }, g = {};
  if (t.length === 0) return g;
  const h = new M2.graphlib.Graph();
  h.setGraph({ rankdir: s, nodesep: c, ranksep: f, marginx: 24, marginy: 24 }), h.setDefaultEdgeLabel(() => ({}));
  for (const v of t) h.setNode(v.id, { width: u, height: o });
  for (const v of r)
    v.source !== v.target && h.hasNode(v.source) && h.hasNode(v.target) && h.setEdge(v.source, v.target);
  M2.layout(h);
  for (const v of t) {
    const p = h.node(v.id);
    g[v.id] = p ? { x: p.x - u / 2, y: p.y - o / 2 } : { x: 0, y: 0 };
  }
  return g;
}
var xL = [
  "#6ea8fe",
  "#5fd0a8",
  "#f7a072",
  "#c98bdb",
  "#e8c468",
  "#7ec9e8",
  "#e87e9e",
  "#9ee87e"
];
function SL(t) {
  const { flat: r, color: i, state: u, renderNode: o, vertical: s } = t.data, c = t.selected ?? !1;
  return /* @__PURE__ */ J.jsxs(J.Fragment, { children: [
    /* @__PURE__ */ J.jsx(gi, { type: "target", position: s ? _e.Top : _e.Left }),
    o ? o(r, { selected: c }) : /* @__PURE__ */ J.jsx(mL, { flat: r, selected: c, color: i, state: u }),
    /* @__PURE__ */ J.jsx(gi, { type: "source", position: s ? _e.Bottom : _e.Right })
  ] });
}
var EL = (t) => t === "loop" ? { strokeDasharray: "4 4" } : void 0;
function wL({
  nodes: t,
  edges: r,
  selectedId: i,
  onNodeClick: u,
  onNodeDoubleClick: o,
  renderNode: s,
  theme: c = "dark",
  phaseColors: f = xL,
  edgeStyle: g,
  nodeState: h,
  layoutOptions: v,
  verticalRender: p,
  fitView: m = !0,
  className: b,
  style: _,
  children: A
}) {
  const [w, E] = re.useState(null), M = i !== void 0, S = M ? i : w, T = (v == null ? void 0 : v.direction) ?? (p ? "TB" : "LR"), O = T === "TB" || T === "BT", C = re.useMemo(
    () => ({ ...v, direction: T }),
    [v, T]
  ), R = re.useMemo(() => ({ nwf: SL }), []), H = re.useMemo(
    () => _L(t, r, C),
    [t, r, C]
  ), B = re.useMemo(() => new Map(t.map((G) => [G.id, G])), [t]), X = re.useMemo(
    () => t.map((G) => {
      const N = G.phaseIndex == null ? void 0 : f[G.phaseIndex % f.length];
      return {
        id: G.id,
        type: "nwf",
        position: H[G.id] ?? { x: 0, y: 0 },
        selected: G.id === S,
        data: { flat: G, color: N, state: h == null ? void 0 : h[G.id], renderNode: s, vertical: O }
      };
    }),
    [t, H, S, s, f, h, O]
  ), Y = re.useMemo(
    () => r.map((G) => ({
      id: G.id,
      source: G.source,
      target: G.target,
      type: "smoothstep",
      animated: G.kind === "loop",
      style: (g == null ? void 0 : g(G)) ?? EL(G.kind)
    })),
    [r, g]
  ), F = re.useCallback(
    (G, N) => {
      const j = B.get(N.id);
      j && (M || E(N.id), u == null || u(j, G));
    },
    [B, M, u]
  ), K = re.useCallback(
    (G, N) => {
      const j = B.get(N.id);
      j && (o == null || o(j, G));
    },
    [B, o]
  ), D = [
    "nwf-graph",
    c === "dark" ? "nwf-theme-dark" : "",
    O ? "nwf-vertical" : "",
    b ?? ""
  ].filter(Boolean).join(" ");
  return /* @__PURE__ */ J.jsx("div", { className: D, style: { width: "100%", height: "100%", ..._ }, children: /* @__PURE__ */ J.jsx(
    XH,
    {
      nodes: X,
      edges: Y,
      nodeTypes: R,
      onNodeClick: F,
      onNodeDoubleClick: K,
      fitView: m,
      proOptions: { hideAttribution: !0 },
      children: A ?? /* @__PURE__ */ J.jsx($H, {})
    }
  ) });
}
function AL(t) {
  return /* @__PURE__ */ J.jsx(Eq, { children: /* @__PURE__ */ J.jsx(wL, { ...t }) });
}
const TL = [
  "--color-cornflower",
  "--color-teal",
  "--color-salmon",
  "--color-coral",
  "--color-green-l2"
];
function BA() {
  return typeof document > "u" ? "light" : document.documentElement.getAttribute("data-theme") === "dark" ? "dark" : "light";
}
function Mq() {
  const [t, r] = re.useState(BA);
  return re.useEffect(() => {
    const i = new MutationObserver(() => r(BA()));
    return i.observe(document.documentElement, {
      attributes: !0,
      attributeFilter: ["data-theme"]
    }), () => i.disconnect();
  }, []), t;
}
function ML() {
  const t = Mq();
  return re.useMemo(() => {
    if (typeof document > "u") return [];
    const r = getComputedStyle(document.documentElement);
    return TL.map((i) => r.getPropertyValue(i).trim()).filter(Boolean);
  }, [t]);
}
const qL = [
  "task",
  "classifier",
  "rag",
  "delegator",
  "mcp",
  "multi_model",
  "multi_prompt",
  "matrixed",
  "pipeline"
];
function qq(t) {
  return t.type ?? "task";
}
function CL(t) {
  var r;
  return ((r = t.display_name) == null ? void 0 : r.trim()) || t.name;
}
function RL(t) {
  const r = new Map(t.map((s) => [s.name, s])), i = [], u = /* @__PURE__ */ new Set();
  let o = t[0];
  for (; o && !u.has(o.name); )
    u.add(o.name), i.push(o), o = o.next_step ? r.get(o.next_step) : void 0;
  for (const s of t)
    u.has(s.name) || (u.add(s.name), i.push(s));
  return i;
}
function NL(t) {
  const r = t.review_type;
  return r && r !== "none" ? `${r} review` : void 0;
}
function OL(t) {
  var s, c, f, g, h, v, p, m;
  const r = [], i = [], u = qq(t), o = (b, _, A) => {
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
      if (!b.length) break;
      const _ = ((c = t.multi_model_config) == null ? void 0 : c.voting_strategy) ?? "vote";
      return { nodes: r, edges: i, exit: o(b, "parallel", `vote: ${_}`) };
    }
    case "multi_prompt": {
      const b = ((f = t.multi_prompt_config) == null ? void 0 : f.prompt_variations) ?? [];
      if (!b.length) break;
      const _ = ((g = t.multi_prompt_config) == null ? void 0 : g.selection_strategy) ?? "select", A = b.map((w, E) => `variation ${E + 1}`);
      return { nodes: r, edges: i, exit: o(A, "parallel", `select: ${_}`) };
    }
    case "matrixed": {
      const b = ((h = t.matrixed_config) == null ? void 0 : h.delegators) ?? [], _ = ((v = t.matrixed_config) == null ? void 0 : v.prompt_variations) ?? [];
      if (!b.length || !_.length) break;
      const A = b.flatMap(
        (w) => _.map((E, M) => `${w} · variation ${M + 1}`)
      );
      return { nodes: r, edges: i, exit: o(A, "parallel", "matrix aggregate") };
    }
    case "pipeline": {
      const b = ((p = t.pipeline_config) == null ? void 0 : p.stages) ?? [];
      if (!b.length) break;
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
function zL(t) {
  var c, f;
  const r = RL(t.steps ?? []), i = [], u = [], o = /* @__PURE__ */ new Map();
  for (const g of r) {
    const h = qq(g);
    i.push({
      id: g.name,
      kind: qL.includes(h) ? "agent" : "note",
      label: CL(g),
      phase: g.name,
      prompt: g.prompt ?? void 0,
      agentType: g.agent ?? ((c = g.delegator_config) == null ? void 0 : c.delegator) ?? void 0,
      badge: NL(g),
      source: h
    });
    const v = OL(g);
    i.push(...v.nodes), u.push(...v.edges), o.set(g.name, v.exit);
  }
  for (let g = 0; g < r.length - 1; g += 1) {
    const h = o.get(r[g].name) ?? r[g].name, v = r[g + 1].name;
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
function DL({
  issueType: t,
  height: r = 520,
  vertical: i = !1,
  className: u
}) {
  const o = Mq(), s = ML(), { nodes: c, edges: f } = re.useMemo(() => zL(t), [t]);
  return c.length ? /* @__PURE__ */ J.jsx("div", { className: u ?? "operator-workflow-canvas", style: { height: r }, children: /* @__PURE__ */ J.jsx(
    AL,
    {
      nodes: c,
      edges: f,
      theme: o,
      phaseColors: s,
      verticalRender: i,
      fitView: !0
    },
    t.key
  ) }) : /* @__PURE__ */ J.jsx("div", { className: "operator-workflow-empty", children: "This issue type defines no steps." });
}
async function jA(t) {
  const r = await fetch(t);
  if (!r.ok) throw new Error(`${r.status} ${r.statusText} for ${t}`);
  return await r.json();
}
function HL({ base: t, initial: r }) {
  const [i, u] = re.useState(null), [o, s] = re.useState(r ?? null), [c, f] = re.useState(null), [g, h] = re.useState(null);
  return re.useEffect(() => {
    let v = !1;
    return jA(`${t}collection.json`).then((p) => {
      if (v) return;
      const m = p.issue_types ?? [];
      u(m), s((b) => {
        var _;
        return b ?? ((_ = m[0]) == null ? void 0 : _.key) ?? null;
      });
    }).catch((p) => !v && h(p.message)), () => {
      v = !0;
    };
  }, [t]), re.useEffect(() => {
    if (!i || !o) return;
    const v = i.find((m) => m.key === o);
    if (!v) return;
    let p = !1;
    return f(null), jA(`${t}${v.schema_path}`).then((m) => !p && f(m)).catch((m) => !p && h(m.message)), () => {
      p = !0;
    };
  }, [t, i, o]), g ? /* @__PURE__ */ J.jsxs("div", { className: "workflow-explorer-error", children: [
    "Could not load this collection: ",
    g
  ] }) : i ? /* @__PURE__ */ J.jsxs("div", { className: "workflow-explorer-body", children: [
    /* @__PURE__ */ J.jsx("nav", { className: "workflow-explorer-rail", "aria-label": "Issue types", children: /* @__PURE__ */ J.jsx("ul", { children: i.map((v) => /* @__PURE__ */ J.jsx("li", { children: /* @__PURE__ */ J.jsx(
      "button",
      {
        type: "button",
        "aria-current": v.key === o,
        className: v.key === o ? "is-selected" : void 0,
        onClick: () => s(v.key),
        children: v.key
      }
    ) }, v.key)) }) }),
    /* @__PURE__ */ J.jsx("div", { className: "workflow-explorer-canvas", children: c ? /* @__PURE__ */ J.jsxs(J.Fragment, { children: [
      /* @__PURE__ */ J.jsxs("h3", { className: "workflow-explorer-title", children: [
        c.name,
        " ",
        /* @__PURE__ */ J.jsx("code", { children: c.key })
      ] }),
      c.description && /* @__PURE__ */ J.jsx("p", { className: "workflow-explorer-description", children: c.description }),
      /* @__PURE__ */ J.jsx(DL, { issueType: c, vertical: !0 })
    ] }) : /* @__PURE__ */ J.jsx("div", { className: "workflow-explorer-loading", children: "Loading workflow…" }) })
  ] }) : /* @__PURE__ */ J.jsx("div", { className: "workflow-explorer-loading", children: "Loading collection…" });
}
class LL extends HTMLElement {
  constructor() {
    super(...arguments);
    Df(this, "root");
  }
  connectedCallback() {
    if (this.root) return;
    const i = this.getAttribute("base");
    if (!i) {
      this.textContent = 'operator-workflow-explorer: missing required "base" attribute.';
      return;
    }
    this.root = KR.createRoot(this), this.root.render(
      /* @__PURE__ */ J.jsx(re.StrictMode, { children: /* @__PURE__ */ J.jsx(HL, { base: i.endsWith("/") ? i : `${i}/`, initial: this.getAttribute("selected") }) })
    );
  }
  disconnectedCallback() {
    const i = this.root;
    this.root = void 0, queueMicrotask(() => i == null ? void 0 : i.unmount());
  }
}
const BL = "operator-workflow-explorer";
function Cq(t, r) {
  customElements.get(t) || customElements.define(t, r);
}
Cq(BL, LL);
Cq(jR, BR);
export {
  BR as OperatorCollectionSearch,
  LL as OperatorWorkflowExplorer
};
