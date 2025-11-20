export default function DocsPage() {
  return (
    <main style={{ padding: '2rem', maxWidth: '1200px', margin: '0 auto' }}>
      <h1>API Documentation</h1>
      
      <section style={{ marginTop: '2rem' }}>
        <h2>Base URL</h2>
        <code>http://localhost:8080/api</code>
      </section>

      <section style={{ marginTop: '2rem' }}>
        <h2>Endpoints</h2>
        
        <div style={{ marginTop: '1.5rem' }}>
          <h3>GET /api/accounts</h3>
          <p>Get all email accounts</p>
          <pre style={{ background: '#f5f5f5', padding: '1rem', borderRadius: '4px', overflow: 'auto' }}>
{`Response: [
  {
    "id": "string",
    "email": "string",
    "displayName": "string",
    "isActive": boolean
  }
]`}
          </pre>
        </div>

        <div style={{ marginTop: '1.5rem' }}>
          <h3>POST /api/accounts</h3>
          <p>Create a new email account</p>
          <pre style={{ background: '#f5f5f5', padding: '1rem', borderRadius: '4px', overflow: 'auto' }}>
{`Request Body: {
  "email": "string",
  "displayName": "string",
  "password": "string",
  "isActive": boolean
}`}
          </pre>
        </div>

        <div style={{ marginTop: '1.5rem' }}>
          <h3>PATCH /api/accounts/:id</h3>
          <p>Update an email account</p>
          <pre style={{ background: '#f5f5f5', padding: '1rem', borderRadius: '4px', overflow: 'auto' }}>
{`Request Body: {
  "isActive": boolean
}`}
          </pre>
        </div>

        <div style={{ marginTop: '1.5rem' }}>
          <h3>POST /api/send</h3>
          <p>Send an email</p>
          <pre style={{ background: '#f5f5f5', padding: '1rem', borderRadius: '4px', overflow: 'auto' }}>
{`Request Body: {
  "from": "string",
  "to": "string",
  "subject": "string",
  "body": "string"
}`}
          </pre>
        </div>

        <div style={{ marginTop: '1.5rem' }}>
          <h3>GET /api/inbox</h3>
          <p>Get inbox messages (IMAP)</p>
          <pre style={{ background: '#f5f5f5', padding: '1rem', borderRadius: '4px', overflow: 'auto' }}>
{`Query Parameters:
  account: email address
  limit: number (optional)`}
          </pre>
        </div>
      </section>
    </main>
  )
}

