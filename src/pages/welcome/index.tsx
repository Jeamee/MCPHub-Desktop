import logo from "@/assets/logo.svg";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

type DependencyStatus = {
  uv: boolean;
  node: boolean;
};

export default function WelcomePage() {
  const [npmInstalled, setNpmInstalled] = useState(false);
  const [npmIsInstalling, setNpmIsInstalling] = useState(false);
  const [uvInstalled, setUvInstalled] = useState(false);
  const [uvIsInstalling, setUvIsInstalling] = useState(false);
  const [isChecking, setIsChecking] = useState(false);
  const [resourceLoaded, setResourceLoaded] = useState(false);

  const checkDependencies = async () => {
    try {
      setIsChecking(true);
      const status = await invoke<DependencyStatus>("check_dependency");
      setNpmInstalled(status.node);
      setUvInstalled(status.uv);
    } finally {
      setIsChecking(false);
    }
  };

  const checkResource = async () => {
    try {
      setResourceLoaded(await invoke("check_resource"));
    } finally {
      setIsChecking(false);
    }
  };

  const installNpm = async () => {
    try {
      setNpmIsInstalling(true);
      await invoke("install_npm");
      await checkDependencies();
    } finally {
      setNpmIsInstalling(false);
    }
  };
  const installUv = async () => {
    try {
      setUvIsInstalling(true);
      await invoke("install_uv");
      await checkDependencies();
    } finally {
      setUvIsInstalling(false);
    }
  };

  useEffect(() => {
    checkDependencies();
    checkResource();
    // Set up periodic check every 10 seconds
    const intervalId = setInterval(() => {
      checkDependencies();
    }, 10000);

    // Cleanup interval on component unmount
    return () => clearInterval(intervalId);
  }, []);

  return (
    <div className="flex flex-col items-center w-full max-w-4xl mx-auto p-8">
      <h1 className="text-3xl font-bold text-center mb-8">Welcome to MCPHub</h1>

      <div className="flex justify-center items-center gap-8 mb-8">
        <a href="https://www.mcphub.net" target="_blank" className="hover:scale-110 transition-transform">
          <img src={logo} className="h-24 w-24" alt="MCPHub logo" />
        </a>
      </div>

      <div className="flex flex-col md:flex-row w-full gap-4 px-4">
        <div className="flex-1 text-center p-4 bg-gray-50 rounded-lg">
          <h2 className="text-xl font-semibold mb-2">Node.js Status</h2>
          <div className="flex items-center justify-center gap-3">
            {isChecking ? (
              <div className="animate-spin h-3 w-3 border-2 border-blue-500 border-t-transparent rounded-full"></div>
            ) : (
              <span className={`inline-block w-3 h-3 rounded-full ${npmInstalled ? 'bg-green-500' : 'bg-red-500'}`}></span>
            )}
            <span className="text-gray-700">
              {npmInstalled ? 'Installed' : 'Not installed'}
            </span>
            {!npmInstalled && !npmIsInstalling && !isChecking && (
              <button
                onClick={installNpm}
                className="ml-4 px-4 py-1 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
              >
                Install Node.js
              </button>
            )}
            {npmIsInstalling && (
              <div className="ml-4 flex items-center gap-2">
                <div className="animate-spin h-5 w-5 border-2 border-blue-500 border-t-transparent rounded-full"></div>
                <span className="text-blue-500">Installing...</span>
              </div>
            )}
          </div>
        </div>

        <div className="flex-1 text-center p-4 bg-gray-50 rounded-lg">
          <h2 className="text-xl font-semibold mb-2">UV Status</h2>
          <div className="flex items-center justify-center gap-3">
            {isChecking ? (
              <div className="animate-spin h-3 w-3 border-2 border-blue-500 border-t-transparent rounded-full"></div>
            ) : (
              <span className={`inline-block w-3 h-3 rounded-full ${uvInstalled ? 'bg-green-500' : 'bg-red-500'}`}></span>
            )}
            <span className="text-gray-700">
              {uvInstalled ? 'Installed' : 'Not installed'}
            </span>
            {!uvInstalled && !uvIsInstalling && !isChecking && (
              <button
                onClick={installUv}
                className="ml-4 px-4 py-1 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
              >
                Install UV
              </button>
            )}
            {uvIsInstalling && (
              <div className="ml-4 flex items-center gap-2">
                <div className="animate-spin h-5 w-5 border-2 border-blue-500 border-t-transparent rounded-full"></div>
                <span className="text-blue-500">Installing...</span>
              </div>
            )}
          </div>
        </div>
        <div className="flex-1 text-center p-4 bg-gray-50 rounded-lg">
          <h2 className="text-xl font-semibold mb-2">Resource Status</h2>
          <div className="flex items-center justify-center gap-3">
            {isChecking ? (
              <div className="animate-spin h-3 w-3 border-2 border-blue-500 border-t-transparent rounded-full"></div>
            ) : (
              <span className={`inline-block w-3 h-3 rounded-full ${resourceLoaded ? 'bg-green-500' : 'bg-red-500'}`}></span>
            )}
            <span className="text-gray-700">
              {resourceLoaded ? 'Loaded' : 'Not loaded'}
            </span>
            {!resourceLoaded && !isChecking && (
              <button
                onClick={checkResource}
                className="ml-4 px-4 py-1 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
              >
                Check Resource
              </button>
            )}
            {isChecking && (
              <div className="ml-4 flex items-center gap-2">
                <div className="animate-spin h-5 w-5 border-2 border-blue-500 border-t-transparent rounded-full"></div>
                <span className="text-blue-500">Checking...</span>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
