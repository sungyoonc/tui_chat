import React from 'react';
import '../css/App.css'

function Home() {
  return (
    <div></div>
  );
}

function NaviBar() {
  return (
    <div className="NaviBar">
      <div className="Logo">
        <a className="LogoLink" href='/'></a>
        <svg className="LogoSvg" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" xmlnsXlink="http://www.w3.org/1999/xlink" xmlSpace="preserve" width="100%" height="100%">
          <path xmlns="http://www.w3.org/2000/svg" d="M18 2H6a3 3 0 0 0-3 3v11a3 3 0 0 0 3 3h2.59l2.7 2.71A1 1 0 0 0 12 22a1 1 0 0 0 .65-.24L15.87 19H18a3 3 0 0 0 3-3V5a3 3 0 0 0-3-3zm1 14a1 1 0 0 1-1 1h-2.5a1 1 0 0 0-.65.24l-2.8 2.4-2.34-2.35A1 1 0 0 0 9 17H6a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h12a1 1 0 0 1 1 1z">
          </path>
        </svg>
      </div>
      <div className="Signup">
        <p className="SignupText">signup</p>
      </div>
      <div className="Signin">

      </div>
    </div>
  );
}

function Index() {
  return (
    <div>
      <NaviBar></NaviBar>
      <Home></Home>
    </div>

  );
}

export default Index;