import { DragDropProvider, useDroppable, useDraggable } from "@dnd-kit/react";
import { DragDropProviderProps } from "../../components/DefaultDragDropProvider/DefaultDragDropProvider";

type useDroppableReturnType = ReturnType<typeof useDroppable>;
type useDraggableReturnType = ReturnType<typeof useDraggable>;

export function mockDndKit() {
	return {
		...mockDragDropProvider(),
		...mockUseDraggable(),
		...mockUseDroppable(),
	};
}

export function mockDragDropProvider() {
	let capturedProps: DragDropProviderProps | null = null;

	vi.mocked(DragDropProvider).mockImplementation(props => {
		capturedProps = props;
		return <>{props.children}</>;
	});

	return {
		getCapturedProviderProps: () => capturedProps,
	};
}

export function mockUseDraggable(
	returnValue: Partial<useDraggableReturnType> = {},
) {
	let capturedCaptured: Parameters<typeof useDraggable>[0] | null = null;

	vi.mocked(useDraggable).mockImplementation(input => {
		capturedCaptured = input;
		return {
			isDragging: false,
			handleRef: vi.fn(),
			ref: vi.fn(),
			...returnValue,
		} as unknown as useDraggableReturnType;
	});

	return {
		getUseDraggableInputs: () => capturedCaptured,
	};
}

export function mockUseDroppable(
	returnValue: Partial<useDroppableReturnType> = {},
) {
	let capturedInput: Parameters<typeof useDroppable>[0] | null = null;

	vi.mocked(useDroppable).mockImplementation(input => {
		capturedInput = input;
		return {
			ref: vi.fn(),
			...returnValue,
		} as unknown as useDroppableReturnType;
	});

	return {
		getUseDroppableInputs: () => capturedInput,
	};
}
