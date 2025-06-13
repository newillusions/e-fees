import './styles/app.css'
import { mount } from 'svelte'
import App from './App.svelte'

console.log('main.ts is executing!');

const appDiv = document.getElementById('app');
if (appDiv) {
  console.log('Found app div, creating Svelte 5 app...');
  try {
    const app = mount(App, {
      target: appDiv,
    });
    console.log('Svelte 5 app created successfully!', app);
  } catch (error) {
    console.error('Failed to create Svelte app:', error);
    // Fallback to show error
    appDiv.innerHTML = `
      <div style="color: white; padding: 20px; background: linear-gradient(135deg, #000, #333); min-height: 100vh;">
        <h1>Svelte Error</h1>
        <p>Failed to load Svelte app: ${error}</p>
      </div>
    `;
  }
} else {
  console.error('Could not find app div!');
}