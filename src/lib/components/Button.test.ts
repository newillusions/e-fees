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
    // Updated to match current Button component classes
    expect(button).toHaveClass('font-medium', 'rounded', 'bg-emittiv-splash', 'text-emittiv-black');
  });

  it('should apply variant classes correctly', () => {
    // Test secondary variant - now uses bg-transparent
    const { container: container1 } = render(Button, { variant: 'secondary' });
    let button = container1.querySelector('button');
    expect(button).toHaveClass(
      'bg-transparent',
      'text-emittiv-light',
      'border',
      'border-emittiv-dark'
    );

    // Test ghost variant
    const { container: container2 } = render(Button, { variant: 'ghost' });
    button = container2.querySelector('button');
    expect(button).toHaveClass('text-emittiv-light', 'bg-transparent');

    // Test primary variant
    const { container: container3 } = render(Button, { variant: 'primary' });
    button = container3.querySelector('button');
    expect(button).toHaveClass('bg-emittiv-splash', 'text-emittiv-black');

    // Test danger variant
    const { container: container4 } = render(Button, { variant: 'danger' });
    button = container4.querySelector('button');
    expect(button).toHaveClass('bg-red-600', 'text-white', 'border-red-500');
  });

  it('should apply size styles correctly via inline style', () => {
    // Button now uses inline styles for sizes instead of classes
    // Test small size
    const { container: container1 } = render(Button, { size: 'sm' });
    let button = container1.querySelector('button');
    expect(button).toHaveAttribute('style');
    expect(button?.getAttribute('style')).toContain('height: 28px');
    expect(button?.getAttribute('style')).toContain('font-size: 12px');

    // Test medium size (default)
    const { container: container2 } = render(Button, { size: 'md' });
    button = container2.querySelector('button');
    expect(button?.getAttribute('style')).toContain('height: 32px');
    expect(button?.getAttribute('style')).toContain('font-size: 13px');

    // Test large size
    const { container: container3 } = render(Button, { size: 'lg' });
    button = container3.querySelector('button');
    expect(button?.getAttribute('style')).toContain('height: 40px');
    expect(button?.getAttribute('style')).toContain('font-size: 14px');
  });

  it('should handle disabled state', () => {
    render(Button, { disabled: true });

    const button = screen.getByRole('button');
    expect(button).toBeDisabled();
    expect(button).toHaveClass('disabled:opacity-50', 'disabled:cursor-not-allowed');
  });

  it('should handle loading state', () => {
    const { container } = render(Button, { loading: true });

    const button = container.querySelector('button');
    expect(button).toBeDisabled();

    // Loading state shows a spinner
    const spinner = container.querySelector('.emittiv-spinner');
    expect(spinner).toBeInTheDocument();
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

  it('should not emit click when loading', async () => {
    const user = userEvent.setup();
    const handleClick = vi.fn();

    render(Button, { loading: true });

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

  it('should have focus ring classes', () => {
    render(Button);

    const button = screen.getByRole('button');
    expect(button).toHaveClass('focus:outline-none', 'focus:ring-2', 'focus:ring-emittiv-splash');
  });
});
