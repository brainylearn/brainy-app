import { LexicalEditor, RangeSelection } from "lexical";
import styles from "../../styles.module.css";
import Icon from "@mdi/react";

export interface IFloatingMenuButton {
	icon: string;
	name: string;
	title: string;
	onClick: (editor: LexicalEditor, isActive: boolean) => void;
	isActive: (selection: RangeSelection) => boolean;
}

interface IProps {
	editor: LexicalEditor;
	floatingButtonProps: IFloatingMenuButton;
	state: Record<string, boolean>;
}

export default function FloatingMenuButton({
	editor,
	floatingButtonProps,
	state,
}: IProps) {
	return (
		<button
			onClick={() =>
				floatingButtonProps.onClick(
					editor,
					state[floatingButtonProps.name],
				)
			}
			className={`transparent ${state[floatingButtonProps.name] && styles.activeButton}`}
			title={floatingButtonProps.title}
			aria-label={floatingButtonProps.title}>
			<Icon path={floatingButtonProps.icon} size={1} />
		</button>
	);
}
