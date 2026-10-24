import { cn } from '@/lib/utils'

interface LeftSideBarProps {
  children?: React.ReactNode
  className?: string
}

export function LeftSideBar({ children, className }: LeftSideBarProps) {
  return (
    <div className={cn('flex h-full min-w-0 flex-col border-e bg-sidebar', className)}>
      {children}
    </div>
  )
}
