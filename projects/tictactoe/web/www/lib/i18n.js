// Message loading for the page. Pure functions, so Node can test them without a DOM.

// Matches a named placeholder such as {row}. Only word characters are allowed in the name.
const PLACEHOLDER = /\{(\w+)\}/g;

/**
 * Fetches the message catalogue (a JSON object of key to text).
 * `fetchImpl` is passed in so tests can supply a fake.
 */
export async function loadMessages(fetchImpl, url) {
  const response = await fetchImpl(url);
  if (!response.ok) {
    throw new Error(`could not load messages from ${url}: HTTP ${response.status}`);
  }
  return response.json();
}

/**
 * Replaces each {name} in `template` with params[name]. A placeholder with no
 * matching param is left as written, so a missing value shows up on screen.
 */
export function formatMessage(template, params = {}) {
  return template.replace(PLACEHOLDER, (match, name) =>
    Object.hasOwn(params, name) ? String(params[name]) : match,
  );
}

/**
 * Returns `t(key, params)`. An unknown key throws, so a missing string is
 * found in development instead of shipping as a blank label.
 */
export function translator(messages) {
  return (key, params) => {
    if (!Object.hasOwn(messages, key)) {
      throw new Error(`missing message: ${key}`);
    }
    return formatMessage(messages[key], params);
  };
}
