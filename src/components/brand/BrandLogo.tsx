import lightLogo from '../../../assets/brand/logo-light-theme.svg'
import darkLogo from '../../../assets/brand/logo-dark-theme.svg'
import { cn } from '@/lib/utils'

export function BrandLogo({ className }: { className?: string }) {
  return (
    <span className={cn('inline-flex items-center', className)}>
      <img src={lightLogo} alt="" className="h-full w-auto max-w-full dark:hidden" />
      <img src={darkLogo} alt="" className="hidden h-full w-auto max-w-full dark:block" />
      <span className="sr-only">Texpio</span>
    </span>
  )
}
