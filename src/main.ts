import { mount } from 'svelte';
import App from './App.svelte';
import { applyTheme, loadTheme } from './lib/theme';
import './style.css';
applyTheme(loadTheme());
mount(App, { target: document.getElementById('app')! });
