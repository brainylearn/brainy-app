import styles from "./styles.module.css";
import Dialog from "../../../components/Dialog/Dialog";
import Form, {
	FormButtons,
	FormHeader,
	FormRows,
} from "../../../components/Form/Form";
import { mdiDeleteOutline, mdiPlusBoxOutline, mdiTuneVariant } from "@mdi/js";
import Icon from "@mdi/react";
import { useCallback, useEffect, useState } from "react";
import errorToString from "../../../utils/errorToString";
import Alert from "../../../components/Alert/Alert";
import FsrsProfile from "../../../types/backend/entity/fsrsProfile";
import {
	createProfile,
	deleteFsrsProfile,
	getAllFsrsProfiles,
	getFileFsrsProfile,
	getFolderFsrsProfile,
	getFsrsProfileChoiceForFile,
	getFsrsProfileChoiceForFolder,
	getParentFsrsProfileForFile,
	getParentFsrsProfileForFolder,
	setFsrsProfileChoiceForFile,
	setFsrsProfileChoiceForFolder,
	updateProfile,
} from "../../../api/fsrsApi";
import { ROOT_FOLDER_ID } from "../../../config/constants";
import {
	FsrsProfileChoice,
	FsrsProfileChoiceId,
} from "../../../types/backend/value_objects/fsrsProfileChoice";
import ConfirmationDialog from "../../../components/ConfirmationDialog/ConfirmationDialog";

interface Props {
	id: string;
	isFolder: boolean;
	onClose: () => void;
}

// TODO: unit test
export default function FsrsDialog({ id, isFolder, onClose }: Props) {
	const [allFsrsProfiles, setAllFsrsProfiles] = useState<FsrsProfile[]>([]);
	const [errorMessage, setErrorMessage] = useState("");
	const [chosenProfile, setChosenProfile] = useState<FsrsProfileChoice>({
		type: "inherit",
	});
	const [profileState, setProfileState] = useState<
		(Omit<FsrsProfile, "weights"> & { weights: string }) | null
	>();
	const [showDeleteProfileDialog, setShowDeleteProfileDialog] =
		useState(false);

	const isRoot = id === ROOT_FOLDER_ID;

	const executeRequest = useCallback(
		async (cb: () => Promise<void>): Promise<void> => {
			try {
				await cb();
			} catch (e) {
				console.error(e);
				setErrorMessage(errorToString(e));
			}
		},
		[],
	);

	const setChosenProfileAndState = (
		chosenProfile: FsrsProfileChoice,
		state: FsrsProfile,
	) => {
		setChosenProfile(chosenProfile);
		setProfileState({
			...state,
			weights: state.weights.join(" "),
		});
	};

	/** Loads all profiles, set the choice and the state for this components
	 * by calling the backend for getting these values. */
	const loadComponentState = useCallback(async () => {
		setAllFsrsProfiles(await getAllFsrsProfiles());
		const profileChoice = isFolder
			? await getFsrsProfileChoiceForFolder(id)
			: await getFsrsProfileChoiceForFile(id);

		const itemProfile = isFolder
			? await getFolderFsrsProfile(id)
			: await getFileFsrsProfile(id);

		setChosenProfileAndState(profileChoice, itemProfile);
	}, [id, isFolder]);

	useEffect(() => {
		void (async () => await loadComponentState())();
	}, [loadComponentState]);

	const verifyWeightsAndGetAsNumbers = (): number[] | null => {
		const weightsAsNumber = profileState?.weights
			.split(" ")
			.map(w => Number(w))
			.filter(w => !isNaN(w));
		if (weightsAsNumber?.length != 21) {
			setErrorMessage("Please enter 21 weights separated by space!");
			return null;
		}

		return weightsAsNumber;
	};

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();

		const weightsAsNumber = verifyWeightsAndGetAsNumbers();
		if (weightsAsNumber === null) return;

		await executeRequest(async () => {
			await updateProfile({
				...profileState!,
				weights: weightsAsNumber,
			});

			if (isFolder) {
				await setFsrsProfileChoiceForFolder(id, chosenProfile);
			} else {
				await setFsrsProfileChoiceForFile(id, chosenProfile);
			}
			onClose();
		});
	};

	const handleDelete = async () => {
		await deleteFsrsProfile((chosenProfile as FsrsProfileChoiceId).content);
		await loadComponentState();
		setShowDeleteProfileDialog(false);
	};

	const changeProfileChoice = async (newValue: string) => {
		if (newValue === "inherit") {
			const itemProfile = isFolder
				? await getParentFsrsProfileForFolder(id)
				: await getParentFsrsProfileForFile(id);
			setChosenProfileAndState(
				{
					type: "inherit",
				},
				itemProfile,
			);
		} else {
			const itemProfile = allFsrsProfiles.find(p => p.id === newValue);
			setChosenProfileAndState(
				{
					type: "id",
					content: newValue,
				},
				itemProfile!,
			);
		}
	};

	const handleCloneProfile = async () => {
		if (!profileState) return;

		const weightsAsNumber = verifyWeightsAndGetAsNumbers();
		if (weightsAsNumber === null) return;

		const profile = await createProfile({
			name: profileState.name + " clone",
			maximumInterval: profileState.maximumInterval,
			requestRetention: profileState.requestRetention,
			weights: weightsAsNumber,
		});
		await loadComponentState();
		await changeProfileChoice(profile.id);
	};

	return (
		<>
			<Dialog onHide={onClose} focusTrap className={styles.fsrsDialog}>
				<Form onSubmit={e => void handleSubmit(e)}>
					<FormHeader icon={mdiTuneVariant} title="FSRS Profile" />

					{profileState && (
						<FormRows
							rows={[
								{
									label: "Profile",
									labelHtmlFor: "profile",
									children: (
										<div
											className={styles.chooseProfileRow}>
											<select
												id="profile"
												value={
													chosenProfile.type ===
													"inherit"
														? chosenProfile.type
														: chosenProfile.content
												}
												onChange={e =>
													void changeProfileChoice(
														e.target.value,
													)
												}
												autoFocus>
												{!isRoot && (
													<option value="inherit">
														Inherit from parent
													</option>
												)}

												{allFsrsProfiles.map(
													profile => (
														<option
															value={profile.id}
															key={profile.id}>
															{profile.name}
														</option>
													),
												)}
											</select>

											<button
												className="transparent"
												type="button"
												title="Clone profile"
												onClick={() =>
													void handleCloneProfile()
												}>
												<Icon
													path={mdiPlusBoxOutline}
													size={1}
												/>
											</button>

											<button
												className="red"
												type="button"
												disabled={
													chosenProfile.type ===
														"inherit" ||
													allFsrsProfiles.length === 1
												}
												onClick={() =>
													setShowDeleteProfileDialog(
														true,
													)
												}
												title="Delete profile">
												<Icon
													path={mdiDeleteOutline}
													size={1}
												/>
											</button>
										</div>
									),
								},
								{
									label: "Name",
									labelHtmlFor: "name",
									children: (
										<input
											id="name"
											type="text"
											minLength={1}
											value={profileState.name}
											onChange={e =>
												setProfileState({
													...profileState,
													name: e.target.value,
												})
											}
											readOnly={
												chosenProfile.type === "inherit"
											}
											required
										/>
									),
								},
								{
									label: "Request retention",
									labelHtmlFor: "request-retention",
									children: (
										<input
											id="request-retention"
											type="number"
											step="any"
											readOnly={
												chosenProfile.type === "inherit"
											}
											value={
												profileState.requestRetention
											}
											onChange={e =>
												setProfileState({
													...profileState,
													requestRetention: Number(
														e.target.value,
													),
												})
											}
											required
										/>
									),
								},
								{
									label: "Maximum interval",
									labelHtmlFor: "maximum-interval",
									children: (
										<input
											id="maximum-interval"
											type="number"
											step="any"
											readOnly={
												chosenProfile.type === "inherit"
											}
											value={profileState.maximumInterval}
											onChange={e =>
												setProfileState({
													...profileState,
													maximumInterval: Number(
														e.target.value,
													),
												})
											}
											required
										/>
									),
								},
								{
									label: "Weights",
									labelHtmlFor: "weights",
									children: (
										<textarea
											id="weights"
											placeholder="0.212 1.2931 2.3065 8.2956 6.4133 0.8334 3.0194 0.001 1.8722 0.1666 0.796 1.4835 0.0614 0.2629 1.6483 0.6014 1.8729 0.5425 0.0912 0.0658 0.1542"
											rows={3}
											readOnly={
												chosenProfile.type === "inherit"
											}
											value={profileState.weights}
											onChange={e =>
												setProfileState({
													...profileState,
													weights: e.target.value,
												})
											}
											required
										/>
									),
								},
							]}
						/>
					)}

					{errorMessage && (
						<Alert type="error" onClose={() => setErrorMessage("")}>
							<p>{errorMessage}</p>
						</Alert>
					)}

					<FormButtons onClose={onClose} submitText="Save" />
				</Form>
			</Dialog>

			{showDeleteProfileDialog && (
				<ConfirmationDialog
					title="Delete FSRS profile?"
					icon={mdiDeleteOutline}
					text="Are you sure you want to delete the FSRS profile?"
					onCancel={() => setShowDeleteProfileDialog(false)}
					onConfirm={() => void handleDelete()}
				/>
			)}
		</>
	);
}
