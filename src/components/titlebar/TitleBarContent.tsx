import appIcon from '../../../assets/brand/icon-with-bg.svg'

export function TitleBarLeftActions({ title = 'Texpio' }: { title?: string }) {
  return (
    <>
      <img src={appIcon} alt="" draggable={false} className="pointer-events-none size-4 shrink-0" />
      <span className="pointer-events-none text-sm font-medium">{title}</span>
    </>
  )
}
