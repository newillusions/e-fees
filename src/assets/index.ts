// Asset exports for easy importing

// Logo exports with simplified file names (as URLs for img src)
export { default as logoGrey } from './images/logo-grey.svg?url';
export { default as logoWhite } from './images/logo-white.svg?url';

// Alias exports for easier usage
export { default as logo } from './images/logo-white.svg?url'; // Default logo (white)
export { default as logoDark } from './images/logo-grey.svg?url'; // For light backgrounds  
export { default as logoLight } from './images/logo-white.svg?url'; // For dark backgrounds

// Direct SVG imports for inline usage
export { default as logoGreyInline } from './images/logo-grey.svg';
export { default as logoWhiteInline } from './images/logo-white.svg';

// This allows imports like:
// import { logo, logoLight, logoDark } from '../assets';