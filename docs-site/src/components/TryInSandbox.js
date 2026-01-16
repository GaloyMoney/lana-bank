import React from 'react';

/**
 * Apollo Sandbox "Try It" button component.
 *
 * Opens Apollo Sandbox with the GraphQL endpoint and operation pre-filled.
 *
 * @param {Object} props
 * @param {string} props.endpoint - GraphQL endpoint URL
 * @param {string} props.operation - GraphQL query/mutation string
 * @param {string} [props.operationName] - Name of the operation
 * @param {Object} [props.variables] - Default variables (optional)
 */
export default function TryInSandbox({ endpoint, operation, operationName, variables }) {
  // Build Apollo Sandbox URL with pre-filled operation
  const params = new URLSearchParams();
  params.set('endpoint', endpoint);

  if (operation) {
    params.set('document', operation);
  }

  if (variables && Object.keys(variables).length > 0) {
    params.set('variables', JSON.stringify(variables, null, 2));
  }

  const sandboxUrl = `https://studio.apollographql.com/sandbox/explorer?${params.toString()}`;

  return (
    <a
      href={sandboxUrl}
      target="_blank"
      rel="noopener noreferrer"
      className="try-in-sandbox-button"
      title={operationName ? `Try ${operationName} in Apollo Sandbox` : 'Try in Apollo Sandbox'}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '0.4rem',
        padding: '0.4rem 0.8rem',
        backgroundColor: 'var(--ifm-color-primary)',
        color: 'white',
        borderRadius: '4px',
        textDecoration: 'none',
        fontWeight: 500,
        fontSize: '0.85rem',
        marginBottom: '1rem',
        transition: 'background-color 0.2s ease',
      }}
      onMouseOver={(e) => e.currentTarget.style.backgroundColor = 'var(--ifm-color-primary-dark)'}
      onMouseOut={(e) => e.currentTarget.style.backgroundColor = 'var(--ifm-color-primary)'}
    >
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <polygon points="5 3 19 12 5 21 5 3"></polygon>
      </svg>
      Try in Sandbox
    </a>
  );
}
