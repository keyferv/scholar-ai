import { Route, Routes } from "react-router-dom";
import { BrowserRouter } from "react-router-dom";
import Sidebar from "./Sidebar";
import DashboardPage from "./pages/DashboardPage";
import SearchPage from "./pages/SearchPage";
import ReportsPage from "./pages/ReportsPage";
import ConfigPage from "./pages/ConfigPage";

function App() {
  return (
    <BrowserRouter>
      <div className="flex h-screen bg-gray-100 dark:bg-gray-900">
        <Sidebar />
        <main className="flex-1 overflow-auto p-6">
          <Routes>
            <Route path="/" element={<DashboardPage />} />
            <Route path="/search" element={<SearchPage />} />
            <Route path="/reports" element={<ReportsPage />} />
            <Route path="/config" element={<ConfigPage />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  );
}

export default App;