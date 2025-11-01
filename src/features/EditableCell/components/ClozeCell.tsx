import RichTextEditor from "../../../components/RichTextEditor/RichTextEditor";
import {
	mdiDotsHorizontal,
	mdiNumericNegative1,
	mdiNumericPositive1,
} from "@mdi/js";
import Cell from "../../../types/backend/entity/cell";
import {
	ClozePlugin,
	DECREASE_CLOZE_GROUP_NUMBER,
	INCREASE_CLOZE_GROUP_NUMBER,
	TOGGLE_CLOZE_NODE,
} from "../plugins/clozePlugin";
import { LexicalEditor } from "lexical";
import { $isSelectionInsideCloze, ClozeNode } from "../plugins/clozeNode";

interface Props {
	cell: Cell;
	autofocus: boolean;
	onUpdate: (content: string) => void;
	onFocus: (editor: LexicalEditor) => void;
}

function ClozeCell({ cell, autofocus, onUpdate, onFocus }: Props) {
	return (
		<RichTextEditor
			extraNodes={[ClozeNode]}
			additionalFloatingMenuButtons={[
				{
					name: "Toggle Cloze",
					icon: mdiDotsHorizontal,
					title: "Toggle Cloze",
					onClick: editor =>
						editor.dispatchCommand(TOGGLE_CLOZE_NODE, undefined),
					isActive: $isSelectionInsideCloze,
				},
				{
                    // TODO: should be visibile only within cloze
					name: "Cloze+1",
					icon: mdiNumericPositive1,
					title: "Increase cloze group number",
					onClick: editor =>
						editor.dispatchCommand(
							INCREASE_CLOZE_GROUP_NUMBER,
							undefined,
						),
					isActive: () => false,
				},
				{
					name: "Cloze-1",
					icon: mdiNumericNegative1,
					title: "Decrease cloze group number",
					onClick: editor =>
						editor.dispatchCommand(
							DECREASE_CLOZE_GROUP_NUMBER,
							undefined,
						),
                    // TODO: should be visibile only within cloze
					isActive: () => false,
				},
			]}
			content={cell.content}
			autofocus={autofocus}
			onChange={onUpdate}
			onFocus={onFocus}
			plugins={[<ClozePlugin key={1} />]}
		/>
	);
}

export default ClozeCell;
