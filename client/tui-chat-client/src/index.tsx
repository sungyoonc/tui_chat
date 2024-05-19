import React from 'react';
import ReactDOM from 'react-dom/client';
import NotFound from './pages/not_found.tsx';
import Index from './pages/index.tsx';
import reportWebVitals from './reportWebVitals';
import {BrowserRouter as Router, Routes, Route} from "react-router-dom";

const rootElement = document.getElementById('root');
if (!rootElement) throw new Error('Failed to find the root element');
const root = ReactDOM.createRoot(rootElement);
root.render(
  <Router>
    <Routes>
      <Route path="/" element={<Index/>}/>
      <Route path={"*"} element={<NotFound/>}/>
    </Routes>
  </Router>
);

reportWebVitals();
