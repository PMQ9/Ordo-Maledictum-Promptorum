import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Layout } from './components/Layout';
import { QueryInterface } from './pages/QueryInterface';
import { ApprovalReview } from './pages/ApprovalReview';
import { AuditLogs } from './pages/AuditLogs';

function App() {
  return (
    <Router>
      <Layout>
        <Routes>
          <Route path="/" element={<QueryInterface />} />
          <Route path="/approval/:id" element={<ApprovalReview />} />
          <Route path="/audit" element={<AuditLogs />} />
        </Routes>
      </Layout>
    </Router>
  );
}

export default App;
