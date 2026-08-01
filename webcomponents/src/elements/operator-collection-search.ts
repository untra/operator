/**
 * `<operator-collection-search>` — inline filter + view toggle for the hosted
 * collection catalog.
 *
 * Progressive enhancement only: the cards and the table are rendered
 * statically by the docs generator, each carrying a `data-search` haystack.
 * This element adds an input that hides non-matching rows and a control that
 * swaps card/table view. With JavaScript disabled the full catalog still
 * renders — nothing here fetches or draws content.
 *
 * Usage:
 *   <operator-collection-search for="collection-catalog"></operator-collection-search>
 *   <div id="collection-catalog" data-view="cards">
 *     <div class="collection-grid">     <article data-search="ralph loop …">…</article> </div>
 *     <table class="collection-table">  <tr data-search="ralph loop …">…</tr>        </table>
 *   </div>
 */

const MATCH_ATTR = 'data-search';

export class OperatorCollectionSearch extends HTMLElement {
  private input?: HTMLInputElement;

  connectedCallback() {
    if (this.dataset.enhanced === 'true') return;
    this.dataset.enhanced = 'true';
    this.render();
  }

  private get catalog(): HTMLElement | null {
    const id = this.getAttribute('for');
    return id ? document.getElementById(id) : null;
  }

  private render() {
    const label = document.createElement('label');
    label.className = 'collection-search';

    const input = document.createElement('input');
    input.type = 'search';
    input.placeholder =
      this.getAttribute('placeholder') ?? 'Filter by name, author, tag, loop shape, issue type…';
    input.setAttribute('aria-label', 'Filter collections');
    input.addEventListener('input', () => this.applyFilter(input.value));
    this.input = input;

    const toggle = document.createElement('button');
    toggle.type = 'button';
    toggle.className = 'collection-view-toggle';
    toggle.addEventListener('click', () => this.toggleView(toggle));

    const count = document.createElement('span');
    count.className = 'collection-search-count';
    count.setAttribute('role', 'status');

    label.append(input, toggle, count);
    this.append(label);

    this.syncToggleLabel(toggle);
    this.applyFilter('');
  }

  private currentView(): 'cards' | 'table' {
    return this.catalog?.dataset.view === 'table' ? 'table' : 'cards';
  }

  private syncToggleLabel(toggle: HTMLButtonElement) {
    const next = this.currentView() === 'cards' ? 'table' : 'cards';
    toggle.textContent = `View as ${next}`;
    toggle.setAttribute('aria-label', `Switch to ${next} view`);
  }

  private toggleView(toggle: HTMLButtonElement) {
    const catalog = this.catalog;
    if (!catalog) return;
    catalog.dataset.view = this.currentView() === 'cards' ? 'table' : 'cards';
    this.syncToggleLabel(toggle);
    this.applyFilter(this.input?.value ?? '');
  }

  private applyFilter(query: string) {
    const catalog = this.catalog;
    if (!catalog) return;

    const terms = query.toLowerCase().split(/\s+/).filter(Boolean);
    // Filter every copy so both views stay in sync, but count only the copies
    // the active view shows — each collection is rendered once per view.
    const active = catalog.querySelector<HTMLElement>(
      `[data-view-target="${this.currentView()}"]`
    );
    let visible = 0;
    let total = 0;

    for (const entry of catalog.querySelectorAll<HTMLElement>(`[${MATCH_ATTR}]`)) {
      const haystack = (entry.getAttribute(MATCH_ATTR) ?? '').toLowerCase();
      const matches = terms.every((term) => haystack.includes(term));
      entry.hidden = !matches;
      if (active?.contains(entry)) {
        total += 1;
        if (matches) visible += 1;
      }
    }

    const count = this.querySelector('.collection-search-count');
    if (count) {
      count.textContent =
        visible === total ? `${total} collections` : `${visible} of ${total} collections`;
    }
    catalog.dataset.empty = visible === 0 ? 'true' : 'false';
  }
}

export const OPERATOR_COLLECTION_SEARCH_TAG = 'operator-collection-search';
