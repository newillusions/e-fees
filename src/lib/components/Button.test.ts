/**
 * Button Component Tests
 * 
 * Tests for the base Button component including all variants,
 * states, and interaction patterns.
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import Button from './Button.svelte';

describe('Button Component', () => {
  it('should render a button element', () => {
    render(Button);
    
    const button = screen.getByRole('button');
    expect(button).toBeInTheDocument();
    expect(button).toHaveAttribute('type', 'button');
  });

  it('should apply default classes', () => {
    render(Button);
    
    const button = screen.getByRole('button');
    expect(button).toHaveClass('font-medium', 'rounded-lg', 'bg-emittiv-splash', 'text-emittiv-black');
  });

  it('should apply variant classes correctly', () => {
    // Test secondary variant
    const { container: container1 } = render(Button, { variant: 'secondary' });
    let button = container1.querySelector('button');
    expect(button).toHaveClass('bg-emittiv-dark', 'text-emittiv-white');

    // Test ghost variant  
    const { container: container2 } = render(Button, { variant: 'ghost' });
    button = container2.querySelector('button');
    expect(button).toHaveClass('text-emittiv-light');

    // Test primary variant
    const { container: container3 } = render(Button, { variant: 'primary' });
    button = container3.querySelector('button');
    expect(button).toHaveClass('bg-emittiv-splash', 'text-emittiv-black');
  });

  it('should apply size classes correctly', () => {
    // Test small size
    const { container: container1 } = render(Button, { size: 'sm' });
    let button = container1.querySelector('button');
    expect(button).toHaveClass('px-3', 'py-1.5', 'text-sm');

    // Test medium size
    const { container: container2 } = render(Button, { size: 'md' });
    button = container2.querySelector('button');
    expect(button).toHaveClass('px-4', 'py-2', 'text-base');

    // Test large size
    const { container: container3 } = render(Button, { size: 'lg' });
    button = container3.querySelector('button');
    expect(button).toHaveClass('px-6', 'py-3', 'text-lg');
  });

  it('should handle disabled state', () => {
    render(Button, { disabled: true });
    
    const button = screen.getByRole('button');
    expect(button).toBeDisabled();
    expect(button).toHaveClass('disabled:opacity-50', 'disabled:cursor-not-allowed');
  });

  it('should handle type attribute', () => {
    // Test submit type
    const { container: container1 } = render(Button, { type: 'submit' });
    let button = container1.querySelector('button');
    expect(button).toHaveAttribute('type', 'submit');

    // Test button type (default)
    const { container: container2 } = render(Button, { type: 'button' });
    button = container2.querySelector('button');
    expect(button).toHaveAttribute('type', 'button');
  });

  it('should apply custom className', () => {
    render(Button, { className: 'custom-class another-class' });
    
    const button = screen.getByRole('button');
    expect(button).toHaveClass('custom-class', 'another-class');
  });

  it('should emit click events', async () => {
    const user = userEvent.setup();
    const handleClick = vi.fn();
    
    render(Button);
    
    const button = screen.getByRole('button');
    button.addEventListener('click', handleClick);
    await user.click(button);
    
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('should not emit click when disabled', async () => {
    const user = userEvent.setup();
    const handleClick = vi.fn();
    
    render(Button, { disabled: true });
    
    const button = screen.getByRole('button');
    button.addEventListener('click', handleClick);
    await user.click(button);
    
    expect(handleClick).not.toHaveBeenCalled();
  });

  it('should be focusable by default', () => {
    render(Button);
    
    const button = screen.getByRole('button');
    button.focus();
    expect(button).toHaveFocus();
  });

  it('should not be focusable when disabled', () => {
    render(Button, { disabled: true });
    
    const button = screen.getByRole('button');
    expect(button).toBeDisabled();
  });
});