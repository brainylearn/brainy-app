import RichTextEditor from "../../../components/RichTextEditor/RichTextEditor";
import {
	mdiDotsHorizontal,
	mdiNumericNegative1,
	mdiNumericPositive1,
} from "@mdi/js";
import Cell from "../../../types/backend/entity/cell";
import clozeMark, {
    $isClozeNode,
	ClozeNode,
	ClozePlugin,
	TOGGLE_CLOZE_NODE,
} from "../utils/clozeMark";
import { clozeMarkName } from "../config/constants";
import { LexicalEditor } from "lexical";

interface Props {
	cell: Cell;
	autofocus: boolean;
	eagerLoadRichTextEditor: boolean;
	onUpdate: (content: string) => void;
	onFocus: (editor: LexicalEditor) => void;
}

const regexp = /<cloze[^>]*index="(\d+)"[^>]*>/g;

function ClozeCell({
	cell,
	autofocus,
	eagerLoadRichTextEditor,
	onUpdate,
	onFocus,
}: Props) {
	const handleToggleCloze = (editor: LexicalEditor, isActive: boolean) => {
		// TODO:
	};

	return (
		<RichTextEditor
			extraNodes={[ClozeNode]}
			eagerLoadRichTextEditor={eagerLoadRichTextEditor}
			additionalFloatingMenuButtons={[
				{
					name: clozeMarkName,
					icon: mdiDotsHorizontal,
					title: "Cloze",
					onClick: editor =>
						editor.dispatchCommand(TOGGLE_CLOZE_NODE, undefined),
					isActive: (selection) => {
                        let allCloze = true;
                        for (const node of selection.getNodes()) {
                            let anyCloze = false;
                            let current = node.getParent();
                            while (current !== null) {
                                anyCloze = anyCloze || $isClozeNode(current);
                                current = current.getParent();
                            }
                            allCloze = allCloze && anyCloze;
                        }

                        return allCloze
                    },
				},
				// TODO:
				// {
				// 	name: "Cloze+1",
				// 	icon: mdiNumericPositive1,
				// 	title: "Increase cloze group number",
				// 	onClick: c => c.increaseClozeIndex(),
				// },
				// {
				// 	name: "Cloze-1",
				// 	icon: mdiNumericNegative1,
				// 	title: "Decrease cloze group number",
				// 	onClick: c => c.decreaseClozeIndex(),
				// },
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
