import React from 'react';
import { Link, useLocation } from 'react-router-dom';

interface LayoutProps {
  children: React.ReactNode;
}

export const Layout: React.FC<LayoutProps> = ({ children }) => {
  const location = useLocation();

  const isActive = (path: string) => {
    return location.pathname === path;
  };

  const navLinkClass = (path: string) => {
    const baseClass = 'px-4 py-2 rounded-md text-sm font-medium transition-colors';
    if (isActive(path)) {
      return `${baseClass} bg-blue-700 text-white`;
    }
    return `${baseClass} text-gray-300 hover:bg-blue-600 hover:text-white`;
  };

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-blue-800 text-white shadow-lg">
        <div className="container mx-auto px-4">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center">
              <h1 className="text-xl font-bold">Intent Segregation System</h1>
            </div>
            <nav className="flex space-x-2">
              <Link to="/" className={navLinkClass('/')}>
                Query Interface
              </Link>
              <Link to="/audit" className={navLinkClass('/audit')}>
                Audit Logs
              </Link>
            </nav>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="container mx-auto px-4 py-8">
        {children}
      </main>

      {/* Footer */}
      <footer className="bg-gray-800 text-gray-300 mt-12">
        <div className="container mx-auto px-4 py-6">
          <div className="text-center text-sm">
            <p>&copy; 2025 Intent Segregation Cybersecurity Architecture for AI</p>
            <p className="mt-1 text-gray-400">Multi-layered security through intent parsing and validation</p>
          </div>
        </div>
      </footer>
    </div>
  );
};
