import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { trace} from '@tauri-apps/plugin-log';

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [npmInstalled, setNpmInstalled] = useState(false);
  const [npmIsInstalling, setNpmIsInstalling] = useState(false);
  const [uvInstalled, setUvInstalled] = useState(false);
  const [uvIsInstalling, setUvIsInstalling] = useState(false);

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  const checkNpmStatus = async () => {
    const npmInstallResult = await invoke<boolean>("check_npm");
    trace("npm status: " + npmInstallResult);
    setNpmInstalled(npmInstallResult);
  };
  const checkUvStatus = async () => {
    const uvInstallResult = await invoke<boolean>("check_uv");
    trace("uv status: " + uvInstallResult);
    setUvInstalled(uvInstallResult);
  };

  const installNpm = async () => {
    try {
      setNpmIsInstalling(true);
      await invoke("install_npm");
      await checkNpmStatus();
    } finally {
      setNpmIsInstalling(false);
    }
  };
  const installUv = async () => {
    try {
      setUvIsInstalling(true);
      await invoke("install_uv");
      await checkUvStatus();
    } finally {
      setUvIsInstalling(false);
    }
  };

  useEffect(() => {
    checkNpmStatus();
    checkUvStatus();
    
    // Set up periodic check every 10 seconds
    const intervalId = setInterval(() => {
      checkNpmStatus();
      checkUvStatus();
    }, 10000);

    // Cleanup interval on component unmount
    return () => clearInterval(intervalId);
  }, []);

  return (
    <main className="container mx-auto p-8 max-w-4xl">
      <h1 className="text-3xl font-bold text-center mb-8">Welcome to MCPHub</h1>

      <div className="flex justify-center items-center gap-8 mb-8">
        <a href="https://vitejs.dev" target="_blank" className="hover:scale-110 transition-transform">
          <img src="/vite.svg" className="h-16 w-16" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank" className="hover:scale-110 transition-transform">
          <img src="/tauri.svg" className="h-16 w-16" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank" className="hover:scale-110 transition-transform">
          <img src={reactLogo} className="h-16 w-16" alt="React logo" />
        </a>
      </div>

      <div className="text-center mb-8 p-4 bg-gray-50 rounded-lg">
        <h2 className="text-xl font-semibold mb-2">Node.js Status</h2>
        <div className="flex items-center justify-center gap-3">
          <span className={`inline-block w-3 h-3 rounded-full ${npmInstalled ? 'bg-green-500' : 'bg-red-500'}`}></span>
          <span className="text-gray-700">
            {npmInstalled ? 'Node.js is installed' : 'Node.js is not installed'}
          </span>
          {!npmInstalled && !npmIsInstalling && (
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

      <div className="text-center mb-8 p-4 bg-gray-50 rounded-lg">
        <h2 className="text-xl font-semibold mb-2">UV Status</h2>
        <div className="flex items-center justify-center gap-3">
          <span className={`inline-block w-3 h-3 rounded-full ${uvInstalled ? 'bg-green-500' : 'bg-red-500'}`}></span>
          <span className="text-gray-700">
            {uvInstalled ? 'UV is installed' : 'UV is not installed'}
          </span>
          {!uvInstalled && !uvIsInstalling && (
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

      <p className="text-center text-gray-600 mb-8">Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="flex flex-col items-center gap-4"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
          className="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button
          type="submit"
          className="px-6 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
        >
          Greet
        </button>
      </form>
      <p className="mt-4 text-center text-lg">{greetMsg}</p>
    </main>
  );
}

export default App;
