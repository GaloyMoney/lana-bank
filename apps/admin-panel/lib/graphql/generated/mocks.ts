/* eslint-disable */
// @ts-nocheck
import { 
  UsdCents, 
  Satoshis, 
  SignedUsdCents, 
  SignedSatoshis,
} from 'types';

faker.seed(12345);

const getRandomEnumValue = <T extends Record<string, any>>(enumObj: T): T[keyof T] => {
  const enumValues = Object.values(enumObj).filter(v => typeof v === 'string') as T[keyof T][];
  return faker.helpers.arrayElement(enumValues);
};

// Generate mock values dynamically
const generateMockValue = {
  uuid: () => faker.string.uuid(),
  email: () => faker.internet.email(),
  telegramId: () => faker.string.alphanumeric(10),
  name: () => faker.person.fullName(),
  url: () => faker.internet.url(),
  description: () => faker.lorem.paragraph(),
  timestamp: () => faker.date.recent().toISOString(),
  reference: () => faker.string.alphanumeric(12),
  filename: () => faker.system.fileName(),
  boolean: () => faker.datatype.boolean(),
  usdCents: () => faker.number.int({ min: 0, max: 1000000 }) as UsdCents,
  satoshis: () => faker.number.int({ min: 0, max: 100000000 }) as Satoshis,
  signedUsdCents: () => faker.number.int({ min: -1000000, max: 1000000 }) as SignedUsdCents,
  signedSatoshis: () => faker.number.int({ min: -100000000, max: 100000000 }) as SignedSatoshis,
  int: () => faker.number.int({ min: 0, max: 1000 }),
  cursor: () => faker.string.alphanumeric(20),
  deniedReason: () => null,
  applicantId: () => faker.datatype.boolean() ? faker.string.uuid() : null,
  oneTimeFeeRate: () => faker.number.int({ min: 0, max: 5 })
};

const mockEnums = {
  accountStatus: () => getRandomEnumValue(AccountStatus),
  approvalProcessStatus: () => getRandomEnumValue(ApprovalProcessStatus),
  approvalProcessType: () => getRandomEnumValue(ApprovalProcessType),
  collateralAction: () => getRandomEnumValue(CollateralAction),
  collateralizationState: () => getRandomEnumValue(CollateralizationState),
  creditFacilityStatus: () => getRandomEnumValue(CreditFacilityStatus),
  disbursalStatus: () => getRandomEnumValue(DisbursalStatus),
  documentStatus: () => getRandomEnumValue(DocumentStatus),
  interestInterval: () => getRandomEnumValue(InterestInterval),
  kycLevel: () => getRandomEnumValue(KycLevel),
  period: () => getRandomEnumValue(Period),
  reportProgress: () => getRandomEnumValue(ReportProgress),
  role: () => getRandomEnumValue(Role),
  sortDirection: () => getRandomEnumValue(SortDirection),
  withdrawalStatus: () => getRandomEnumValue(WithdrawalStatus)
};

import { fakerEN as faker } from '@faker-js/faker';
import { AccountingCsvDocument, AccountingCsvDownloadLink, AccountingCsvDownloadLinkGenerateInput, AccountingCsvDownloadLinkGeneratePayload, ApprovalProcess, ApprovalProcessApproveInput, ApprovalProcessApprovePayload, ApprovalProcessConnection, ApprovalProcessDenyInput, ApprovalProcessDenyPayload, ApprovalProcessEdge, ApprovalProcessVoter, AuditEntry, AuditEntryConnection, AuditEntryEdge, BalanceSheet, BalanceSheetModuleConfig, BalanceSheetModuleConfigureInput, BalanceSheetModuleConfigurePayload, BitgoConfig, BtcAmount, BtcBalanceDetails, BtcLedgerAccountBalance, BtcLedgerAccountBalanceRange, CancelledWithdrawalEntry, ChartNode, ChartOfAccounts, ChartOfAccountsAddChildNodeInput, ChartOfAccountsAddChildNodePayload, ChartOfAccountsAddRootNodeInput, ChartOfAccountsAddRootNodePayload, ChartOfAccountsCsvImportInput, ChartOfAccountsCsvImportPayload, Collateral, CollateralBalance, Committee, CommitteeAddUserInput, CommitteeAddUserPayload, CommitteeConnection, CommitteeCreateInput, CommitteeCreatePayload, CommitteeEdge, CommitteeRemoveUserInput, CommitteeRemoveUserPayload, CommitteeThreshold, CreditFacilitiesFilter, CreditFacilitiesSort, CreditFacility, CreditFacilityApproved, CreditFacilityBalance, CreditFacilityCollateralSentOut, CreditFacilityCollateralUpdateInput, CreditFacilityCollateralUpdatePayload, CreditFacilityCollateralUpdated, CreditFacilityCollateralizationUpdated, CreditFacilityCompleteInput, CreditFacilityCompletePayload, CreditFacilityConnection, CreditFacilityDisbursal, CreditFacilityDisbursalConnection, CreditFacilityDisbursalEdge, CreditFacilityDisbursalExecuted, CreditFacilityDisbursalInitiateInput, CreditFacilityDisbursalInitiatePayload, CreditFacilityEdge, CreditFacilityIncrementalPayment, CreditFacilityInterestAccrued, CreditFacilityLedgerAccounts, CreditFacilityPartialPaymentRecordInput, CreditFacilityPartialPaymentRecordPayload, CreditFacilityPartialPaymentWithDateRecordInput, CreditFacilityPaymentAllocation, CreditFacilityProposal, CreditFacilityProposalConnection, CreditFacilityProposalCreateInput, CreditFacilityProposalCreatePayload, CreditFacilityProposalCustomerApprovalConcludeInput, CreditFacilityProposalCustomerApprovalConcludePayload, CreditFacilityProposalEdge, CreditFacilityRepaymentAmountReceived, CreditFacilityRepaymentPlanEntry, CreditModuleConfig, CreditModuleConfigureInput, CreditModuleConfigurePayload, Custodian, CustodianConfigInput, CustodianConfigUpdateInput, CustodianConfigUpdatePayload, CustodianConnection, CustodianCreateInput, CustodianCreatePayload, CustodianEdge, Customer, CustomerConnection, CustomerCreateInput, CustomerCreatePayload, CustomerDocument, CustomerDocumentArchiveInput, CustomerDocumentArchivePayload, CustomerDocumentCreateInput, CustomerDocumentCreatePayload, CustomerDocumentDeleteInput, CustomerDocumentDeletePayload, CustomerDocumentDownloadLinksGenerateInput, CustomerDocumentDownloadLinksGeneratePayload, CustomerEdge, CustomerEmailUpdateInput, CustomerEmailUpdatePayload, CustomerTelegramIdUpdateInput, CustomerTelegramIdUpdatePayload, CustomersFilter, CustomersSort, Dashboard, Deposit, DepositAccount, DepositAccountBalance, DepositAccountCloseInput, DepositAccountClosePayload, DepositAccountConnection, DepositAccountCreateInput, DepositAccountCreatePayload, DepositAccountEdge, DepositAccountFreezeInput, DepositAccountFreezePayload, DepositAccountHistoryEntryConnection, DepositAccountHistoryEntryEdge, DepositAccountLedgerAccounts, DepositAccountUnfreezeInput, DepositAccountUnfreezePayload, DepositConnection, DepositEdge, DepositEntry, DepositModuleConfig, DepositModuleConfigureInput, DepositModuleConfigurePayload, DepositRecordInput, DepositRecordPayload, DepositRevertInput, DepositRevertPayload, DisbursalEntry, Disbursed, Duration, DurationInput, ExposedConfigItem, ExposedConfigUpdateInput, ExposedConfigUpdatePayload, FacilityRemaining, FiniteCvlPct, FiscalMonthClosure, FiscalYear, FiscalYearCloseInput, FiscalYearCloseMonthInput, FiscalYearCloseMonthPayload, FiscalYearClosePayload, FiscalYearConnection, FiscalYearEdge, FiscalYearInitInput, FiscalYearInitPayload, FiscalYearOpenNextInput, FiscalYearOpenNextPayload, FreezeEntry, GovernanceNavigationItems, InfiniteCvlPct, Interest, JournalEntry, JournalEntryConnection, JournalEntryEdge, KomainuConfig, LedgerAccount, LedgerAccountBalanceRangeByCurrency, LedgerAccountCsvCreateInput, LedgerAccountCsvCreatePayload, LedgerTransaction, LedgerTransactionConnection, LedgerTransactionEdge, Liquidation, LiquidationCollateralSent, LiquidationConnection, LiquidationEdge, LiquidationPaymentReceived, LiquidationRecordCollateralSentInput, LiquidationRecordCollateralSentPayload, LiquidationRecordPaymentReceivedInput, LiquidationRecordPaymentReceivedPayload, Loan, LoanAgreement, LoanAgreementDownloadLinksGenerateInput, LoanAgreementDownloadLinksGeneratePayload, LoanAgreementGenerateInput, LoanAgreementGeneratePayload, ManualTransactionEntryInput, ManualTransactionExecuteInput, ManualTransactionExecutePayload, Me, Mutation, Outstanding, PageInfo, PaymentEntry, PendingCreditFacility, PendingCreditFacilityCollateralUpdateInput, PendingCreditFacilityCollateralUpdatePayload, PendingCreditFacilityCollateralizationUpdated, PendingCreditFacilityConnection, PendingCreditFacilityEdge, PermissionSet, PermissionSetConnection, PermissionSetEdge, Policy, PolicyAssignCommitteeInput, PolicyAssignCommitteePayload, PolicyConnection, PolicyEdge, ProfitAndLossModuleConfigureInput, ProfitAndLossStatement, ProfitAndLossStatementModuleConfig, ProfitAndLossStatementModuleConfigurePayload, Query, RealtimePrice, Report, ReportFile, ReportFileGenerateDownloadLinkInput, ReportFileGenerateDownloadLinkPayload, ReportRun, ReportRunConnection, ReportRunCreatePayload, ReportRunEdge, Role, RoleAddPermissionSetsInput, RoleAddPermissionSetsPayload, RoleConnection, RoleCreateInput, RoleCreatePayload, RoleEdge, RoleRemovePermissionSetsInput, RoleRemovePermissionSetsPayload, SumsubPermalinkCreateInput, SumsubPermalinkCreatePayload, System, SystemApproval, TermValues, TermsInput, TermsTemplate, TermsTemplateCreateInput, TermsTemplateCreatePayload, TermsTemplateUpdateInput, TermsTemplateUpdatePayload, Total, TransactionTemplate, TransactionTemplateConnection, TransactionTemplateEdge, TrialBalance, UnfreezeEntry, UnknownEntry, UsdAmount, UsdBalanceDetails, UsdLedgerAccountBalance, UsdLedgerAccountBalanceRange, User, UserCreateInput, UserCreatePayload, UserUpdateRoleInput, UserUpdateRolePayload, VisibleNavigationItems, Wallet, Withdrawal, WithdrawalCancelInput, WithdrawalCancelPayload, WithdrawalConfirmInput, WithdrawalConfirmPayload, WithdrawalConnection, WithdrawalEdge, WithdrawalEntry, WithdrawalInitiateInput, WithdrawalInitiatePayload, WithdrawalRevertInput, WithdrawalRevertPayload, Activity, ApprovalProcessStatus, ApprovalProcessType, CollateralAction, CollateralizationState, ConfigType, CreditFacilitiesFilterBy, CreditFacilitiesSortBy, CreditFacilityProposalStatus, CreditFacilityRepaymentStatus, CreditFacilityRepaymentType, CreditFacilityStatus, CustomerType, CustomersFilterBy, CustomersSortBy, DebitOrCredit, DepositAccountStatus, DepositStatus, DisbursalPolicy, DisbursalStatus, DocumentStatus, InterestInterval, KycLevel, KycVerification, Layer, LoanAgreementStatus, PendingCreditFacilityCollateralizationState, PendingCreditFacilityStatus, Period, PermissionSetName, ReportRunState, ReportRunType, SortDirection, WalletNetwork, WithdrawalStatus } from './index';

faker.seed(0);

export const mockAccountingCsvDocument = (overrides?: Partial<AccountingCsvDocument>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'AccountingCsvDocument' } & AccountingCsvDocument => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('AccountingCsvDocument');
    return {
        __typename: 'AccountingCsvDocument',
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        documentId: overrides && overrides.hasOwnProperty('documentId') ? overrides.documentId! : generateMockValue.uuid(),
        filename: overrides && overrides.hasOwnProperty('filename') ? overrides.filename! : generateMockValue.filename(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        ledgerAccountId: overrides && overrides.hasOwnProperty('ledgerAccountId') ? overrides.ledgerAccountId! : generateMockValue.uuid(),
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : DocumentStatus.Active,
    };
};

export const mockAccountingCsvDownloadLink = (overrides?: Partial<AccountingCsvDownloadLink>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'AccountingCsvDownloadLink' } & AccountingCsvDownloadLink => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('AccountingCsvDownloadLink');
    return {
        __typename: 'AccountingCsvDownloadLink',
        csvId: overrides && overrides.hasOwnProperty('csvId') ? overrides.csvId! : generateMockValue.uuid(),
        url: overrides && overrides.hasOwnProperty('url') ? overrides.url! : generateMockValue.url(),
    };
};

export const mockAccountingCsvDownloadLinkGenerateInput = (overrides?: Partial<AccountingCsvDownloadLinkGenerateInput>, _relationshipsToOmit: Set<string> = new Set()): AccountingCsvDownloadLinkGenerateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('AccountingCsvDownloadLinkGenerateInput');
    return {
        documentId: overrides && overrides.hasOwnProperty('documentId') ? overrides.documentId! : generateMockValue.uuid(),
    };
};

export const mockAccountingCsvDownloadLinkGeneratePayload = (overrides?: Partial<AccountingCsvDownloadLinkGeneratePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'AccountingCsvDownloadLinkGeneratePayload' } & AccountingCsvDownloadLinkGeneratePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('AccountingCsvDownloadLinkGeneratePayload');
    return {
        __typename: 'AccountingCsvDownloadLinkGeneratePayload',
        link: overrides && overrides.hasOwnProperty('link') ? overrides.link! : relationshipsToOmit.has('AccountingCsvDownloadLink') ? {} as AccountingCsvDownloadLink : mockAccountingCsvDownloadLink({}, relationshipsToOmit),
    };
};

export const mockApprovalProcess = (overrides?: Partial<ApprovalProcess>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ApprovalProcess' } & ApprovalProcess => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ApprovalProcess');
    return {
        __typename: 'ApprovalProcess',
        approvalProcessId: overrides && overrides.hasOwnProperty('approvalProcessId') ? overrides.approvalProcessId! : generateMockValue.uuid(),
        approvalProcessType: overrides && overrides.hasOwnProperty('approvalProcessType') ? overrides.approvalProcessType! : mockEnums.approvalProcessType(),
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        deniedReason: overrides && overrides.hasOwnProperty('deniedReason') ? overrides.deniedReason! : generateMockValue.deniedReason(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        policy: overrides && overrides.hasOwnProperty('policy') ? overrides.policy! : relationshipsToOmit.has('Policy') ? {} as Policy : mockPolicy({}, relationshipsToOmit),
        rules: overrides && overrides.hasOwnProperty('rules') ? overrides.rules! : relationshipsToOmit.has('CommitteeThreshold') ? {} as CommitteeThreshold : mockCommitteeThreshold({}, relationshipsToOmit),
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : mockEnums.approvalProcessStatus(),
        target: overrides && overrides.hasOwnProperty('target') ? overrides.target! : relationshipsToOmit.has('CreditFacilityDisbursal') ? {} as CreditFacilityDisbursal : mockCreditFacilityDisbursal({}, relationshipsToOmit),
        userCanSubmitDecision: overrides && overrides.hasOwnProperty('userCanSubmitDecision') ? overrides.userCanSubmitDecision! : faker.datatype.boolean(),
        voters: overrides && overrides.hasOwnProperty('voters') ? overrides.voters! : [relationshipsToOmit.has('ApprovalProcessVoter') ? {} as ApprovalProcessVoter : mockApprovalProcessVoter({}, relationshipsToOmit)],
    };
};

export const mockApprovalProcessApproveInput = (overrides?: Partial<ApprovalProcessApproveInput>, _relationshipsToOmit: Set<string> = new Set()): ApprovalProcessApproveInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ApprovalProcessApproveInput');
    return {
        processId: overrides && overrides.hasOwnProperty('processId') ? overrides.processId! : generateMockValue.uuid(),
    };
};

export const mockApprovalProcessApprovePayload = (overrides?: Partial<ApprovalProcessApprovePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ApprovalProcessApprovePayload' } & ApprovalProcessApprovePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ApprovalProcessApprovePayload');
    return {
        __typename: 'ApprovalProcessApprovePayload',
        approvalProcess: overrides && overrides.hasOwnProperty('approvalProcess') ? overrides.approvalProcess! : relationshipsToOmit.has('ApprovalProcess') ? {} as ApprovalProcess : mockApprovalProcess({}, relationshipsToOmit),
    };
};

export const mockApprovalProcessConnection = (overrides?: Partial<ApprovalProcessConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ApprovalProcessConnection' } & ApprovalProcessConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ApprovalProcessConnection');
    return {
        __typename: 'ApprovalProcessConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('ApprovalProcessEdge') ? {} as ApprovalProcessEdge : mockApprovalProcessEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('ApprovalProcess') ? {} as ApprovalProcess : mockApprovalProcess({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockApprovalProcessDenyInput = (overrides?: Partial<ApprovalProcessDenyInput>, _relationshipsToOmit: Set<string> = new Set()): ApprovalProcessDenyInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ApprovalProcessDenyInput');
    return {
        processId: overrides && overrides.hasOwnProperty('processId') ? overrides.processId! : generateMockValue.uuid(),
    };
};

export const mockApprovalProcessDenyPayload = (overrides?: Partial<ApprovalProcessDenyPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ApprovalProcessDenyPayload' } & ApprovalProcessDenyPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ApprovalProcessDenyPayload');
    return {
        __typename: 'ApprovalProcessDenyPayload',
        approvalProcess: overrides && overrides.hasOwnProperty('approvalProcess') ? overrides.approvalProcess! : relationshipsToOmit.has('ApprovalProcess') ? {} as ApprovalProcess : mockApprovalProcess({}, relationshipsToOmit),
    };
};

export const mockApprovalProcessEdge = (overrides?: Partial<ApprovalProcessEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ApprovalProcessEdge' } & ApprovalProcessEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ApprovalProcessEdge');
    return {
        __typename: 'ApprovalProcessEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('ApprovalProcess') ? {} as ApprovalProcess : mockApprovalProcess({}, relationshipsToOmit),
    };
};

export const mockApprovalProcessVoter = (overrides?: Partial<ApprovalProcessVoter>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ApprovalProcessVoter' } & ApprovalProcessVoter => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ApprovalProcessVoter');
    return {
        __typename: 'ApprovalProcessVoter',
        didApprove: overrides && overrides.hasOwnProperty('didApprove') ? overrides.didApprove! : generateMockValue.boolean(),
        didDeny: overrides && overrides.hasOwnProperty('didDeny') ? overrides.didDeny! : generateMockValue.boolean(),
        didVote: overrides && overrides.hasOwnProperty('didVote') ? overrides.didVote! : generateMockValue.boolean(),
        stillEligible: overrides && overrides.hasOwnProperty('stillEligible') ? overrides.stillEligible! : generateMockValue.boolean(),
        user: overrides && overrides.hasOwnProperty('user') ? overrides.user! : relationshipsToOmit.has('User') ? {} as User : mockUser({}, relationshipsToOmit),
        votedAt: overrides && overrides.hasOwnProperty('votedAt') ? overrides.votedAt! : generateMockValue.timestamp(),
    };
};

export const mockAuditEntry = (overrides?: Partial<AuditEntry>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'AuditEntry' } & AuditEntry => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('AuditEntry');
    return {
        __typename: 'AuditEntry',
        action: overrides && overrides.hasOwnProperty('action') ? overrides.action! : faker.lorem.word(),
        auditEntryId: overrides && overrides.hasOwnProperty('auditEntryId') ? overrides.auditEntryId! : faker.lorem.word(),
        authorized: overrides && overrides.hasOwnProperty('authorized') ? overrides.authorized! : generateMockValue.boolean(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        object: overrides && overrides.hasOwnProperty('object') ? overrides.object! : faker.lorem.word(),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        subject: overrides && overrides.hasOwnProperty('subject') ? overrides.subject! : relationshipsToOmit.has('System') ? {} as System : mockSystem({}, relationshipsToOmit),
    };
};

export const mockAuditEntryConnection = (overrides?: Partial<AuditEntryConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'AuditEntryConnection' } & AuditEntryConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('AuditEntryConnection');
    return {
        __typename: 'AuditEntryConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('AuditEntryEdge') ? {} as AuditEntryEdge : mockAuditEntryEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('AuditEntry') ? {} as AuditEntry : mockAuditEntry({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockAuditEntryEdge = (overrides?: Partial<AuditEntryEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'AuditEntryEdge' } & AuditEntryEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('AuditEntryEdge');
    return {
        __typename: 'AuditEntryEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('AuditEntry') ? {} as AuditEntry : mockAuditEntry({}, relationshipsToOmit),
    };
};

export const mockBalanceSheet = (overrides?: Partial<BalanceSheet>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'BalanceSheet' } & BalanceSheet => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('BalanceSheet');
    return {
        __typename: 'BalanceSheet',
        balance: overrides && overrides.hasOwnProperty('balance') ? overrides.balance! : relationshipsToOmit.has('BtcLedgerAccountBalanceRange') ? {} as BtcLedgerAccountBalanceRange : mockBtcLedgerAccountBalanceRange({}, relationshipsToOmit),
        categories: overrides && overrides.hasOwnProperty('categories') ? overrides.categories! : [relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit)],
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
    };
};

export const mockBalanceSheetModuleConfig = (overrides?: Partial<BalanceSheetModuleConfig>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'BalanceSheetModuleConfig' } & BalanceSheetModuleConfig => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('BalanceSheetModuleConfig');
    return {
        __typename: 'BalanceSheetModuleConfig',
        chartOfAccountsAssetsCode: overrides && overrides.hasOwnProperty('chartOfAccountsAssetsCode') ? overrides.chartOfAccountsAssetsCode! : faker.lorem.word(),
        chartOfAccountsCostOfRevenueCode: overrides && overrides.hasOwnProperty('chartOfAccountsCostOfRevenueCode') ? overrides.chartOfAccountsCostOfRevenueCode! : faker.lorem.word(),
        chartOfAccountsEquityCode: overrides && overrides.hasOwnProperty('chartOfAccountsEquityCode') ? overrides.chartOfAccountsEquityCode! : faker.lorem.word(),
        chartOfAccountsExpensesCode: overrides && overrides.hasOwnProperty('chartOfAccountsExpensesCode') ? overrides.chartOfAccountsExpensesCode! : faker.lorem.word(),
        chartOfAccountsId: overrides && overrides.hasOwnProperty('chartOfAccountsId') ? overrides.chartOfAccountsId! : generateMockValue.uuid(),
        chartOfAccountsLiabilitiesCode: overrides && overrides.hasOwnProperty('chartOfAccountsLiabilitiesCode') ? overrides.chartOfAccountsLiabilitiesCode! : faker.lorem.word(),
        chartOfAccountsRevenueCode: overrides && overrides.hasOwnProperty('chartOfAccountsRevenueCode') ? overrides.chartOfAccountsRevenueCode! : faker.lorem.word(),
    };
};

export const mockBalanceSheetModuleConfigureInput = (overrides?: Partial<BalanceSheetModuleConfigureInput>, _relationshipsToOmit: Set<string> = new Set()): BalanceSheetModuleConfigureInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('BalanceSheetModuleConfigureInput');
    return {
        chartOfAccountsAssetsCode: overrides && overrides.hasOwnProperty('chartOfAccountsAssetsCode') ? overrides.chartOfAccountsAssetsCode! : faker.lorem.word(),
        chartOfAccountsCostOfRevenueCode: overrides && overrides.hasOwnProperty('chartOfAccountsCostOfRevenueCode') ? overrides.chartOfAccountsCostOfRevenueCode! : faker.lorem.word(),
        chartOfAccountsEquityCode: overrides && overrides.hasOwnProperty('chartOfAccountsEquityCode') ? overrides.chartOfAccountsEquityCode! : faker.lorem.word(),
        chartOfAccountsExpensesCode: overrides && overrides.hasOwnProperty('chartOfAccountsExpensesCode') ? overrides.chartOfAccountsExpensesCode! : faker.lorem.word(),
        chartOfAccountsLiabilitiesCode: overrides && overrides.hasOwnProperty('chartOfAccountsLiabilitiesCode') ? overrides.chartOfAccountsLiabilitiesCode! : faker.lorem.word(),
        chartOfAccountsRevenueCode: overrides && overrides.hasOwnProperty('chartOfAccountsRevenueCode') ? overrides.chartOfAccountsRevenueCode! : faker.lorem.word(),
    };
};

export const mockBalanceSheetModuleConfigurePayload = (overrides?: Partial<BalanceSheetModuleConfigurePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'BalanceSheetModuleConfigurePayload' } & BalanceSheetModuleConfigurePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('BalanceSheetModuleConfigurePayload');
    return {
        __typename: 'BalanceSheetModuleConfigurePayload',
        balanceSheetConfig: overrides && overrides.hasOwnProperty('balanceSheetConfig') ? overrides.balanceSheetConfig! : relationshipsToOmit.has('BalanceSheetModuleConfig') ? {} as BalanceSheetModuleConfig : mockBalanceSheetModuleConfig({}, relationshipsToOmit),
    };
};

export const mockBitgoConfig = (overrides?: Partial<BitgoConfig>, _relationshipsToOmit: Set<string> = new Set()): BitgoConfig => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('BitgoConfig');
    return {
        enterpriseId: overrides && overrides.hasOwnProperty('enterpriseId') ? overrides.enterpriseId! : faker.lorem.word(),
        longLivedToken: overrides && overrides.hasOwnProperty('longLivedToken') ? overrides.longLivedToken! : faker.lorem.word(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        passphrase: overrides && overrides.hasOwnProperty('passphrase') ? overrides.passphrase! : faker.lorem.word(),
        testingInstance: overrides && overrides.hasOwnProperty('testingInstance') ? overrides.testingInstance! : faker.datatype.boolean(),
        webhookSecret: overrides && overrides.hasOwnProperty('webhookSecret') ? overrides.webhookSecret! : faker.lorem.word(),
        webhookUrl: overrides && overrides.hasOwnProperty('webhookUrl') ? overrides.webhookUrl! : faker.lorem.word(),
    };
};

export const mockBtcAmount = (overrides?: Partial<BtcAmount>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'BtcAmount' } & BtcAmount => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('BtcAmount');
    return {
        __typename: 'BtcAmount',
        btc: overrides && overrides.hasOwnProperty('btc') ? overrides.btc! : generateMockValue.satoshis(),
    };
};

export const mockBtcBalanceDetails = (overrides?: Partial<BtcBalanceDetails>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'BtcBalanceDetails' } & BtcBalanceDetails => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('BtcBalanceDetails');
    return {
        __typename: 'BtcBalanceDetails',
        credit: overrides && overrides.hasOwnProperty('credit') ? overrides.credit! : generateMockValue.satoshis(),
        debit: overrides && overrides.hasOwnProperty('debit') ? overrides.debit! : generateMockValue.satoshis(),
        net: overrides && overrides.hasOwnProperty('net') ? overrides.net! : generateMockValue.signedSatoshis(),
    };
};

export const mockBtcLedgerAccountBalance = (overrides?: Partial<BtcLedgerAccountBalance>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'BtcLedgerAccountBalance' } & BtcLedgerAccountBalance => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('BtcLedgerAccountBalance');
    return {
        __typename: 'BtcLedgerAccountBalance',
        encumbrance: overrides && overrides.hasOwnProperty('encumbrance') ? overrides.encumbrance! : relationshipsToOmit.has('BtcBalanceDetails') ? {} as BtcBalanceDetails : mockBtcBalanceDetails({}, relationshipsToOmit),
        pending: overrides && overrides.hasOwnProperty('pending') ? overrides.pending! : relationshipsToOmit.has('BtcBalanceDetails') ? {} as BtcBalanceDetails : mockBtcBalanceDetails({}, relationshipsToOmit),
        settled: overrides && overrides.hasOwnProperty('settled') ? overrides.settled! : relationshipsToOmit.has('BtcBalanceDetails') ? {} as BtcBalanceDetails : mockBtcBalanceDetails({}, relationshipsToOmit),
    };
};

export const mockBtcLedgerAccountBalanceRange = (overrides?: Partial<BtcLedgerAccountBalanceRange>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'BtcLedgerAccountBalanceRange' } & BtcLedgerAccountBalanceRange => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('BtcLedgerAccountBalanceRange');
    return {
        __typename: 'BtcLedgerAccountBalanceRange',
        close: overrides && overrides.hasOwnProperty('close') ? overrides.close! : relationshipsToOmit.has('BtcLedgerAccountBalance') ? {} as BtcLedgerAccountBalance : mockBtcLedgerAccountBalance({}, relationshipsToOmit),
        open: overrides && overrides.hasOwnProperty('open') ? overrides.open! : relationshipsToOmit.has('BtcLedgerAccountBalance') ? {} as BtcLedgerAccountBalance : mockBtcLedgerAccountBalance({}, relationshipsToOmit),
        periodActivity: overrides && overrides.hasOwnProperty('periodActivity') ? overrides.periodActivity! : relationshipsToOmit.has('BtcLedgerAccountBalance') ? {} as BtcLedgerAccountBalance : mockBtcLedgerAccountBalance({}, relationshipsToOmit),
    };
};

export const mockCancelledWithdrawalEntry = (overrides?: Partial<CancelledWithdrawalEntry>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CancelledWithdrawalEntry' } & CancelledWithdrawalEntry => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CancelledWithdrawalEntry');
    return {
        __typename: 'CancelledWithdrawalEntry',
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        withdrawal: overrides && overrides.hasOwnProperty('withdrawal') ? overrides.withdrawal! : relationshipsToOmit.has('Withdrawal') ? {} as Withdrawal : mockWithdrawal({}, relationshipsToOmit),
    };
};

export const mockChartNode = (overrides?: Partial<ChartNode>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ChartNode' } & ChartNode => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ChartNode');
    return {
        __typename: 'ChartNode',
        accountCode: overrides && overrides.hasOwnProperty('accountCode') ? overrides.accountCode! : faker.lorem.word(),
        children: overrides && overrides.hasOwnProperty('children') ? overrides.children! : [relationshipsToOmit.has('ChartNode') ? {} as ChartNode : mockChartNode({}, relationshipsToOmit)],
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
    };
};

export const mockChartOfAccounts = (overrides?: Partial<ChartOfAccounts>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ChartOfAccounts' } & ChartOfAccounts => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ChartOfAccounts');
    return {
        __typename: 'ChartOfAccounts',
        chartId: overrides && overrides.hasOwnProperty('chartId') ? overrides.chartId! : generateMockValue.uuid(),
        children: overrides && overrides.hasOwnProperty('children') ? overrides.children! : [relationshipsToOmit.has('ChartNode') ? {} as ChartNode : mockChartNode({}, relationshipsToOmit)],
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
    };
};

export const mockChartOfAccountsAddChildNodeInput = (overrides?: Partial<ChartOfAccountsAddChildNodeInput>, _relationshipsToOmit: Set<string> = new Set()): ChartOfAccountsAddChildNodeInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ChartOfAccountsAddChildNodeInput');
    return {
        code: overrides && overrides.hasOwnProperty('code') ? overrides.code! : faker.lorem.word(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        parent: overrides && overrides.hasOwnProperty('parent') ? overrides.parent! : faker.lorem.word(),
    };
};

export const mockChartOfAccountsAddChildNodePayload = (overrides?: Partial<ChartOfAccountsAddChildNodePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ChartOfAccountsAddChildNodePayload' } & ChartOfAccountsAddChildNodePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ChartOfAccountsAddChildNodePayload');
    return {
        __typename: 'ChartOfAccountsAddChildNodePayload',
        chartOfAccounts: overrides && overrides.hasOwnProperty('chartOfAccounts') ? overrides.chartOfAccounts! : relationshipsToOmit.has('ChartOfAccounts') ? {} as ChartOfAccounts : mockChartOfAccounts({}, relationshipsToOmit),
    };
};

export const mockChartOfAccountsAddRootNodeInput = (overrides?: Partial<ChartOfAccountsAddRootNodeInput>, _relationshipsToOmit: Set<string> = new Set()): ChartOfAccountsAddRootNodeInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ChartOfAccountsAddRootNodeInput');
    return {
        code: overrides && overrides.hasOwnProperty('code') ? overrides.code! : faker.lorem.word(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        normalBalanceType: overrides && overrides.hasOwnProperty('normalBalanceType') ? overrides.normalBalanceType! : DebitOrCredit.Credit,
    };
};

export const mockChartOfAccountsAddRootNodePayload = (overrides?: Partial<ChartOfAccountsAddRootNodePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ChartOfAccountsAddRootNodePayload' } & ChartOfAccountsAddRootNodePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ChartOfAccountsAddRootNodePayload');
    return {
        __typename: 'ChartOfAccountsAddRootNodePayload',
        chartOfAccounts: overrides && overrides.hasOwnProperty('chartOfAccounts') ? overrides.chartOfAccounts! : relationshipsToOmit.has('ChartOfAccounts') ? {} as ChartOfAccounts : mockChartOfAccounts({}, relationshipsToOmit),
    };
};

export const mockChartOfAccountsCsvImportInput = (overrides?: Partial<ChartOfAccountsCsvImportInput>, _relationshipsToOmit: Set<string> = new Set()): ChartOfAccountsCsvImportInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ChartOfAccountsCsvImportInput');
    return {
        file: overrides && overrides.hasOwnProperty('file') ? overrides.file! : faker.lorem.word(),
    };
};

export const mockChartOfAccountsCsvImportPayload = (overrides?: Partial<ChartOfAccountsCsvImportPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ChartOfAccountsCsvImportPayload' } & ChartOfAccountsCsvImportPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ChartOfAccountsCsvImportPayload');
    return {
        __typename: 'ChartOfAccountsCsvImportPayload',
        chartOfAccounts: overrides && overrides.hasOwnProperty('chartOfAccounts') ? overrides.chartOfAccounts! : relationshipsToOmit.has('ChartOfAccounts') ? {} as ChartOfAccounts : mockChartOfAccounts({}, relationshipsToOmit),
    };
};

export const mockCollateral = (overrides?: Partial<Collateral>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Collateral' } & Collateral => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Collateral');
    return {
        __typename: 'Collateral',
        account: overrides && overrides.hasOwnProperty('account') ? overrides.account! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        accountId: overrides && overrides.hasOwnProperty('accountId') ? overrides.accountId! : generateMockValue.uuid(),
        collateralId: overrides && overrides.hasOwnProperty('collateralId') ? overrides.collateralId! : generateMockValue.uuid(),
        creditFacility: overrides && overrides.hasOwnProperty('creditFacility') ? overrides.creditFacility! : relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        walletId: overrides && overrides.hasOwnProperty('walletId') ? overrides.walletId! : generateMockValue.uuid(),
    };
};

export const mockCollateralBalance = (overrides?: Partial<CollateralBalance>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CollateralBalance' } & CollateralBalance => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CollateralBalance');
    return {
        __typename: 'CollateralBalance',
        btcBalance: overrides && overrides.hasOwnProperty('btcBalance') ? overrides.btcBalance! : generateMockValue.satoshis(),
    };
};

export const mockCommittee = (overrides?: Partial<Committee>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Committee' } & Committee => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Committee');
    return {
        __typename: 'Committee',
        committeeId: overrides && overrides.hasOwnProperty('committeeId') ? overrides.committeeId! : generateMockValue.uuid(),
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        currentMembers: overrides && overrides.hasOwnProperty('currentMembers') ? overrides.currentMembers! : [relationshipsToOmit.has('User') ? {} as User : mockUser({}, relationshipsToOmit)],
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
    };
};

export const mockCommitteeAddUserInput = (overrides?: Partial<CommitteeAddUserInput>, _relationshipsToOmit: Set<string> = new Set()): CommitteeAddUserInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CommitteeAddUserInput');
    return {
        committeeId: overrides && overrides.hasOwnProperty('committeeId') ? overrides.committeeId! : generateMockValue.uuid(),
        userId: overrides && overrides.hasOwnProperty('userId') ? overrides.userId! : generateMockValue.uuid(),
    };
};

export const mockCommitteeAddUserPayload = (overrides?: Partial<CommitteeAddUserPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CommitteeAddUserPayload' } & CommitteeAddUserPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CommitteeAddUserPayload');
    return {
        __typename: 'CommitteeAddUserPayload',
        committee: overrides && overrides.hasOwnProperty('committee') ? overrides.committee! : relationshipsToOmit.has('Committee') ? {} as Committee : mockCommittee({}, relationshipsToOmit),
    };
};

export const mockCommitteeConnection = (overrides?: Partial<CommitteeConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CommitteeConnection' } & CommitteeConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CommitteeConnection');
    return {
        __typename: 'CommitteeConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('CommitteeEdge') ? {} as CommitteeEdge : mockCommitteeEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('Committee') ? {} as Committee : mockCommittee({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockCommitteeCreateInput = (overrides?: Partial<CommitteeCreateInput>, _relationshipsToOmit: Set<string> = new Set()): CommitteeCreateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CommitteeCreateInput');
    return {
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
    };
};

export const mockCommitteeCreatePayload = (overrides?: Partial<CommitteeCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CommitteeCreatePayload' } & CommitteeCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CommitteeCreatePayload');
    return {
        __typename: 'CommitteeCreatePayload',
        committee: overrides && overrides.hasOwnProperty('committee') ? overrides.committee! : relationshipsToOmit.has('Committee') ? {} as Committee : mockCommittee({}, relationshipsToOmit),
    };
};

export const mockCommitteeEdge = (overrides?: Partial<CommitteeEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CommitteeEdge' } & CommitteeEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CommitteeEdge');
    return {
        __typename: 'CommitteeEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('Committee') ? {} as Committee : mockCommittee({}, relationshipsToOmit),
    };
};

export const mockCommitteeRemoveUserInput = (overrides?: Partial<CommitteeRemoveUserInput>, _relationshipsToOmit: Set<string> = new Set()): CommitteeRemoveUserInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CommitteeRemoveUserInput');
    return {
        committeeId: overrides && overrides.hasOwnProperty('committeeId') ? overrides.committeeId! : generateMockValue.uuid(),
        userId: overrides && overrides.hasOwnProperty('userId') ? overrides.userId! : generateMockValue.uuid(),
    };
};

export const mockCommitteeRemoveUserPayload = (overrides?: Partial<CommitteeRemoveUserPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CommitteeRemoveUserPayload' } & CommitteeRemoveUserPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CommitteeRemoveUserPayload');
    return {
        __typename: 'CommitteeRemoveUserPayload',
        committee: overrides && overrides.hasOwnProperty('committee') ? overrides.committee! : relationshipsToOmit.has('Committee') ? {} as Committee : mockCommittee({}, relationshipsToOmit),
    };
};

export const mockCommitteeThreshold = (overrides?: Partial<CommitteeThreshold>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CommitteeThreshold' } & CommitteeThreshold => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CommitteeThreshold');
    return {
        __typename: 'CommitteeThreshold',
        committee: overrides && overrides.hasOwnProperty('committee') ? overrides.committee! : relationshipsToOmit.has('Committee') ? {} as Committee : mockCommittee({}, relationshipsToOmit),
        threshold: overrides && overrides.hasOwnProperty('threshold') ? overrides.threshold! : faker.number.int({ min: 0, max: 9999 }),
    };
};

export const mockCreditFacilitiesFilter = (overrides?: Partial<CreditFacilitiesFilter>, _relationshipsToOmit: Set<string> = new Set()): CreditFacilitiesFilter => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilitiesFilter');
    return {
        collateralizationState: overrides && overrides.hasOwnProperty('collateralizationState') ? overrides.collateralizationState! : mockEnums.collateralizationState(),
        field: overrides && overrides.hasOwnProperty('field') ? overrides.field! : CreditFacilitiesFilterBy.CollateralizationState,
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : mockEnums.creditFacilityStatus(),
    };
};

export const mockCreditFacilitiesSort = (overrides?: Partial<CreditFacilitiesSort>, _relationshipsToOmit: Set<string> = new Set()): CreditFacilitiesSort => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilitiesSort');
    return {
        by: overrides && overrides.hasOwnProperty('by') ? overrides.by! : CreditFacilitiesSortBy.CreatedAt,
        direction: overrides && overrides.hasOwnProperty('direction') ? overrides.direction! : SortDirection.Asc,
    };
};

export const mockCreditFacility = (overrides?: Partial<CreditFacility>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacility' } & CreditFacility => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacility');
    return {
        __typename: 'CreditFacility',
        activatedAt: overrides && overrides.hasOwnProperty('activatedAt') ? overrides.activatedAt! : generateMockValue.timestamp(),
        balance: overrides && overrides.hasOwnProperty('balance') ? overrides.balance! : relationshipsToOmit.has('CreditFacilityBalance') ? {} as CreditFacilityBalance : mockCreditFacilityBalance({}, relationshipsToOmit),
        canBeCompleted: overrides && overrides.hasOwnProperty('canBeCompleted') ? overrides.canBeCompleted! : generateMockValue.boolean(),
        collateralToMatchInitialCvl: overrides && overrides.hasOwnProperty('collateralToMatchInitialCvl') ? overrides.collateralToMatchInitialCvl! : generateMockValue.satoshis(),
        collateralizationState: overrides && overrides.hasOwnProperty('collateralizationState') ? overrides.collateralizationState! : mockEnums.collateralizationState(),
        creditFacilityId: overrides && overrides.hasOwnProperty('creditFacilityId') ? overrides.creditFacilityId! : generateMockValue.uuid(),
        creditFacilityTerms: overrides && overrides.hasOwnProperty('creditFacilityTerms') ? overrides.creditFacilityTerms! : relationshipsToOmit.has('TermValues') ? {} as TermValues : mockTermValues({}, relationshipsToOmit),
        currentCvl: overrides && overrides.hasOwnProperty('currentCvl') ? overrides.currentCvl! : relationshipsToOmit.has('FiniteCvlPct') ? {} as FiniteCvlPct : mockFiniteCvlPct({}, relationshipsToOmit),
        customer: overrides && overrides.hasOwnProperty('customer') ? overrides.customer! : relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit),
        disbursals: overrides && overrides.hasOwnProperty('disbursals') ? overrides.disbursals! : [relationshipsToOmit.has('CreditFacilityDisbursal') ? {} as CreditFacilityDisbursal : mockCreditFacilityDisbursal({}, relationshipsToOmit)],
        facilityAmount: overrides && overrides.hasOwnProperty('facilityAmount') ? overrides.facilityAmount! : generateMockValue.usdCents(),
        history: overrides && overrides.hasOwnProperty('history') ? overrides.history! : [relationshipsToOmit.has('CreditFacilityApproved') ? {} as CreditFacilityApproved : mockCreditFacilityApproved({}, relationshipsToOmit)],
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        ledgerAccounts: overrides && overrides.hasOwnProperty('ledgerAccounts') ? overrides.ledgerAccounts! : relationshipsToOmit.has('CreditFacilityLedgerAccounts') ? {} as CreditFacilityLedgerAccounts : mockCreditFacilityLedgerAccounts({}, relationshipsToOmit),
        liquidations: overrides && overrides.hasOwnProperty('liquidations') ? overrides.liquidations! : [relationshipsToOmit.has('Liquidation') ? {} as Liquidation : mockLiquidation({}, relationshipsToOmit)],
        maturesAt: overrides && overrides.hasOwnProperty('maturesAt') ? overrides.maturesAt! : generateMockValue.timestamp(),
        publicId: overrides && overrides.hasOwnProperty('publicId') ? overrides.publicId! : faker.lorem.word(),
        repaymentPlan: overrides && overrides.hasOwnProperty('repaymentPlan') ? overrides.repaymentPlan! : [relationshipsToOmit.has('CreditFacilityRepaymentPlanEntry') ? {} as CreditFacilityRepaymentPlanEntry : mockCreditFacilityRepaymentPlanEntry({}, relationshipsToOmit)],
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : mockEnums.creditFacilityStatus(),
        userCanComplete: overrides && overrides.hasOwnProperty('userCanComplete') ? overrides.userCanComplete! : faker.datatype.boolean(),
        userCanInitiateDisbursal: overrides && overrides.hasOwnProperty('userCanInitiateDisbursal') ? overrides.userCanInitiateDisbursal! : faker.datatype.boolean(),
        userCanRecordPayment: overrides && overrides.hasOwnProperty('userCanRecordPayment') ? overrides.userCanRecordPayment! : faker.datatype.boolean(),
        userCanRecordPaymentWithDate: overrides && overrides.hasOwnProperty('userCanRecordPaymentWithDate') ? overrides.userCanRecordPaymentWithDate! : faker.datatype.boolean(),
        userCanUpdateCollateral: overrides && overrides.hasOwnProperty('userCanUpdateCollateral') ? overrides.userCanUpdateCollateral! : faker.datatype.boolean(),
        wallet: overrides && overrides.hasOwnProperty('wallet') ? overrides.wallet! : relationshipsToOmit.has('Wallet') ? {} as Wallet : mockWallet({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityApproved = (overrides?: Partial<CreditFacilityApproved>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityApproved' } & CreditFacilityApproved => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityApproved');
    return {
        __typename: 'CreditFacilityApproved',
        cents: overrides && overrides.hasOwnProperty('cents') ? overrides.cents! : generateMockValue.usdCents(),
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        txId: overrides && overrides.hasOwnProperty('txId') ? overrides.txId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityBalance = (overrides?: Partial<CreditFacilityBalance>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityBalance' } & CreditFacilityBalance => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityBalance');
    return {
        __typename: 'CreditFacilityBalance',
        collateral: overrides && overrides.hasOwnProperty('collateral') ? overrides.collateral! : relationshipsToOmit.has('CollateralBalance') ? {} as CollateralBalance : mockCollateralBalance({}, relationshipsToOmit),
        disbursed: overrides && overrides.hasOwnProperty('disbursed') ? overrides.disbursed! : relationshipsToOmit.has('Disbursed') ? {} as Disbursed : mockDisbursed({}, relationshipsToOmit),
        dueOutstanding: overrides && overrides.hasOwnProperty('dueOutstanding') ? overrides.dueOutstanding! : relationshipsToOmit.has('Outstanding') ? {} as Outstanding : mockOutstanding({}, relationshipsToOmit),
        facilityRemaining: overrides && overrides.hasOwnProperty('facilityRemaining') ? overrides.facilityRemaining! : relationshipsToOmit.has('FacilityRemaining') ? {} as FacilityRemaining : mockFacilityRemaining({}, relationshipsToOmit),
        interest: overrides && overrides.hasOwnProperty('interest') ? overrides.interest! : relationshipsToOmit.has('Interest') ? {} as Interest : mockInterest({}, relationshipsToOmit),
        outstanding: overrides && overrides.hasOwnProperty('outstanding') ? overrides.outstanding! : relationshipsToOmit.has('Outstanding') ? {} as Outstanding : mockOutstanding({}, relationshipsToOmit),
        outstandingPayable: overrides && overrides.hasOwnProperty('outstandingPayable') ? overrides.outstandingPayable! : relationshipsToOmit.has('Outstanding') ? {} as Outstanding : mockOutstanding({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityCollateralSentOut = (overrides?: Partial<CreditFacilityCollateralSentOut>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityCollateralSentOut' } & CreditFacilityCollateralSentOut => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityCollateralSentOut');
    return {
        __typename: 'CreditFacilityCollateralSentOut',
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.satoshis(),
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        txId: overrides && overrides.hasOwnProperty('txId') ? overrides.txId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityCollateralUpdateInput = (overrides?: Partial<CreditFacilityCollateralUpdateInput>, _relationshipsToOmit: Set<string> = new Set()): CreditFacilityCollateralUpdateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityCollateralUpdateInput');
    return {
        collateral: overrides && overrides.hasOwnProperty('collateral') ? overrides.collateral! : generateMockValue.satoshis(),
        creditFacilityId: overrides && overrides.hasOwnProperty('creditFacilityId') ? overrides.creditFacilityId! : generateMockValue.uuid(),
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
    };
};

export const mockCreditFacilityCollateralUpdatePayload = (overrides?: Partial<CreditFacilityCollateralUpdatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityCollateralUpdatePayload' } & CreditFacilityCollateralUpdatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityCollateralUpdatePayload');
    return {
        __typename: 'CreditFacilityCollateralUpdatePayload',
        creditFacility: overrides && overrides.hasOwnProperty('creditFacility') ? overrides.creditFacility! : relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityCollateralUpdated = (overrides?: Partial<CreditFacilityCollateralUpdated>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityCollateralUpdated' } & CreditFacilityCollateralUpdated => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityCollateralUpdated');
    return {
        __typename: 'CreditFacilityCollateralUpdated',
        action: overrides && overrides.hasOwnProperty('action') ? overrides.action! : CollateralAction.Add,
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        satoshis: overrides && overrides.hasOwnProperty('satoshis') ? overrides.satoshis! : generateMockValue.satoshis(),
        txId: overrides && overrides.hasOwnProperty('txId') ? overrides.txId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityCollateralizationUpdated = (overrides?: Partial<CreditFacilityCollateralizationUpdated>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityCollateralizationUpdated' } & CreditFacilityCollateralizationUpdated => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityCollateralizationUpdated');
    return {
        __typename: 'CreditFacilityCollateralizationUpdated',
        collateral: overrides && overrides.hasOwnProperty('collateral') ? overrides.collateral! : generateMockValue.satoshis(),
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        outstandingDisbursal: overrides && overrides.hasOwnProperty('outstandingDisbursal') ? overrides.outstandingDisbursal! : generateMockValue.usdCents(),
        outstandingInterest: overrides && overrides.hasOwnProperty('outstandingInterest') ? overrides.outstandingInterest! : generateMockValue.usdCents(),
        price: overrides && overrides.hasOwnProperty('price') ? overrides.price! : generateMockValue.usdCents(),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        state: overrides && overrides.hasOwnProperty('state') ? overrides.state! : CollateralizationState.FullyCollateralized,
    };
};

export const mockCreditFacilityCompleteInput = (overrides?: Partial<CreditFacilityCompleteInput>, _relationshipsToOmit: Set<string> = new Set()): CreditFacilityCompleteInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityCompleteInput');
    return {
        creditFacilityId: overrides && overrides.hasOwnProperty('creditFacilityId') ? overrides.creditFacilityId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityCompletePayload = (overrides?: Partial<CreditFacilityCompletePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityCompletePayload' } & CreditFacilityCompletePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityCompletePayload');
    return {
        __typename: 'CreditFacilityCompletePayload',
        creditFacility: overrides && overrides.hasOwnProperty('creditFacility') ? overrides.creditFacility! : relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityConnection = (overrides?: Partial<CreditFacilityConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityConnection' } & CreditFacilityConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityConnection');
    return {
        __typename: 'CreditFacilityConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('CreditFacilityEdge') ? {} as CreditFacilityEdge : mockCreditFacilityEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityDisbursal = (overrides?: Partial<CreditFacilityDisbursal>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityDisbursal' } & CreditFacilityDisbursal => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityDisbursal');
    return {
        __typename: 'CreditFacilityDisbursal',
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.usdCents(),
        approvalProcess: overrides && overrides.hasOwnProperty('approvalProcess') ? overrides.approvalProcess! : relationshipsToOmit.has('ApprovalProcess') ? {} as ApprovalProcess : mockApprovalProcess({}, relationshipsToOmit),
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        creditFacility: overrides && overrides.hasOwnProperty('creditFacility') ? overrides.creditFacility! : relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit),
        disbursalId: overrides && overrides.hasOwnProperty('disbursalId') ? overrides.disbursalId! : generateMockValue.uuid(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        ledgerTransactions: overrides && overrides.hasOwnProperty('ledgerTransactions') ? overrides.ledgerTransactions! : [relationshipsToOmit.has('LedgerTransaction') ? {} as LedgerTransaction : mockLedgerTransaction({}, relationshipsToOmit)],
        publicId: overrides && overrides.hasOwnProperty('publicId') ? overrides.publicId! : faker.lorem.word(),
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : DisbursalStatus.Approved,
    };
};

export const mockCreditFacilityDisbursalConnection = (overrides?: Partial<CreditFacilityDisbursalConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityDisbursalConnection' } & CreditFacilityDisbursalConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityDisbursalConnection');
    return {
        __typename: 'CreditFacilityDisbursalConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('CreditFacilityDisbursalEdge') ? {} as CreditFacilityDisbursalEdge : mockCreditFacilityDisbursalEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('CreditFacilityDisbursal') ? {} as CreditFacilityDisbursal : mockCreditFacilityDisbursal({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityDisbursalEdge = (overrides?: Partial<CreditFacilityDisbursalEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityDisbursalEdge' } & CreditFacilityDisbursalEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityDisbursalEdge');
    return {
        __typename: 'CreditFacilityDisbursalEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('CreditFacilityDisbursal') ? {} as CreditFacilityDisbursal : mockCreditFacilityDisbursal({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityDisbursalExecuted = (overrides?: Partial<CreditFacilityDisbursalExecuted>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityDisbursalExecuted' } & CreditFacilityDisbursalExecuted => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityDisbursalExecuted');
    return {
        __typename: 'CreditFacilityDisbursalExecuted',
        cents: overrides && overrides.hasOwnProperty('cents') ? overrides.cents! : generateMockValue.usdCents(),
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        txId: overrides && overrides.hasOwnProperty('txId') ? overrides.txId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityDisbursalInitiateInput = (overrides?: Partial<CreditFacilityDisbursalInitiateInput>, _relationshipsToOmit: Set<string> = new Set()): CreditFacilityDisbursalInitiateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityDisbursalInitiateInput');
    return {
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.usdCents(),
        creditFacilityId: overrides && overrides.hasOwnProperty('creditFacilityId') ? overrides.creditFacilityId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityDisbursalInitiatePayload = (overrides?: Partial<CreditFacilityDisbursalInitiatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityDisbursalInitiatePayload' } & CreditFacilityDisbursalInitiatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityDisbursalInitiatePayload');
    return {
        __typename: 'CreditFacilityDisbursalInitiatePayload',
        disbursal: overrides && overrides.hasOwnProperty('disbursal') ? overrides.disbursal! : relationshipsToOmit.has('CreditFacilityDisbursal') ? {} as CreditFacilityDisbursal : mockCreditFacilityDisbursal({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityEdge = (overrides?: Partial<CreditFacilityEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityEdge' } & CreditFacilityEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityEdge');
    return {
        __typename: 'CreditFacilityEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityIncrementalPayment = (overrides?: Partial<CreditFacilityIncrementalPayment>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityIncrementalPayment' } & CreditFacilityIncrementalPayment => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityIncrementalPayment');
    return {
        __typename: 'CreditFacilityIncrementalPayment',
        cents: overrides && overrides.hasOwnProperty('cents') ? overrides.cents! : generateMockValue.usdCents(),
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        txId: overrides && overrides.hasOwnProperty('txId') ? overrides.txId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityInterestAccrued = (overrides?: Partial<CreditFacilityInterestAccrued>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityInterestAccrued' } & CreditFacilityInterestAccrued => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityInterestAccrued');
    return {
        __typename: 'CreditFacilityInterestAccrued',
        cents: overrides && overrides.hasOwnProperty('cents') ? overrides.cents! : generateMockValue.usdCents(),
        days: overrides && overrides.hasOwnProperty('days') ? overrides.days! : faker.number.int({ min: 0, max: 9999 }),
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        txId: overrides && overrides.hasOwnProperty('txId') ? overrides.txId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityLedgerAccounts = (overrides?: Partial<CreditFacilityLedgerAccounts>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityLedgerAccounts' } & CreditFacilityLedgerAccounts => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityLedgerAccounts');
    return {
        __typename: 'CreditFacilityLedgerAccounts',
        collateralAccount: overrides && overrides.hasOwnProperty('collateralAccount') ? overrides.collateralAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        collateralAccountId: overrides && overrides.hasOwnProperty('collateralAccountId') ? overrides.collateralAccountId! : generateMockValue.uuid(),
        collateralInLiquidationAccount: overrides && overrides.hasOwnProperty('collateralInLiquidationAccount') ? overrides.collateralInLiquidationAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        collateralInLiquidationAccountId: overrides && overrides.hasOwnProperty('collateralInLiquidationAccountId') ? overrides.collateralInLiquidationAccountId! : generateMockValue.uuid(),
        disbursedDefaultedAccount: overrides && overrides.hasOwnProperty('disbursedDefaultedAccount') ? overrides.disbursedDefaultedAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        disbursedDefaultedAccountId: overrides && overrides.hasOwnProperty('disbursedDefaultedAccountId') ? overrides.disbursedDefaultedAccountId! : generateMockValue.uuid(),
        disbursedReceivableDueAccount: overrides && overrides.hasOwnProperty('disbursedReceivableDueAccount') ? overrides.disbursedReceivableDueAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        disbursedReceivableDueAccountId: overrides && overrides.hasOwnProperty('disbursedReceivableDueAccountId') ? overrides.disbursedReceivableDueAccountId! : generateMockValue.uuid(),
        disbursedReceivableNotYetDueAccount: overrides && overrides.hasOwnProperty('disbursedReceivableNotYetDueAccount') ? overrides.disbursedReceivableNotYetDueAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        disbursedReceivableNotYetDueAccountId: overrides && overrides.hasOwnProperty('disbursedReceivableNotYetDueAccountId') ? overrides.disbursedReceivableNotYetDueAccountId! : generateMockValue.uuid(),
        disbursedReceivableOverdueAccount: overrides && overrides.hasOwnProperty('disbursedReceivableOverdueAccount') ? overrides.disbursedReceivableOverdueAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        disbursedReceivableOverdueAccountId: overrides && overrides.hasOwnProperty('disbursedReceivableOverdueAccountId') ? overrides.disbursedReceivableOverdueAccountId! : generateMockValue.uuid(),
        facilityAccount: overrides && overrides.hasOwnProperty('facilityAccount') ? overrides.facilityAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        facilityAccountId: overrides && overrides.hasOwnProperty('facilityAccountId') ? overrides.facilityAccountId! : generateMockValue.uuid(),
        feeIncomeAccount: overrides && overrides.hasOwnProperty('feeIncomeAccount') ? overrides.feeIncomeAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        feeIncomeAccountId: overrides && overrides.hasOwnProperty('feeIncomeAccountId') ? overrides.feeIncomeAccountId! : generateMockValue.uuid(),
        interestDefaultedAccount: overrides && overrides.hasOwnProperty('interestDefaultedAccount') ? overrides.interestDefaultedAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        interestDefaultedAccountId: overrides && overrides.hasOwnProperty('interestDefaultedAccountId') ? overrides.interestDefaultedAccountId! : generateMockValue.uuid(),
        interestIncomeAccount: overrides && overrides.hasOwnProperty('interestIncomeAccount') ? overrides.interestIncomeAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        interestIncomeAccountId: overrides && overrides.hasOwnProperty('interestIncomeAccountId') ? overrides.interestIncomeAccountId! : generateMockValue.uuid(),
        interestReceivableDueAccount: overrides && overrides.hasOwnProperty('interestReceivableDueAccount') ? overrides.interestReceivableDueAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        interestReceivableDueAccountId: overrides && overrides.hasOwnProperty('interestReceivableDueAccountId') ? overrides.interestReceivableDueAccountId! : generateMockValue.uuid(),
        interestReceivableNotYetDueAccount: overrides && overrides.hasOwnProperty('interestReceivableNotYetDueAccount') ? overrides.interestReceivableNotYetDueAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        interestReceivableNotYetDueAccountId: overrides && overrides.hasOwnProperty('interestReceivableNotYetDueAccountId') ? overrides.interestReceivableNotYetDueAccountId! : generateMockValue.uuid(),
        interestReceivableOverdueAccount: overrides && overrides.hasOwnProperty('interestReceivableOverdueAccount') ? overrides.interestReceivableOverdueAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        interestReceivableOverdueAccountId: overrides && overrides.hasOwnProperty('interestReceivableOverdueAccountId') ? overrides.interestReceivableOverdueAccountId! : generateMockValue.uuid(),
        liquidationPaymentReceivableAccount: overrides && overrides.hasOwnProperty('liquidationPaymentReceivableAccount') ? overrides.liquidationPaymentReceivableAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        liquidationPaymentReceivableAccountId: overrides && overrides.hasOwnProperty('liquidationPaymentReceivableAccountId') ? overrides.liquidationPaymentReceivableAccountId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityPartialPaymentRecordInput = (overrides?: Partial<CreditFacilityPartialPaymentRecordInput>, _relationshipsToOmit: Set<string> = new Set()): CreditFacilityPartialPaymentRecordInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityPartialPaymentRecordInput');
    return {
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.usdCents(),
        creditFacilityId: overrides && overrides.hasOwnProperty('creditFacilityId') ? overrides.creditFacilityId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityPartialPaymentRecordPayload = (overrides?: Partial<CreditFacilityPartialPaymentRecordPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityPartialPaymentRecordPayload' } & CreditFacilityPartialPaymentRecordPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityPartialPaymentRecordPayload');
    return {
        __typename: 'CreditFacilityPartialPaymentRecordPayload',
        creditFacility: overrides && overrides.hasOwnProperty('creditFacility') ? overrides.creditFacility! : relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityPartialPaymentWithDateRecordInput = (overrides?: Partial<CreditFacilityPartialPaymentWithDateRecordInput>, _relationshipsToOmit: Set<string> = new Set()): CreditFacilityPartialPaymentWithDateRecordInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityPartialPaymentWithDateRecordInput');
    return {
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.usdCents(),
        creditFacilityId: overrides && overrides.hasOwnProperty('creditFacilityId') ? overrides.creditFacilityId! : generateMockValue.uuid(),
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
    };
};

export const mockCreditFacilityPaymentAllocation = (overrides?: Partial<CreditFacilityPaymentAllocation>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityPaymentAllocation' } & CreditFacilityPaymentAllocation => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityPaymentAllocation');
    return {
        __typename: 'CreditFacilityPaymentAllocation',
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.usdCents(),
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        creditFacility: overrides && overrides.hasOwnProperty('creditFacility') ? overrides.creditFacility! : relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        paymentAllocationId: overrides && overrides.hasOwnProperty('paymentAllocationId') ? overrides.paymentAllocationId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityProposal = (overrides?: Partial<CreditFacilityProposal>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityProposal' } & CreditFacilityProposal => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityProposal');
    return {
        __typename: 'CreditFacilityProposal',
        approvalProcess: overrides && overrides.hasOwnProperty('approvalProcess') ? overrides.approvalProcess! : relationshipsToOmit.has('ApprovalProcess') ? {} as ApprovalProcess : mockApprovalProcess({}, relationshipsToOmit),
        approvalProcessId: overrides && overrides.hasOwnProperty('approvalProcessId') ? overrides.approvalProcessId! : generateMockValue.uuid(),
        collateralToMatchInitialCvl: overrides && overrides.hasOwnProperty('collateralToMatchInitialCvl') ? overrides.collateralToMatchInitialCvl! : generateMockValue.satoshis(),
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        creditFacilityProposalId: overrides && overrides.hasOwnProperty('creditFacilityProposalId') ? overrides.creditFacilityProposalId! : generateMockValue.uuid(),
        creditFacilityTerms: overrides && overrides.hasOwnProperty('creditFacilityTerms') ? overrides.creditFacilityTerms! : relationshipsToOmit.has('TermValues') ? {} as TermValues : mockTermValues({}, relationshipsToOmit),
        custodian: overrides && overrides.hasOwnProperty('custodian') ? overrides.custodian! : relationshipsToOmit.has('Custodian') ? {} as Custodian : mockCustodian({}, relationshipsToOmit),
        customer: overrides && overrides.hasOwnProperty('customer') ? overrides.customer! : relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit),
        facilityAmount: overrides && overrides.hasOwnProperty('facilityAmount') ? overrides.facilityAmount! : generateMockValue.usdCents(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        repaymentPlan: overrides && overrides.hasOwnProperty('repaymentPlan') ? overrides.repaymentPlan! : [relationshipsToOmit.has('CreditFacilityRepaymentPlanEntry') ? {} as CreditFacilityRepaymentPlanEntry : mockCreditFacilityRepaymentPlanEntry({}, relationshipsToOmit)],
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : CreditFacilityProposalStatus.Approved,
    };
};

export const mockCreditFacilityProposalConnection = (overrides?: Partial<CreditFacilityProposalConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityProposalConnection' } & CreditFacilityProposalConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityProposalConnection');
    return {
        __typename: 'CreditFacilityProposalConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('CreditFacilityProposalEdge') ? {} as CreditFacilityProposalEdge : mockCreditFacilityProposalEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('CreditFacilityProposal') ? {} as CreditFacilityProposal : mockCreditFacilityProposal({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityProposalCreateInput = (overrides?: Partial<CreditFacilityProposalCreateInput>, _relationshipsToOmit: Set<string> = new Set()): CreditFacilityProposalCreateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityProposalCreateInput');
    return {
        custodianId: overrides && overrides.hasOwnProperty('custodianId') ? overrides.custodianId! : generateMockValue.uuid(),
        customerId: overrides && overrides.hasOwnProperty('customerId') ? overrides.customerId! : generateMockValue.uuid(),
        facility: overrides && overrides.hasOwnProperty('facility') ? overrides.facility! : generateMockValue.usdCents(),
        terms: overrides && overrides.hasOwnProperty('terms') ? overrides.terms! : relationshipsToOmit.has('TermsInput') ? {} as TermsInput : mockTermsInput({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityProposalCreatePayload = (overrides?: Partial<CreditFacilityProposalCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityProposalCreatePayload' } & CreditFacilityProposalCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityProposalCreatePayload');
    return {
        __typename: 'CreditFacilityProposalCreatePayload',
        creditFacilityProposal: overrides && overrides.hasOwnProperty('creditFacilityProposal') ? overrides.creditFacilityProposal! : relationshipsToOmit.has('CreditFacilityProposal') ? {} as CreditFacilityProposal : mockCreditFacilityProposal({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityProposalCustomerApprovalConcludeInput = (overrides?: Partial<CreditFacilityProposalCustomerApprovalConcludeInput>, _relationshipsToOmit: Set<string> = new Set()): CreditFacilityProposalCustomerApprovalConcludeInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityProposalCustomerApprovalConcludeInput');
    return {
        approved: overrides && overrides.hasOwnProperty('approved') ? overrides.approved! : faker.datatype.boolean(),
        creditFacilityProposalId: overrides && overrides.hasOwnProperty('creditFacilityProposalId') ? overrides.creditFacilityProposalId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityProposalCustomerApprovalConcludePayload = (overrides?: Partial<CreditFacilityProposalCustomerApprovalConcludePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityProposalCustomerApprovalConcludePayload' } & CreditFacilityProposalCustomerApprovalConcludePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityProposalCustomerApprovalConcludePayload');
    return {
        __typename: 'CreditFacilityProposalCustomerApprovalConcludePayload',
        creditFacilityProposal: overrides && overrides.hasOwnProperty('creditFacilityProposal') ? overrides.creditFacilityProposal! : relationshipsToOmit.has('CreditFacilityProposal') ? {} as CreditFacilityProposal : mockCreditFacilityProposal({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityProposalEdge = (overrides?: Partial<CreditFacilityProposalEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityProposalEdge' } & CreditFacilityProposalEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityProposalEdge');
    return {
        __typename: 'CreditFacilityProposalEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('CreditFacilityProposal') ? {} as CreditFacilityProposal : mockCreditFacilityProposal({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityRepaymentAmountReceived = (overrides?: Partial<CreditFacilityRepaymentAmountReceived>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityRepaymentAmountReceived' } & CreditFacilityRepaymentAmountReceived => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityRepaymentAmountReceived');
    return {
        __typename: 'CreditFacilityRepaymentAmountReceived',
        cents: overrides && overrides.hasOwnProperty('cents') ? overrides.cents! : generateMockValue.usdCents(),
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        txId: overrides && overrides.hasOwnProperty('txId') ? overrides.txId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityRepaymentPlanEntry = (overrides?: Partial<CreditFacilityRepaymentPlanEntry>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityRepaymentPlanEntry' } & CreditFacilityRepaymentPlanEntry => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityRepaymentPlanEntry');
    return {
        __typename: 'CreditFacilityRepaymentPlanEntry',
        accrualAt: overrides && overrides.hasOwnProperty('accrualAt') ? overrides.accrualAt! : generateMockValue.timestamp(),
        dueAt: overrides && overrides.hasOwnProperty('dueAt') ? overrides.dueAt! : generateMockValue.timestamp(),
        initial: overrides && overrides.hasOwnProperty('initial') ? overrides.initial! : generateMockValue.usdCents(),
        outstanding: overrides && overrides.hasOwnProperty('outstanding') ? overrides.outstanding! : generateMockValue.usdCents(),
        repaymentType: overrides && overrides.hasOwnProperty('repaymentType') ? overrides.repaymentType! : CreditFacilityRepaymentType.Disbursal,
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : CreditFacilityRepaymentStatus.Defaulted,
    };
};

export const mockCreditModuleConfig = (overrides?: Partial<CreditModuleConfig>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditModuleConfig' } & CreditModuleConfig => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditModuleConfig');
    return {
        __typename: 'CreditModuleConfig',
        chartOfAccountCollateralOmnibusParentCode: overrides && overrides.hasOwnProperty('chartOfAccountCollateralOmnibusParentCode') ? overrides.chartOfAccountCollateralOmnibusParentCode! : faker.lorem.word(),
        chartOfAccountCollateralParentCode: overrides && overrides.hasOwnProperty('chartOfAccountCollateralParentCode') ? overrides.chartOfAccountCollateralParentCode! : faker.lorem.word(),
        chartOfAccountFacilityOmnibusParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFacilityOmnibusParentCode') ? overrides.chartOfAccountFacilityOmnibusParentCode! : faker.lorem.word(),
        chartOfAccountFacilityParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFacilityParentCode') ? overrides.chartOfAccountFacilityParentCode! : faker.lorem.word(),
        chartOfAccountFeeIncomeParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFeeIncomeParentCode') ? overrides.chartOfAccountFeeIncomeParentCode! : faker.lorem.word(),
        chartOfAccountInLiquidationOmnibusParentCode: overrides && overrides.hasOwnProperty('chartOfAccountInLiquidationOmnibusParentCode') ? overrides.chartOfAccountInLiquidationOmnibusParentCode! : faker.lorem.word(),
        chartOfAccountInLiquidationParentCode: overrides && overrides.hasOwnProperty('chartOfAccountInLiquidationParentCode') ? overrides.chartOfAccountInLiquidationParentCode! : faker.lorem.word(),
        chartOfAccountInterestIncomeParentCode: overrides && overrides.hasOwnProperty('chartOfAccountInterestIncomeParentCode') ? overrides.chartOfAccountInterestIncomeParentCode! : faker.lorem.word(),
        chartOfAccountLongTermBankDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermBankDisbursedReceivableParentCode') ? overrides.chartOfAccountLongTermBankDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermBankInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermBankInterestReceivableParentCode') ? overrides.chartOfAccountLongTermBankInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermFinancialInstitutionDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermFinancialInstitutionDisbursedReceivableParentCode') ? overrides.chartOfAccountLongTermFinancialInstitutionDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermFinancialInstitutionInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermFinancialInstitutionInterestReceivableParentCode') ? overrides.chartOfAccountLongTermFinancialInstitutionInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode') ? overrides.chartOfAccountLongTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermForeignAgencyOrSubsidiaryInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermForeignAgencyOrSubsidiaryInterestReceivableParentCode') ? overrides.chartOfAccountLongTermForeignAgencyOrSubsidiaryInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermGovernmentEntityDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermGovernmentEntityDisbursedReceivableParentCode') ? overrides.chartOfAccountLongTermGovernmentEntityDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermGovernmentEntityInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermGovernmentEntityInterestReceivableParentCode') ? overrides.chartOfAccountLongTermGovernmentEntityInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermIndividualDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermIndividualDisbursedReceivableParentCode') ? overrides.chartOfAccountLongTermIndividualDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermIndividualInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermIndividualInterestReceivableParentCode') ? overrides.chartOfAccountLongTermIndividualInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermNonDomiciledCompanyDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermNonDomiciledCompanyDisbursedReceivableParentCode') ? overrides.chartOfAccountLongTermNonDomiciledCompanyDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermNonDomiciledCompanyInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermNonDomiciledCompanyInterestReceivableParentCode') ? overrides.chartOfAccountLongTermNonDomiciledCompanyInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermPrivateCompanyDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermPrivateCompanyDisbursedReceivableParentCode') ? overrides.chartOfAccountLongTermPrivateCompanyDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermPrivateCompanyInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermPrivateCompanyInterestReceivableParentCode') ? overrides.chartOfAccountLongTermPrivateCompanyInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountOverdueBankDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountOverdueBankDisbursedReceivableParentCode') ? overrides.chartOfAccountOverdueBankDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountOverdueFinancialInstitutionDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountOverdueFinancialInstitutionDisbursedReceivableParentCode') ? overrides.chartOfAccountOverdueFinancialInstitutionDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountOverdueForeignAgencyOrSubsidiaryDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountOverdueForeignAgencyOrSubsidiaryDisbursedReceivableParentCode') ? overrides.chartOfAccountOverdueForeignAgencyOrSubsidiaryDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountOverdueGovernmentEntityDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountOverdueGovernmentEntityDisbursedReceivableParentCode') ? overrides.chartOfAccountOverdueGovernmentEntityDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountOverdueIndividualDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountOverdueIndividualDisbursedReceivableParentCode') ? overrides.chartOfAccountOverdueIndividualDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountOverdueNonDomiciledCompanyDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountOverdueNonDomiciledCompanyDisbursedReceivableParentCode') ? overrides.chartOfAccountOverdueNonDomiciledCompanyDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountOverduePrivateCompanyDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountOverduePrivateCompanyDisbursedReceivableParentCode') ? overrides.chartOfAccountOverduePrivateCompanyDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountPaymentHoldingParentCode: overrides && overrides.hasOwnProperty('chartOfAccountPaymentHoldingParentCode') ? overrides.chartOfAccountPaymentHoldingParentCode! : faker.lorem.word(),
        chartOfAccountShortTermBankDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermBankDisbursedReceivableParentCode') ? overrides.chartOfAccountShortTermBankDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermBankInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermBankInterestReceivableParentCode') ? overrides.chartOfAccountShortTermBankInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermFinancialInstitutionDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermFinancialInstitutionDisbursedReceivableParentCode') ? overrides.chartOfAccountShortTermFinancialInstitutionDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermFinancialInstitutionInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermFinancialInstitutionInterestReceivableParentCode') ? overrides.chartOfAccountShortTermFinancialInstitutionInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode') ? overrides.chartOfAccountShortTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermForeignAgencyOrSubsidiaryInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermForeignAgencyOrSubsidiaryInterestReceivableParentCode') ? overrides.chartOfAccountShortTermForeignAgencyOrSubsidiaryInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermGovernmentEntityDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermGovernmentEntityDisbursedReceivableParentCode') ? overrides.chartOfAccountShortTermGovernmentEntityDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermGovernmentEntityInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermGovernmentEntityInterestReceivableParentCode') ? overrides.chartOfAccountShortTermGovernmentEntityInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermIndividualDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermIndividualDisbursedReceivableParentCode') ? overrides.chartOfAccountShortTermIndividualDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermIndividualInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermIndividualInterestReceivableParentCode') ? overrides.chartOfAccountShortTermIndividualInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermNonDomiciledCompanyDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermNonDomiciledCompanyDisbursedReceivableParentCode') ? overrides.chartOfAccountShortTermNonDomiciledCompanyDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermNonDomiciledCompanyInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermNonDomiciledCompanyInterestReceivableParentCode') ? overrides.chartOfAccountShortTermNonDomiciledCompanyInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermPrivateCompanyDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermPrivateCompanyDisbursedReceivableParentCode') ? overrides.chartOfAccountShortTermPrivateCompanyDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermPrivateCompanyInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermPrivateCompanyInterestReceivableParentCode') ? overrides.chartOfAccountShortTermPrivateCompanyInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountsId: overrides && overrides.hasOwnProperty('chartOfAccountsId') ? overrides.chartOfAccountsId! : generateMockValue.uuid(),
    };
};

export const mockCreditModuleConfigureInput = (overrides?: Partial<CreditModuleConfigureInput>, _relationshipsToOmit: Set<string> = new Set()): CreditModuleConfigureInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditModuleConfigureInput');
    return {
        chartOfAccountCollateralOmnibusParentCode: overrides && overrides.hasOwnProperty('chartOfAccountCollateralOmnibusParentCode') ? overrides.chartOfAccountCollateralOmnibusParentCode! : faker.lorem.word(),
        chartOfAccountCollateralParentCode: overrides && overrides.hasOwnProperty('chartOfAccountCollateralParentCode') ? overrides.chartOfAccountCollateralParentCode! : faker.lorem.word(),
        chartOfAccountFacilityOmnibusParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFacilityOmnibusParentCode') ? overrides.chartOfAccountFacilityOmnibusParentCode! : faker.lorem.word(),
        chartOfAccountFacilityParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFacilityParentCode') ? overrides.chartOfAccountFacilityParentCode! : faker.lorem.word(),
        chartOfAccountFeeIncomeParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFeeIncomeParentCode') ? overrides.chartOfAccountFeeIncomeParentCode! : faker.lorem.word(),
        chartOfAccountInLiquidationOmnibusParentCode: overrides && overrides.hasOwnProperty('chartOfAccountInLiquidationOmnibusParentCode') ? overrides.chartOfAccountInLiquidationOmnibusParentCode! : faker.lorem.word(),
        chartOfAccountInLiquidationParentCode: overrides && overrides.hasOwnProperty('chartOfAccountInLiquidationParentCode') ? overrides.chartOfAccountInLiquidationParentCode! : faker.lorem.word(),
        chartOfAccountInterestIncomeParentCode: overrides && overrides.hasOwnProperty('chartOfAccountInterestIncomeParentCode') ? overrides.chartOfAccountInterestIncomeParentCode! : faker.lorem.word(),
        chartOfAccountLongTermBankDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermBankDisbursedReceivableParentCode') ? overrides.chartOfAccountLongTermBankDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermBankInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermBankInterestReceivableParentCode') ? overrides.chartOfAccountLongTermBankInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermFinancialInstitutionDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermFinancialInstitutionDisbursedReceivableParentCode') ? overrides.chartOfAccountLongTermFinancialInstitutionDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermFinancialInstitutionInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermFinancialInstitutionInterestReceivableParentCode') ? overrides.chartOfAccountLongTermFinancialInstitutionInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode') ? overrides.chartOfAccountLongTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermForeignAgencyOrSubsidiaryInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermForeignAgencyOrSubsidiaryInterestReceivableParentCode') ? overrides.chartOfAccountLongTermForeignAgencyOrSubsidiaryInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermGovernmentEntityDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermGovernmentEntityDisbursedReceivableParentCode') ? overrides.chartOfAccountLongTermGovernmentEntityDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermGovernmentEntityInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermGovernmentEntityInterestReceivableParentCode') ? overrides.chartOfAccountLongTermGovernmentEntityInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermIndividualDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermIndividualDisbursedReceivableParentCode') ? overrides.chartOfAccountLongTermIndividualDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermIndividualInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermIndividualInterestReceivableParentCode') ? overrides.chartOfAccountLongTermIndividualInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermNonDomiciledCompanyDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermNonDomiciledCompanyDisbursedReceivableParentCode') ? overrides.chartOfAccountLongTermNonDomiciledCompanyDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermNonDomiciledCompanyInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermNonDomiciledCompanyInterestReceivableParentCode') ? overrides.chartOfAccountLongTermNonDomiciledCompanyInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermPrivateCompanyDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermPrivateCompanyDisbursedReceivableParentCode') ? overrides.chartOfAccountLongTermPrivateCompanyDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountLongTermPrivateCompanyInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountLongTermPrivateCompanyInterestReceivableParentCode') ? overrides.chartOfAccountLongTermPrivateCompanyInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountOverdueBankDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountOverdueBankDisbursedReceivableParentCode') ? overrides.chartOfAccountOverdueBankDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountOverdueFinancialInstitutionDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountOverdueFinancialInstitutionDisbursedReceivableParentCode') ? overrides.chartOfAccountOverdueFinancialInstitutionDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountOverdueForeignAgencyOrSubsidiaryDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountOverdueForeignAgencyOrSubsidiaryDisbursedReceivableParentCode') ? overrides.chartOfAccountOverdueForeignAgencyOrSubsidiaryDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountOverdueGovernmentEntityDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountOverdueGovernmentEntityDisbursedReceivableParentCode') ? overrides.chartOfAccountOverdueGovernmentEntityDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountOverdueIndividualDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountOverdueIndividualDisbursedReceivableParentCode') ? overrides.chartOfAccountOverdueIndividualDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountOverdueNonDomiciledCompanyDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountOverdueNonDomiciledCompanyDisbursedReceivableParentCode') ? overrides.chartOfAccountOverdueNonDomiciledCompanyDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountOverduePrivateCompanyDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountOverduePrivateCompanyDisbursedReceivableParentCode') ? overrides.chartOfAccountOverduePrivateCompanyDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountPaymentHoldingParentCode: overrides && overrides.hasOwnProperty('chartOfAccountPaymentHoldingParentCode') ? overrides.chartOfAccountPaymentHoldingParentCode! : faker.lorem.word(),
        chartOfAccountShortTermBankDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermBankDisbursedReceivableParentCode') ? overrides.chartOfAccountShortTermBankDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermBankInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermBankInterestReceivableParentCode') ? overrides.chartOfAccountShortTermBankInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermFinancialInstitutionDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermFinancialInstitutionDisbursedReceivableParentCode') ? overrides.chartOfAccountShortTermFinancialInstitutionDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermFinancialInstitutionInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermFinancialInstitutionInterestReceivableParentCode') ? overrides.chartOfAccountShortTermFinancialInstitutionInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode') ? overrides.chartOfAccountShortTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermForeignAgencyOrSubsidiaryInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermForeignAgencyOrSubsidiaryInterestReceivableParentCode') ? overrides.chartOfAccountShortTermForeignAgencyOrSubsidiaryInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermGovernmentEntityDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermGovernmentEntityDisbursedReceivableParentCode') ? overrides.chartOfAccountShortTermGovernmentEntityDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermGovernmentEntityInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermGovernmentEntityInterestReceivableParentCode') ? overrides.chartOfAccountShortTermGovernmentEntityInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermIndividualDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermIndividualDisbursedReceivableParentCode') ? overrides.chartOfAccountShortTermIndividualDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermIndividualInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermIndividualInterestReceivableParentCode') ? overrides.chartOfAccountShortTermIndividualInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermNonDomiciledCompanyDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermNonDomiciledCompanyDisbursedReceivableParentCode') ? overrides.chartOfAccountShortTermNonDomiciledCompanyDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermNonDomiciledCompanyInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermNonDomiciledCompanyInterestReceivableParentCode') ? overrides.chartOfAccountShortTermNonDomiciledCompanyInterestReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermPrivateCompanyDisbursedReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermPrivateCompanyDisbursedReceivableParentCode') ? overrides.chartOfAccountShortTermPrivateCompanyDisbursedReceivableParentCode! : faker.lorem.word(),
        chartOfAccountShortTermPrivateCompanyInterestReceivableParentCode: overrides && overrides.hasOwnProperty('chartOfAccountShortTermPrivateCompanyInterestReceivableParentCode') ? overrides.chartOfAccountShortTermPrivateCompanyInterestReceivableParentCode! : faker.lorem.word(),
    };
};

export const mockCreditModuleConfigurePayload = (overrides?: Partial<CreditModuleConfigurePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditModuleConfigurePayload' } & CreditModuleConfigurePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditModuleConfigurePayload');
    return {
        __typename: 'CreditModuleConfigurePayload',
        creditConfig: overrides && overrides.hasOwnProperty('creditConfig') ? overrides.creditConfig! : relationshipsToOmit.has('CreditModuleConfig') ? {} as CreditModuleConfig : mockCreditModuleConfig({}, relationshipsToOmit),
    };
};

export const mockCustodian = (overrides?: Partial<Custodian>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Custodian' } & Custodian => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Custodian');
    return {
        __typename: 'Custodian',
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        custodianId: overrides && overrides.hasOwnProperty('custodianId') ? overrides.custodianId! : generateMockValue.uuid(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
    };
};

export const mockCustodianConfigInput = (override?: CustodianConfigInput, _relationshipsToOmit: Set<string> = new Set()): CustodianConfigInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustodianConfigInput');
    return {
        ...(override ? override : {bitgo : relationshipsToOmit.has('BitgoConfig') ? {} as BitgoConfig : mockBitgoConfig({}, relationshipsToOmit)}),
    };
};

export const mockCustodianConfigUpdateInput = (overrides?: Partial<CustodianConfigUpdateInput>, _relationshipsToOmit: Set<string> = new Set()): CustodianConfigUpdateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustodianConfigUpdateInput');
    return {
        config: overrides && overrides.hasOwnProperty('config') ? overrides.config! : mockCustodianConfigInput(),
        custodianId: overrides && overrides.hasOwnProperty('custodianId') ? overrides.custodianId! : generateMockValue.uuid(),
    };
};

export const mockCustodianConfigUpdatePayload = (overrides?: Partial<CustodianConfigUpdatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CustodianConfigUpdatePayload' } & CustodianConfigUpdatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustodianConfigUpdatePayload');
    return {
        __typename: 'CustodianConfigUpdatePayload',
        custodian: overrides && overrides.hasOwnProperty('custodian') ? overrides.custodian! : relationshipsToOmit.has('Custodian') ? {} as Custodian : mockCustodian({}, relationshipsToOmit),
    };
};

export const mockCustodianConnection = (overrides?: Partial<CustodianConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CustodianConnection' } & CustodianConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustodianConnection');
    return {
        __typename: 'CustodianConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('CustodianEdge') ? {} as CustodianEdge : mockCustodianEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('Custodian') ? {} as Custodian : mockCustodian({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockCustodianCreateInput = (override?: CustodianCreateInput, _relationshipsToOmit: Set<string> = new Set()): CustodianCreateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustodianCreateInput');
    return {
        ...(override ? override : {bitgo : relationshipsToOmit.has('BitgoConfig') ? {} as BitgoConfig : mockBitgoConfig({}, relationshipsToOmit)}),
    };
};

export const mockCustodianCreatePayload = (overrides?: Partial<CustodianCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CustodianCreatePayload' } & CustodianCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustodianCreatePayload');
    return {
        __typename: 'CustodianCreatePayload',
        custodian: overrides && overrides.hasOwnProperty('custodian') ? overrides.custodian! : relationshipsToOmit.has('Custodian') ? {} as Custodian : mockCustodian({}, relationshipsToOmit),
    };
};

export const mockCustodianEdge = (overrides?: Partial<CustodianEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CustodianEdge' } & CustodianEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustodianEdge');
    return {
        __typename: 'CustodianEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('Custodian') ? {} as Custodian : mockCustodian({}, relationshipsToOmit),
    };
};

export const mockCustomer = (overrides?: Partial<Customer>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Customer' } & Customer => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Customer');
    return {
        __typename: 'Customer',
        activity: overrides && overrides.hasOwnProperty('activity') ? overrides.activity! : Activity.Active,
        applicantId: overrides && overrides.hasOwnProperty('applicantId') ? overrides.applicantId! : generateMockValue.applicantId(),
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        creditFacilities: overrides && overrides.hasOwnProperty('creditFacilities') ? overrides.creditFacilities! : [relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit)],
        creditFacilityProposals: overrides && overrides.hasOwnProperty('creditFacilityProposals') ? overrides.creditFacilityProposals! : [relationshipsToOmit.has('CreditFacilityProposal') ? {} as CreditFacilityProposal : mockCreditFacilityProposal({}, relationshipsToOmit)],
        customerId: overrides && overrides.hasOwnProperty('customerId') ? overrides.customerId! : generateMockValue.uuid(),
        customerType: overrides && overrides.hasOwnProperty('customerType') ? overrides.customerType! : CustomerType.Bank,
        depositAccount: overrides && overrides.hasOwnProperty('depositAccount') ? overrides.depositAccount! : relationshipsToOmit.has('DepositAccount') ? {} as DepositAccount : mockDepositAccount({}, relationshipsToOmit),
        documents: overrides && overrides.hasOwnProperty('documents') ? overrides.documents! : [relationshipsToOmit.has('CustomerDocument') ? {} as CustomerDocument : mockCustomerDocument({}, relationshipsToOmit)],
        email: overrides && overrides.hasOwnProperty('email') ? overrides.email! : generateMockValue.email(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        kycVerification: overrides && overrides.hasOwnProperty('kycVerification') ? overrides.kycVerification! : KycVerification.PendingVerification,
        level: overrides && overrides.hasOwnProperty('level') ? overrides.level! : mockEnums.kycLevel(),
        pendingCreditFacilities: overrides && overrides.hasOwnProperty('pendingCreditFacilities') ? overrides.pendingCreditFacilities! : [relationshipsToOmit.has('PendingCreditFacility') ? {} as PendingCreditFacility : mockPendingCreditFacility({}, relationshipsToOmit)],
        publicId: overrides && overrides.hasOwnProperty('publicId') ? overrides.publicId! : faker.lorem.word(),
        telegramId: overrides && overrides.hasOwnProperty('telegramId') ? overrides.telegramId! : generateMockValue.telegramId(),
        transactions: overrides && overrides.hasOwnProperty('transactions') ? overrides.transactions! : [relationshipsToOmit.has('Deposit') ? {} as Deposit : mockDeposit({}, relationshipsToOmit)],
        userCanCreateCreditFacility: overrides && overrides.hasOwnProperty('userCanCreateCreditFacility') ? overrides.userCanCreateCreditFacility! : faker.datatype.boolean(),
    };
};

export const mockCustomerConnection = (overrides?: Partial<CustomerConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CustomerConnection' } & CustomerConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerConnection');
    return {
        __typename: 'CustomerConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('CustomerEdge') ? {} as CustomerEdge : mockCustomerEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockCustomerCreateInput = (overrides?: Partial<CustomerCreateInput>, _relationshipsToOmit: Set<string> = new Set()): CustomerCreateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerCreateInput');
    return {
        customerType: overrides && overrides.hasOwnProperty('customerType') ? overrides.customerType! : CustomerType.Bank,
        email: overrides && overrides.hasOwnProperty('email') ? overrides.email! : generateMockValue.email(),
        telegramId: overrides && overrides.hasOwnProperty('telegramId') ? overrides.telegramId! : generateMockValue.telegramId(),
    };
};

export const mockCustomerCreatePayload = (overrides?: Partial<CustomerCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CustomerCreatePayload' } & CustomerCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerCreatePayload');
    return {
        __typename: 'CustomerCreatePayload',
        customer: overrides && overrides.hasOwnProperty('customer') ? overrides.customer! : relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit),
    };
};

export const mockCustomerDocument = (overrides?: Partial<CustomerDocument>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CustomerDocument' } & CustomerDocument => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerDocument');
    return {
        __typename: 'CustomerDocument',
        customerId: overrides && overrides.hasOwnProperty('customerId') ? overrides.customerId! : generateMockValue.uuid(),
        documentId: overrides && overrides.hasOwnProperty('documentId') ? overrides.documentId! : generateMockValue.uuid(),
        filename: overrides && overrides.hasOwnProperty('filename') ? overrides.filename! : generateMockValue.filename(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : mockEnums.documentStatus(),
    };
};

export const mockCustomerDocumentArchiveInput = (overrides?: Partial<CustomerDocumentArchiveInput>, _relationshipsToOmit: Set<string> = new Set()): CustomerDocumentArchiveInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerDocumentArchiveInput');
    return {
        documentId: overrides && overrides.hasOwnProperty('documentId') ? overrides.documentId! : generateMockValue.uuid(),
    };
};

export const mockCustomerDocumentArchivePayload = (overrides?: Partial<CustomerDocumentArchivePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CustomerDocumentArchivePayload' } & CustomerDocumentArchivePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerDocumentArchivePayload');
    return {
        __typename: 'CustomerDocumentArchivePayload',
        document: overrides && overrides.hasOwnProperty('document') ? overrides.document! : relationshipsToOmit.has('CustomerDocument') ? {} as CustomerDocument : mockCustomerDocument({}, relationshipsToOmit),
    };
};

export const mockCustomerDocumentCreateInput = (overrides?: Partial<CustomerDocumentCreateInput>, _relationshipsToOmit: Set<string> = new Set()): CustomerDocumentCreateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerDocumentCreateInput');
    return {
        customerId: overrides && overrides.hasOwnProperty('customerId') ? overrides.customerId! : generateMockValue.uuid(),
        file: overrides && overrides.hasOwnProperty('file') ? overrides.file! : faker.lorem.word(),
    };
};

export const mockCustomerDocumentCreatePayload = (overrides?: Partial<CustomerDocumentCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CustomerDocumentCreatePayload' } & CustomerDocumentCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerDocumentCreatePayload');
    return {
        __typename: 'CustomerDocumentCreatePayload',
        document: overrides && overrides.hasOwnProperty('document') ? overrides.document! : relationshipsToOmit.has('CustomerDocument') ? {} as CustomerDocument : mockCustomerDocument({}, relationshipsToOmit),
    };
};

export const mockCustomerDocumentDeleteInput = (overrides?: Partial<CustomerDocumentDeleteInput>, _relationshipsToOmit: Set<string> = new Set()): CustomerDocumentDeleteInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerDocumentDeleteInput');
    return {
        documentId: overrides && overrides.hasOwnProperty('documentId') ? overrides.documentId! : generateMockValue.uuid(),
    };
};

export const mockCustomerDocumentDeletePayload = (overrides?: Partial<CustomerDocumentDeletePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CustomerDocumentDeletePayload' } & CustomerDocumentDeletePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerDocumentDeletePayload');
    return {
        __typename: 'CustomerDocumentDeletePayload',
        deletedDocumentId: overrides && overrides.hasOwnProperty('deletedDocumentId') ? overrides.deletedDocumentId! : generateMockValue.uuid(),
    };
};

export const mockCustomerDocumentDownloadLinksGenerateInput = (overrides?: Partial<CustomerDocumentDownloadLinksGenerateInput>, _relationshipsToOmit: Set<string> = new Set()): CustomerDocumentDownloadLinksGenerateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerDocumentDownloadLinksGenerateInput');
    return {
        documentId: overrides && overrides.hasOwnProperty('documentId') ? overrides.documentId! : generateMockValue.uuid(),
    };
};

export const mockCustomerDocumentDownloadLinksGeneratePayload = (overrides?: Partial<CustomerDocumentDownloadLinksGeneratePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CustomerDocumentDownloadLinksGeneratePayload' } & CustomerDocumentDownloadLinksGeneratePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerDocumentDownloadLinksGeneratePayload');
    return {
        __typename: 'CustomerDocumentDownloadLinksGeneratePayload',
        documentId: overrides && overrides.hasOwnProperty('documentId') ? overrides.documentId! : generateMockValue.uuid(),
        link: overrides && overrides.hasOwnProperty('link') ? overrides.link! : faker.lorem.word(),
    };
};

export const mockCustomerEdge = (overrides?: Partial<CustomerEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CustomerEdge' } & CustomerEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerEdge');
    return {
        __typename: 'CustomerEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit),
    };
};

export const mockCustomerEmailUpdateInput = (overrides?: Partial<CustomerEmailUpdateInput>, _relationshipsToOmit: Set<string> = new Set()): CustomerEmailUpdateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerEmailUpdateInput');
    return {
        customerId: overrides && overrides.hasOwnProperty('customerId') ? overrides.customerId! : generateMockValue.uuid(),
        email: overrides && overrides.hasOwnProperty('email') ? overrides.email! : generateMockValue.email(),
    };
};

export const mockCustomerEmailUpdatePayload = (overrides?: Partial<CustomerEmailUpdatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CustomerEmailUpdatePayload' } & CustomerEmailUpdatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerEmailUpdatePayload');
    return {
        __typename: 'CustomerEmailUpdatePayload',
        customer: overrides && overrides.hasOwnProperty('customer') ? overrides.customer! : relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit),
    };
};

export const mockCustomerTelegramIdUpdateInput = (overrides?: Partial<CustomerTelegramIdUpdateInput>, _relationshipsToOmit: Set<string> = new Set()): CustomerTelegramIdUpdateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerTelegramIdUpdateInput');
    return {
        customerId: overrides && overrides.hasOwnProperty('customerId') ? overrides.customerId! : generateMockValue.uuid(),
        telegramId: overrides && overrides.hasOwnProperty('telegramId') ? overrides.telegramId! : generateMockValue.telegramId(),
    };
};

export const mockCustomerTelegramIdUpdatePayload = (overrides?: Partial<CustomerTelegramIdUpdatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CustomerTelegramIdUpdatePayload' } & CustomerTelegramIdUpdatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomerTelegramIdUpdatePayload');
    return {
        __typename: 'CustomerTelegramIdUpdatePayload',
        customer: overrides && overrides.hasOwnProperty('customer') ? overrides.customer! : relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit),
    };
};

export const mockCustomersFilter = (overrides?: Partial<CustomersFilter>, _relationshipsToOmit: Set<string> = new Set()): CustomersFilter => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomersFilter');
    return {
        field: overrides && overrides.hasOwnProperty('field') ? overrides.field! : CustomersFilterBy.AccountKycVerification,
        kycVerification: overrides && overrides.hasOwnProperty('kycVerification') ? overrides.kycVerification! : KycVerification.PendingVerification,
    };
};

export const mockCustomersSort = (overrides?: Partial<CustomersSort>, _relationshipsToOmit: Set<string> = new Set()): CustomersSort => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CustomersSort');
    return {
        by: overrides && overrides.hasOwnProperty('by') ? overrides.by! : CustomersSortBy.CreatedAt,
        direction: overrides && overrides.hasOwnProperty('direction') ? overrides.direction! : SortDirection.Asc,
    };
};

export const mockDashboard = (overrides?: Partial<Dashboard>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Dashboard' } & Dashboard => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Dashboard');
    return {
        __typename: 'Dashboard',
        activeFacilities: overrides && overrides.hasOwnProperty('activeFacilities') ? overrides.activeFacilities! : generateMockValue.int(),
        pendingFacilities: overrides && overrides.hasOwnProperty('pendingFacilities') ? overrides.pendingFacilities! : generateMockValue.int(),
        totalCollateral: overrides && overrides.hasOwnProperty('totalCollateral') ? overrides.totalCollateral! : generateMockValue.satoshis(),
        totalDisbursed: overrides && overrides.hasOwnProperty('totalDisbursed') ? overrides.totalDisbursed! : generateMockValue.usdCents(),
    };
};

export const mockDeposit = (overrides?: Partial<Deposit>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Deposit' } & Deposit => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Deposit');
    return {
        __typename: 'Deposit',
        account: overrides && overrides.hasOwnProperty('account') ? overrides.account! : relationshipsToOmit.has('DepositAccount') ? {} as DepositAccount : mockDepositAccount({}, relationshipsToOmit),
        accountId: overrides && overrides.hasOwnProperty('accountId') ? overrides.accountId! : generateMockValue.uuid(),
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.usdCents(),
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        depositId: overrides && overrides.hasOwnProperty('depositId') ? overrides.depositId! : generateMockValue.uuid(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        ledgerTransactions: overrides && overrides.hasOwnProperty('ledgerTransactions') ? overrides.ledgerTransactions! : [relationshipsToOmit.has('LedgerTransaction') ? {} as LedgerTransaction : mockLedgerTransaction({}, relationshipsToOmit)],
        publicId: overrides && overrides.hasOwnProperty('publicId') ? overrides.publicId! : faker.lorem.word(),
        reference: overrides && overrides.hasOwnProperty('reference') ? overrides.reference! : generateMockValue.reference(),
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : DepositStatus.Confirmed,
    };
};

export const mockDepositAccount = (overrides?: Partial<DepositAccount>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositAccount' } & DepositAccount => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccount');
    return {
        __typename: 'DepositAccount',
        balance: overrides && overrides.hasOwnProperty('balance') ? overrides.balance! : relationshipsToOmit.has('DepositAccountBalance') ? {} as DepositAccountBalance : mockDepositAccountBalance({}, relationshipsToOmit),
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        customer: overrides && overrides.hasOwnProperty('customer') ? overrides.customer! : relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit),
        customerId: overrides && overrides.hasOwnProperty('customerId') ? overrides.customerId! : generateMockValue.uuid(),
        depositAccountId: overrides && overrides.hasOwnProperty('depositAccountId') ? overrides.depositAccountId! : generateMockValue.uuid(),
        deposits: overrides && overrides.hasOwnProperty('deposits') ? overrides.deposits! : [relationshipsToOmit.has('Deposit') ? {} as Deposit : mockDeposit({}, relationshipsToOmit)],
        history: overrides && overrides.hasOwnProperty('history') ? overrides.history! : relationshipsToOmit.has('DepositAccountHistoryEntryConnection') ? {} as DepositAccountHistoryEntryConnection : mockDepositAccountHistoryEntryConnection({}, relationshipsToOmit),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        ledgerAccounts: overrides && overrides.hasOwnProperty('ledgerAccounts') ? overrides.ledgerAccounts! : relationshipsToOmit.has('DepositAccountLedgerAccounts') ? {} as DepositAccountLedgerAccounts : mockDepositAccountLedgerAccounts({}, relationshipsToOmit),
        publicId: overrides && overrides.hasOwnProperty('publicId') ? overrides.publicId! : faker.lorem.word(),
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : DepositAccountStatus.Active,
        withdrawals: overrides && overrides.hasOwnProperty('withdrawals') ? overrides.withdrawals! : [relationshipsToOmit.has('Withdrawal') ? {} as Withdrawal : mockWithdrawal({}, relationshipsToOmit)],
    };
};

export const mockDepositAccountBalance = (overrides?: Partial<DepositAccountBalance>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositAccountBalance' } & DepositAccountBalance => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccountBalance');
    return {
        __typename: 'DepositAccountBalance',
        pending: overrides && overrides.hasOwnProperty('pending') ? overrides.pending! : generateMockValue.usdCents(),
        settled: overrides && overrides.hasOwnProperty('settled') ? overrides.settled! : generateMockValue.usdCents(),
    };
};

export const mockDepositAccountCloseInput = (overrides?: Partial<DepositAccountCloseInput>, _relationshipsToOmit: Set<string> = new Set()): DepositAccountCloseInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccountCloseInput');
    return {
        depositAccountId: overrides && overrides.hasOwnProperty('depositAccountId') ? overrides.depositAccountId! : generateMockValue.uuid(),
    };
};

export const mockDepositAccountClosePayload = (overrides?: Partial<DepositAccountClosePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositAccountClosePayload' } & DepositAccountClosePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccountClosePayload');
    return {
        __typename: 'DepositAccountClosePayload',
        account: overrides && overrides.hasOwnProperty('account') ? overrides.account! : relationshipsToOmit.has('DepositAccount') ? {} as DepositAccount : mockDepositAccount({}, relationshipsToOmit),
    };
};

export const mockDepositAccountConnection = (overrides?: Partial<DepositAccountConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositAccountConnection' } & DepositAccountConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccountConnection');
    return {
        __typename: 'DepositAccountConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('DepositAccountEdge') ? {} as DepositAccountEdge : mockDepositAccountEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('DepositAccount') ? {} as DepositAccount : mockDepositAccount({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockDepositAccountCreateInput = (overrides?: Partial<DepositAccountCreateInput>, _relationshipsToOmit: Set<string> = new Set()): DepositAccountCreateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccountCreateInput');
    return {
        customerId: overrides && overrides.hasOwnProperty('customerId') ? overrides.customerId! : generateMockValue.uuid(),
    };
};

export const mockDepositAccountCreatePayload = (overrides?: Partial<DepositAccountCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositAccountCreatePayload' } & DepositAccountCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccountCreatePayload');
    return {
        __typename: 'DepositAccountCreatePayload',
        account: overrides && overrides.hasOwnProperty('account') ? overrides.account! : relationshipsToOmit.has('DepositAccount') ? {} as DepositAccount : mockDepositAccount({}, relationshipsToOmit),
    };
};

export const mockDepositAccountEdge = (overrides?: Partial<DepositAccountEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositAccountEdge' } & DepositAccountEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccountEdge');
    return {
        __typename: 'DepositAccountEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('DepositAccount') ? {} as DepositAccount : mockDepositAccount({}, relationshipsToOmit),
    };
};

export const mockDepositAccountFreezeInput = (overrides?: Partial<DepositAccountFreezeInput>, _relationshipsToOmit: Set<string> = new Set()): DepositAccountFreezeInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccountFreezeInput');
    return {
        depositAccountId: overrides && overrides.hasOwnProperty('depositAccountId') ? overrides.depositAccountId! : generateMockValue.uuid(),
    };
};

export const mockDepositAccountFreezePayload = (overrides?: Partial<DepositAccountFreezePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositAccountFreezePayload' } & DepositAccountFreezePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccountFreezePayload');
    return {
        __typename: 'DepositAccountFreezePayload',
        account: overrides && overrides.hasOwnProperty('account') ? overrides.account! : relationshipsToOmit.has('DepositAccount') ? {} as DepositAccount : mockDepositAccount({}, relationshipsToOmit),
    };
};

export const mockDepositAccountHistoryEntryConnection = (overrides?: Partial<DepositAccountHistoryEntryConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositAccountHistoryEntryConnection' } & DepositAccountHistoryEntryConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccountHistoryEntryConnection');
    return {
        __typename: 'DepositAccountHistoryEntryConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('DepositAccountHistoryEntryEdge') ? {} as DepositAccountHistoryEntryEdge : mockDepositAccountHistoryEntryEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('CancelledWithdrawalEntry') ? {} as CancelledWithdrawalEntry : mockCancelledWithdrawalEntry({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockDepositAccountHistoryEntryEdge = (overrides?: Partial<DepositAccountHistoryEntryEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositAccountHistoryEntryEdge' } & DepositAccountHistoryEntryEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccountHistoryEntryEdge');
    return {
        __typename: 'DepositAccountHistoryEntryEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('CancelledWithdrawalEntry') ? {} as CancelledWithdrawalEntry : mockCancelledWithdrawalEntry({}, relationshipsToOmit),
    };
};

export const mockDepositAccountLedgerAccounts = (overrides?: Partial<DepositAccountLedgerAccounts>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositAccountLedgerAccounts' } & DepositAccountLedgerAccounts => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccountLedgerAccounts');
    return {
        __typename: 'DepositAccountLedgerAccounts',
        depositAccount: overrides && overrides.hasOwnProperty('depositAccount') ? overrides.depositAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        depositAccountId: overrides && overrides.hasOwnProperty('depositAccountId') ? overrides.depositAccountId! : generateMockValue.uuid(),
        frozenDepositAccount: overrides && overrides.hasOwnProperty('frozenDepositAccount') ? overrides.frozenDepositAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        frozenDepositAccountId: overrides && overrides.hasOwnProperty('frozenDepositAccountId') ? overrides.frozenDepositAccountId! : generateMockValue.uuid(),
    };
};

export const mockDepositAccountUnfreezeInput = (overrides?: Partial<DepositAccountUnfreezeInput>, _relationshipsToOmit: Set<string> = new Set()): DepositAccountUnfreezeInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccountUnfreezeInput');
    return {
        depositAccountId: overrides && overrides.hasOwnProperty('depositAccountId') ? overrides.depositAccountId! : generateMockValue.uuid(),
    };
};

export const mockDepositAccountUnfreezePayload = (overrides?: Partial<DepositAccountUnfreezePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositAccountUnfreezePayload' } & DepositAccountUnfreezePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositAccountUnfreezePayload');
    return {
        __typename: 'DepositAccountUnfreezePayload',
        account: overrides && overrides.hasOwnProperty('account') ? overrides.account! : relationshipsToOmit.has('DepositAccount') ? {} as DepositAccount : mockDepositAccount({}, relationshipsToOmit),
    };
};

export const mockDepositConnection = (overrides?: Partial<DepositConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositConnection' } & DepositConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositConnection');
    return {
        __typename: 'DepositConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('DepositEdge') ? {} as DepositEdge : mockDepositEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('Deposit') ? {} as Deposit : mockDeposit({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockDepositEdge = (overrides?: Partial<DepositEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositEdge' } & DepositEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositEdge');
    return {
        __typename: 'DepositEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('Deposit') ? {} as Deposit : mockDeposit({}, relationshipsToOmit),
    };
};

export const mockDepositEntry = (overrides?: Partial<DepositEntry>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositEntry' } & DepositEntry => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositEntry');
    return {
        __typename: 'DepositEntry',
        deposit: overrides && overrides.hasOwnProperty('deposit') ? overrides.deposit! : relationshipsToOmit.has('Deposit') ? {} as Deposit : mockDeposit({}, relationshipsToOmit),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
    };
};

export const mockDepositModuleConfig = (overrides?: Partial<DepositModuleConfig>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositModuleConfig' } & DepositModuleConfig => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositModuleConfig');
    return {
        __typename: 'DepositModuleConfig',
        chartOfAccountBankDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountBankDepositAccountsParentCode') ? overrides.chartOfAccountBankDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountFinancialInstitutionDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFinancialInstitutionDepositAccountsParentCode') ? overrides.chartOfAccountFinancialInstitutionDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountFrozenBankDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFrozenBankDepositAccountsParentCode') ? overrides.chartOfAccountFrozenBankDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode') ? overrides.chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode') ? overrides.chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode') ? overrides.chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountNonDomiciledIndividualDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountNonDomiciledIndividualDepositAccountsParentCode') ? overrides.chartOfAccountNonDomiciledIndividualDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountPrivateCompanyDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountPrivateCompanyDepositAccountsParentCode') ? overrides.chartOfAccountPrivateCompanyDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode') ? overrides.chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountsFrozenIndividualDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountsFrozenIndividualDepositAccountsParentCode') ? overrides.chartOfAccountsFrozenIndividualDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountsGovernmentEntityDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountsGovernmentEntityDepositAccountsParentCode') ? overrides.chartOfAccountsGovernmentEntityDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountsId: overrides && overrides.hasOwnProperty('chartOfAccountsId') ? overrides.chartOfAccountsId! : generateMockValue.uuid(),
        chartOfAccountsIndividualDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountsIndividualDepositAccountsParentCode') ? overrides.chartOfAccountsIndividualDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountsOmnibusParentCode: overrides && overrides.hasOwnProperty('chartOfAccountsOmnibusParentCode') ? overrides.chartOfAccountsOmnibusParentCode! : faker.lorem.word(),
    };
};

export const mockDepositModuleConfigureInput = (overrides?: Partial<DepositModuleConfigureInput>, _relationshipsToOmit: Set<string> = new Set()): DepositModuleConfigureInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositModuleConfigureInput');
    return {
        chartOfAccountBankDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountBankDepositAccountsParentCode') ? overrides.chartOfAccountBankDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountFinancialInstitutionDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFinancialInstitutionDepositAccountsParentCode') ? overrides.chartOfAccountFinancialInstitutionDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountFrozenBankDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFrozenBankDepositAccountsParentCode') ? overrides.chartOfAccountFrozenBankDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode') ? overrides.chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode') ? overrides.chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode') ? overrides.chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountNonDomiciledIndividualDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountNonDomiciledIndividualDepositAccountsParentCode') ? overrides.chartOfAccountNonDomiciledIndividualDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountPrivateCompanyDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountPrivateCompanyDepositAccountsParentCode') ? overrides.chartOfAccountPrivateCompanyDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode') ? overrides.chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountsFrozenIndividualDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountsFrozenIndividualDepositAccountsParentCode') ? overrides.chartOfAccountsFrozenIndividualDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountsGovernmentEntityDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountsGovernmentEntityDepositAccountsParentCode') ? overrides.chartOfAccountsGovernmentEntityDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountsIndividualDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountsIndividualDepositAccountsParentCode') ? overrides.chartOfAccountsIndividualDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountsOmnibusParentCode: overrides && overrides.hasOwnProperty('chartOfAccountsOmnibusParentCode') ? overrides.chartOfAccountsOmnibusParentCode! : faker.lorem.word(),
    };
};

export const mockDepositModuleConfigurePayload = (overrides?: Partial<DepositModuleConfigurePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositModuleConfigurePayload' } & DepositModuleConfigurePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositModuleConfigurePayload');
    return {
        __typename: 'DepositModuleConfigurePayload',
        depositConfig: overrides && overrides.hasOwnProperty('depositConfig') ? overrides.depositConfig! : relationshipsToOmit.has('DepositModuleConfig') ? {} as DepositModuleConfig : mockDepositModuleConfig({}, relationshipsToOmit),
    };
};

export const mockDepositRecordInput = (overrides?: Partial<DepositRecordInput>, _relationshipsToOmit: Set<string> = new Set()): DepositRecordInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositRecordInput');
    return {
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.usdCents(),
        depositAccountId: overrides && overrides.hasOwnProperty('depositAccountId') ? overrides.depositAccountId! : generateMockValue.uuid(),
        reference: overrides && overrides.hasOwnProperty('reference') ? overrides.reference! : generateMockValue.reference(),
    };
};

export const mockDepositRecordPayload = (overrides?: Partial<DepositRecordPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositRecordPayload' } & DepositRecordPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositRecordPayload');
    return {
        __typename: 'DepositRecordPayload',
        deposit: overrides && overrides.hasOwnProperty('deposit') ? overrides.deposit! : relationshipsToOmit.has('Deposit') ? {} as Deposit : mockDeposit({}, relationshipsToOmit),
    };
};

export const mockDepositRevertInput = (overrides?: Partial<DepositRevertInput>, _relationshipsToOmit: Set<string> = new Set()): DepositRevertInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositRevertInput');
    return {
        depositId: overrides && overrides.hasOwnProperty('depositId') ? overrides.depositId! : generateMockValue.uuid(),
    };
};

export const mockDepositRevertPayload = (overrides?: Partial<DepositRevertPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DepositRevertPayload' } & DepositRevertPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DepositRevertPayload');
    return {
        __typename: 'DepositRevertPayload',
        deposit: overrides && overrides.hasOwnProperty('deposit') ? overrides.deposit! : relationshipsToOmit.has('Deposit') ? {} as Deposit : mockDeposit({}, relationshipsToOmit),
    };
};

export const mockDisbursalEntry = (overrides?: Partial<DisbursalEntry>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DisbursalEntry' } & DisbursalEntry => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DisbursalEntry');
    return {
        __typename: 'DisbursalEntry',
        disbursal: overrides && overrides.hasOwnProperty('disbursal') ? overrides.disbursal! : relationshipsToOmit.has('CreditFacilityDisbursal') ? {} as CreditFacilityDisbursal : mockCreditFacilityDisbursal({}, relationshipsToOmit),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
    };
};

export const mockDisbursed = (overrides?: Partial<Disbursed>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Disbursed' } & Disbursed => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Disbursed');
    return {
        __typename: 'Disbursed',
        outstanding: overrides && overrides.hasOwnProperty('outstanding') ? overrides.outstanding! : relationshipsToOmit.has('Outstanding') ? {} as Outstanding : mockOutstanding({}, relationshipsToOmit),
        outstandingPayable: overrides && overrides.hasOwnProperty('outstandingPayable') ? overrides.outstandingPayable! : relationshipsToOmit.has('Outstanding') ? {} as Outstanding : mockOutstanding({}, relationshipsToOmit),
        total: overrides && overrides.hasOwnProperty('total') ? overrides.total! : relationshipsToOmit.has('Total') ? {} as Total : mockTotal({}, relationshipsToOmit),
    };
};

export const mockDuration = (overrides?: Partial<Duration>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Duration' } & Duration => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Duration');
    return {
        __typename: 'Duration',
        period: overrides && overrides.hasOwnProperty('period') ? overrides.period! : mockEnums.period(),
        units: overrides && overrides.hasOwnProperty('units') ? overrides.units! : faker.helpers.arrayElement([6, 12, 24]),
    };
};

export const mockDurationInput = (overrides?: Partial<DurationInput>, _relationshipsToOmit: Set<string> = new Set()): DurationInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DurationInput');
    return {
        period: overrides && overrides.hasOwnProperty('period') ? overrides.period! : Period.Days,
        units: overrides && overrides.hasOwnProperty('units') ? overrides.units! : faker.number.int({ min: 0, max: 9999 }),
    };
};

export const mockExposedConfigItem = (overrides?: Partial<ExposedConfigItem>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ExposedConfigItem' } & ExposedConfigItem => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ExposedConfigItem');
    return {
        __typename: 'ExposedConfigItem',
        configType: overrides && overrides.hasOwnProperty('configType') ? overrides.configType! : ConfigType.Bool,
        isSet: overrides && overrides.hasOwnProperty('isSet') ? overrides.isSet! : faker.datatype.boolean(),
        key: overrides && overrides.hasOwnProperty('key') ? overrides.key! : faker.lorem.word(),
        value: overrides && overrides.hasOwnProperty('value') ? overrides.value! : faker.lorem.word(),
    };
};

export const mockExposedConfigUpdateInput = (overrides?: Partial<ExposedConfigUpdateInput>, _relationshipsToOmit: Set<string> = new Set()): ExposedConfigUpdateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ExposedConfigUpdateInput');
    return {
        key: overrides && overrides.hasOwnProperty('key') ? overrides.key! : faker.lorem.word(),
        value: overrides && overrides.hasOwnProperty('value') ? overrides.value! : faker.lorem.word(),
    };
};

export const mockExposedConfigUpdatePayload = (overrides?: Partial<ExposedConfigUpdatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ExposedConfigUpdatePayload' } & ExposedConfigUpdatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ExposedConfigUpdatePayload');
    return {
        __typename: 'ExposedConfigUpdatePayload',
        exposedConfig: overrides && overrides.hasOwnProperty('exposedConfig') ? overrides.exposedConfig! : relationshipsToOmit.has('ExposedConfigItem') ? {} as ExposedConfigItem : mockExposedConfigItem({}, relationshipsToOmit),
    };
};

export const mockFacilityRemaining = (overrides?: Partial<FacilityRemaining>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'FacilityRemaining' } & FacilityRemaining => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FacilityRemaining');
    return {
        __typename: 'FacilityRemaining',
        usdBalance: overrides && overrides.hasOwnProperty('usdBalance') ? overrides.usdBalance! : generateMockValue.usdCents(),
    };
};

export const mockFiniteCvlPct = (overrides?: Partial<FiniteCvlPct>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'FiniteCVLPct' } & FiniteCvlPct => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FiniteCvlPct');
    return {
        __typename: 'FiniteCVLPct',
        value: overrides && overrides.hasOwnProperty('value') ? overrides.value! : faker.lorem.word(),
    };
};

export const mockFiscalMonthClosure = (overrides?: Partial<FiscalMonthClosure>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'FiscalMonthClosure' } & FiscalMonthClosure => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FiscalMonthClosure');
    return {
        __typename: 'FiscalMonthClosure',
        closedAsOf: overrides && overrides.hasOwnProperty('closedAsOf') ? overrides.closedAsOf! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        closedAt: overrides && overrides.hasOwnProperty('closedAt') ? overrides.closedAt! : generateMockValue.timestamp(),
    };
};

export const mockFiscalYear = (overrides?: Partial<FiscalYear>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'FiscalYear' } & FiscalYear => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FiscalYear');
    return {
        __typename: 'FiscalYear',
        chartId: overrides && overrides.hasOwnProperty('chartId') ? overrides.chartId! : generateMockValue.uuid(),
        closedAsOf: overrides && overrides.hasOwnProperty('closedAsOf') ? overrides.closedAsOf! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        fiscalYearId: overrides && overrides.hasOwnProperty('fiscalYearId') ? overrides.fiscalYearId! : generateMockValue.uuid(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        isLastMonthOfYearClosed: overrides && overrides.hasOwnProperty('isLastMonthOfYearClosed') ? overrides.isLastMonthOfYearClosed! : faker.datatype.boolean(),
        isOpen: overrides && overrides.hasOwnProperty('isOpen') ? overrides.isOpen! : faker.datatype.boolean(),
        monthClosures: overrides && overrides.hasOwnProperty('monthClosures') ? overrides.monthClosures! : [relationshipsToOmit.has('FiscalMonthClosure') ? {} as FiscalMonthClosure : mockFiscalMonthClosure({}, relationshipsToOmit)],
        nextMonthToClose: overrides && overrides.hasOwnProperty('nextMonthToClose') ? overrides.nextMonthToClose! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        openedAsOf: overrides && overrides.hasOwnProperty('openedAsOf') ? overrides.openedAsOf! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        reference: overrides && overrides.hasOwnProperty('reference') ? overrides.reference! : generateMockValue.reference(),
        year: overrides && overrides.hasOwnProperty('year') ? overrides.year! : faker.lorem.word(),
    };
};

export const mockFiscalYearCloseInput = (overrides?: Partial<FiscalYearCloseInput>, _relationshipsToOmit: Set<string> = new Set()): FiscalYearCloseInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FiscalYearCloseInput');
    return {
        fiscalYearId: overrides && overrides.hasOwnProperty('fiscalYearId') ? overrides.fiscalYearId! : generateMockValue.uuid(),
    };
};

export const mockFiscalYearCloseMonthInput = (overrides?: Partial<FiscalYearCloseMonthInput>, _relationshipsToOmit: Set<string> = new Set()): FiscalYearCloseMonthInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FiscalYearCloseMonthInput');
    return {
        fiscalYearId: overrides && overrides.hasOwnProperty('fiscalYearId') ? overrides.fiscalYearId! : generateMockValue.uuid(),
    };
};

export const mockFiscalYearCloseMonthPayload = (overrides?: Partial<FiscalYearCloseMonthPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'FiscalYearCloseMonthPayload' } & FiscalYearCloseMonthPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FiscalYearCloseMonthPayload');
    return {
        __typename: 'FiscalYearCloseMonthPayload',
        fiscalYear: overrides && overrides.hasOwnProperty('fiscalYear') ? overrides.fiscalYear! : relationshipsToOmit.has('FiscalYear') ? {} as FiscalYear : mockFiscalYear({}, relationshipsToOmit),
    };
};

export const mockFiscalYearClosePayload = (overrides?: Partial<FiscalYearClosePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'FiscalYearClosePayload' } & FiscalYearClosePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FiscalYearClosePayload');
    return {
        __typename: 'FiscalYearClosePayload',
        fiscalYear: overrides && overrides.hasOwnProperty('fiscalYear') ? overrides.fiscalYear! : relationshipsToOmit.has('FiscalYear') ? {} as FiscalYear : mockFiscalYear({}, relationshipsToOmit),
    };
};

export const mockFiscalYearConnection = (overrides?: Partial<FiscalYearConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'FiscalYearConnection' } & FiscalYearConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FiscalYearConnection');
    return {
        __typename: 'FiscalYearConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('FiscalYearEdge') ? {} as FiscalYearEdge : mockFiscalYearEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('FiscalYear') ? {} as FiscalYear : mockFiscalYear({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockFiscalYearEdge = (overrides?: Partial<FiscalYearEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'FiscalYearEdge' } & FiscalYearEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FiscalYearEdge');
    return {
        __typename: 'FiscalYearEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('FiscalYear') ? {} as FiscalYear : mockFiscalYear({}, relationshipsToOmit),
    };
};

export const mockFiscalYearInitInput = (overrides?: Partial<FiscalYearInitInput>, _relationshipsToOmit: Set<string> = new Set()): FiscalYearInitInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FiscalYearInitInput');
    return {
        openedAsOf: overrides && overrides.hasOwnProperty('openedAsOf') ? overrides.openedAsOf! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
    };
};

export const mockFiscalYearInitPayload = (overrides?: Partial<FiscalYearInitPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'FiscalYearInitPayload' } & FiscalYearInitPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FiscalYearInitPayload');
    return {
        __typename: 'FiscalYearInitPayload',
        fiscalYear: overrides && overrides.hasOwnProperty('fiscalYear') ? overrides.fiscalYear! : relationshipsToOmit.has('FiscalYear') ? {} as FiscalYear : mockFiscalYear({}, relationshipsToOmit),
    };
};

export const mockFiscalYearOpenNextInput = (overrides?: Partial<FiscalYearOpenNextInput>, _relationshipsToOmit: Set<string> = new Set()): FiscalYearOpenNextInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FiscalYearOpenNextInput');
    return {
        fiscalYearId: overrides && overrides.hasOwnProperty('fiscalYearId') ? overrides.fiscalYearId! : generateMockValue.uuid(),
    };
};

export const mockFiscalYearOpenNextPayload = (overrides?: Partial<FiscalYearOpenNextPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'FiscalYearOpenNextPayload' } & FiscalYearOpenNextPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FiscalYearOpenNextPayload');
    return {
        __typename: 'FiscalYearOpenNextPayload',
        fiscalYear: overrides && overrides.hasOwnProperty('fiscalYear') ? overrides.fiscalYear! : relationshipsToOmit.has('FiscalYear') ? {} as FiscalYear : mockFiscalYear({}, relationshipsToOmit),
    };
};

export const mockFreezeEntry = (overrides?: Partial<FreezeEntry>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'FreezeEntry' } & FreezeEntry => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FreezeEntry');
    return {
        __typename: 'FreezeEntry',
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.usdCents(),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        txId: overrides && overrides.hasOwnProperty('txId') ? overrides.txId! : generateMockValue.uuid(),
    };
};

export const mockGovernanceNavigationItems = (overrides?: Partial<GovernanceNavigationItems>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'GovernanceNavigationItems' } & GovernanceNavigationItems => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('GovernanceNavigationItems');
    return {
        __typename: 'GovernanceNavigationItems',
        approvalProcess: overrides && overrides.hasOwnProperty('approvalProcess') ? overrides.approvalProcess! : faker.datatype.boolean(),
        committee: overrides && overrides.hasOwnProperty('committee') ? overrides.committee! : faker.datatype.boolean(),
        policy: overrides && overrides.hasOwnProperty('policy') ? overrides.policy! : faker.datatype.boolean(),
    };
};

export const mockInfiniteCvlPct = (overrides?: Partial<InfiniteCvlPct>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'InfiniteCVLPct' } & InfiniteCvlPct => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('InfiniteCvlPct');
    return {
        __typename: 'InfiniteCVLPct',
        isInfinite: overrides && overrides.hasOwnProperty('isInfinite') ? overrides.isInfinite! : faker.datatype.boolean(),
    };
};

export const mockInterest = (overrides?: Partial<Interest>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Interest' } & Interest => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Interest');
    return {
        __typename: 'Interest',
        outstanding: overrides && overrides.hasOwnProperty('outstanding') ? overrides.outstanding! : relationshipsToOmit.has('Outstanding') ? {} as Outstanding : mockOutstanding({}, relationshipsToOmit),
        outstandingPayable: overrides && overrides.hasOwnProperty('outstandingPayable') ? overrides.outstandingPayable! : relationshipsToOmit.has('Outstanding') ? {} as Outstanding : mockOutstanding({}, relationshipsToOmit),
        total: overrides && overrides.hasOwnProperty('total') ? overrides.total! : relationshipsToOmit.has('Total') ? {} as Total : mockTotal({}, relationshipsToOmit),
    };
};

export const mockJournalEntry = (overrides?: Partial<JournalEntry>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'JournalEntry' } & JournalEntry => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('JournalEntry');
    return {
        __typename: 'JournalEntry',
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : relationshipsToOmit.has('BtcAmount') ? {} as BtcAmount : mockBtcAmount({}, relationshipsToOmit),
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        description: overrides && overrides.hasOwnProperty('description') ? overrides.description! : generateMockValue.description(),
        direction: overrides && overrides.hasOwnProperty('direction') ? overrides.direction! : DebitOrCredit.Credit,
        entryId: overrides && overrides.hasOwnProperty('entryId') ? overrides.entryId! : generateMockValue.uuid(),
        entryType: overrides && overrides.hasOwnProperty('entryType') ? overrides.entryType! : faker.lorem.word(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        layer: overrides && overrides.hasOwnProperty('layer') ? overrides.layer! : Layer.Encumbrance,
        ledgerAccount: overrides && overrides.hasOwnProperty('ledgerAccount') ? overrides.ledgerAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        ledgerTransaction: overrides && overrides.hasOwnProperty('ledgerTransaction') ? overrides.ledgerTransaction! : relationshipsToOmit.has('LedgerTransaction') ? {} as LedgerTransaction : mockLedgerTransaction({}, relationshipsToOmit),
        txId: overrides && overrides.hasOwnProperty('txId') ? overrides.txId! : generateMockValue.uuid(),
    };
};

export const mockJournalEntryConnection = (overrides?: Partial<JournalEntryConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'JournalEntryConnection' } & JournalEntryConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('JournalEntryConnection');
    return {
        __typename: 'JournalEntryConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('JournalEntryEdge') ? {} as JournalEntryEdge : mockJournalEntryEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('JournalEntry') ? {} as JournalEntry : mockJournalEntry({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockJournalEntryEdge = (overrides?: Partial<JournalEntryEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'JournalEntryEdge' } & JournalEntryEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('JournalEntryEdge');
    return {
        __typename: 'JournalEntryEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('JournalEntry') ? {} as JournalEntry : mockJournalEntry({}, relationshipsToOmit),
    };
};

export const mockKomainuConfig = (overrides?: Partial<KomainuConfig>, _relationshipsToOmit: Set<string> = new Set()): KomainuConfig => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('KomainuConfig');
    return {
        apiKey: overrides && overrides.hasOwnProperty('apiKey') ? overrides.apiKey! : faker.lorem.word(),
        apiSecret: overrides && overrides.hasOwnProperty('apiSecret') ? overrides.apiSecret! : faker.lorem.word(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        secretKey: overrides && overrides.hasOwnProperty('secretKey') ? overrides.secretKey! : faker.lorem.word(),
        testingInstance: overrides && overrides.hasOwnProperty('testingInstance') ? overrides.testingInstance! : faker.datatype.boolean(),
        webhookSecret: overrides && overrides.hasOwnProperty('webhookSecret') ? overrides.webhookSecret! : faker.lorem.word(),
    };
};

export const mockLedgerAccount = (overrides?: Partial<LedgerAccount>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LedgerAccount' } & LedgerAccount => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LedgerAccount');
    return {
        __typename: 'LedgerAccount',
        ancestors: overrides && overrides.hasOwnProperty('ancestors') ? overrides.ancestors! : [relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit)],
        balanceRange: overrides && overrides.hasOwnProperty('balanceRange') ? overrides.balanceRange! : relationshipsToOmit.has('BtcLedgerAccountBalanceRange') ? {} as BtcLedgerAccountBalanceRange : mockBtcLedgerAccountBalanceRange({}, relationshipsToOmit),
        children: overrides && overrides.hasOwnProperty('children') ? overrides.children! : [relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit)],
        closestAccountWithCode: overrides && overrides.hasOwnProperty('closestAccountWithCode') ? overrides.closestAccountWithCode! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        code: overrides && overrides.hasOwnProperty('code') ? overrides.code! : faker.lorem.word(),
        entity: overrides && overrides.hasOwnProperty('entity') ? overrides.entity! : relationshipsToOmit.has('Collateral') ? {} as Collateral : mockCollateral({}, relationshipsToOmit),
        history: overrides && overrides.hasOwnProperty('history') ? overrides.history! : relationshipsToOmit.has('JournalEntryConnection') ? {} as JournalEntryConnection : mockJournalEntryConnection({}, relationshipsToOmit),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        isRootAccount: overrides && overrides.hasOwnProperty('isRootAccount') ? overrides.isRootAccount! : faker.datatype.boolean(),
        ledgerAccountId: overrides && overrides.hasOwnProperty('ledgerAccountId') ? overrides.ledgerAccountId! : generateMockValue.uuid(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        normalBalanceType: overrides && overrides.hasOwnProperty('normalBalanceType') ? overrides.normalBalanceType! : DebitOrCredit.Credit,
    };
};

export const mockLedgerAccountBalanceRangeByCurrency = (overrides?: Partial<LedgerAccountBalanceRangeByCurrency>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LedgerAccountBalanceRangeByCurrency' } & LedgerAccountBalanceRangeByCurrency => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LedgerAccountBalanceRangeByCurrency');
    return {
        __typename: 'LedgerAccountBalanceRangeByCurrency',
        btc: overrides && overrides.hasOwnProperty('btc') ? overrides.btc! : relationshipsToOmit.has('BtcLedgerAccountBalanceRange') ? {} as BtcLedgerAccountBalanceRange : mockBtcLedgerAccountBalanceRange({}, relationshipsToOmit),
        usd: overrides && overrides.hasOwnProperty('usd') ? overrides.usd! : relationshipsToOmit.has('UsdLedgerAccountBalanceRange') ? {} as UsdLedgerAccountBalanceRange : mockUsdLedgerAccountBalanceRange({}, relationshipsToOmit),
    };
};

export const mockLedgerAccountCsvCreateInput = (overrides?: Partial<LedgerAccountCsvCreateInput>, _relationshipsToOmit: Set<string> = new Set()): LedgerAccountCsvCreateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LedgerAccountCsvCreateInput');
    return {
        ledgerAccountId: overrides && overrides.hasOwnProperty('ledgerAccountId') ? overrides.ledgerAccountId! : generateMockValue.uuid(),
    };
};

export const mockLedgerAccountCsvCreatePayload = (overrides?: Partial<LedgerAccountCsvCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LedgerAccountCsvCreatePayload' } & LedgerAccountCsvCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LedgerAccountCsvCreatePayload');
    return {
        __typename: 'LedgerAccountCsvCreatePayload',
        accountingCsvDocument: overrides && overrides.hasOwnProperty('accountingCsvDocument') ? overrides.accountingCsvDocument! : relationshipsToOmit.has('AccountingCsvDocument') ? {} as AccountingCsvDocument : mockAccountingCsvDocument({}, relationshipsToOmit),
    };
};

export const mockLedgerTransaction = (overrides?: Partial<LedgerTransaction>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LedgerTransaction' } & LedgerTransaction => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LedgerTransaction');
    return {
        __typename: 'LedgerTransaction',
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        description: overrides && overrides.hasOwnProperty('description') ? overrides.description! : generateMockValue.description(),
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        entity: overrides && overrides.hasOwnProperty('entity') ? overrides.entity! : relationshipsToOmit.has('CreditFacilityDisbursal') ? {} as CreditFacilityDisbursal : mockCreditFacilityDisbursal({}, relationshipsToOmit),
        entries: overrides && overrides.hasOwnProperty('entries') ? overrides.entries! : [relationshipsToOmit.has('JournalEntry') ? {} as JournalEntry : mockJournalEntry({}, relationshipsToOmit)],
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        initiatedBy: overrides && overrides.hasOwnProperty('initiatedBy') ? overrides.initiatedBy! : relationshipsToOmit.has('System') ? {} as System : mockSystem({}, relationshipsToOmit),
        ledgerTransactionId: overrides && overrides.hasOwnProperty('ledgerTransactionId') ? overrides.ledgerTransactionId! : generateMockValue.uuid(),
    };
};

export const mockLedgerTransactionConnection = (overrides?: Partial<LedgerTransactionConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LedgerTransactionConnection' } & LedgerTransactionConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LedgerTransactionConnection');
    return {
        __typename: 'LedgerTransactionConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('LedgerTransactionEdge') ? {} as LedgerTransactionEdge : mockLedgerTransactionEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('LedgerTransaction') ? {} as LedgerTransaction : mockLedgerTransaction({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockLedgerTransactionEdge = (overrides?: Partial<LedgerTransactionEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LedgerTransactionEdge' } & LedgerTransactionEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LedgerTransactionEdge');
    return {
        __typename: 'LedgerTransactionEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('LedgerTransaction') ? {} as LedgerTransaction : mockLedgerTransaction({}, relationshipsToOmit),
    };
};

export const mockLiquidation = (overrides?: Partial<Liquidation>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Liquidation' } & Liquidation => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Liquidation');
    return {
        __typename: 'Liquidation',
        completed: overrides && overrides.hasOwnProperty('completed') ? overrides.completed! : faker.datatype.boolean(),
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        creditFacility: overrides && overrides.hasOwnProperty('creditFacility') ? overrides.creditFacility! : relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit),
        creditFacilityId: overrides && overrides.hasOwnProperty('creditFacilityId') ? overrides.creditFacilityId! : generateMockValue.uuid(),
        expectedToReceive: overrides && overrides.hasOwnProperty('expectedToReceive') ? overrides.expectedToReceive! : generateMockValue.usdCents(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        liquidationId: overrides && overrides.hasOwnProperty('liquidationId') ? overrides.liquidationId! : generateMockValue.uuid(),
        receivedPayment: overrides && overrides.hasOwnProperty('receivedPayment') ? overrides.receivedPayment! : [relationshipsToOmit.has('LiquidationPaymentReceived') ? {} as LiquidationPaymentReceived : mockLiquidationPaymentReceived({}, relationshipsToOmit)],
        receivedTotal: overrides && overrides.hasOwnProperty('receivedTotal') ? overrides.receivedTotal! : generateMockValue.usdCents(),
        sentCollateral: overrides && overrides.hasOwnProperty('sentCollateral') ? overrides.sentCollateral! : [relationshipsToOmit.has('LiquidationCollateralSent') ? {} as LiquidationCollateralSent : mockLiquidationCollateralSent({}, relationshipsToOmit)],
        sentTotal: overrides && overrides.hasOwnProperty('sentTotal') ? overrides.sentTotal! : generateMockValue.satoshis(),
    };
};

export const mockLiquidationCollateralSent = (overrides?: Partial<LiquidationCollateralSent>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LiquidationCollateralSent' } & LiquidationCollateralSent => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LiquidationCollateralSent');
    return {
        __typename: 'LiquidationCollateralSent',
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.satoshis(),
        ledgerTxId: overrides && overrides.hasOwnProperty('ledgerTxId') ? overrides.ledgerTxId! : generateMockValue.uuid(),
    };
};

export const mockLiquidationConnection = (overrides?: Partial<LiquidationConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LiquidationConnection' } & LiquidationConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LiquidationConnection');
    return {
        __typename: 'LiquidationConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('LiquidationEdge') ? {} as LiquidationEdge : mockLiquidationEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('Liquidation') ? {} as Liquidation : mockLiquidation({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockLiquidationEdge = (overrides?: Partial<LiquidationEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LiquidationEdge' } & LiquidationEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LiquidationEdge');
    return {
        __typename: 'LiquidationEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('Liquidation') ? {} as Liquidation : mockLiquidation({}, relationshipsToOmit),
    };
};

export const mockLiquidationPaymentReceived = (overrides?: Partial<LiquidationPaymentReceived>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LiquidationPaymentReceived' } & LiquidationPaymentReceived => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LiquidationPaymentReceived');
    return {
        __typename: 'LiquidationPaymentReceived',
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.usdCents(),
        ledgerTxId: overrides && overrides.hasOwnProperty('ledgerTxId') ? overrides.ledgerTxId! : generateMockValue.uuid(),
    };
};

export const mockLiquidationRecordCollateralSentInput = (overrides?: Partial<LiquidationRecordCollateralSentInput>, _relationshipsToOmit: Set<string> = new Set()): LiquidationRecordCollateralSentInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LiquidationRecordCollateralSentInput');
    return {
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.satoshis(),
        liquidationId: overrides && overrides.hasOwnProperty('liquidationId') ? overrides.liquidationId! : generateMockValue.uuid(),
    };
};

export const mockLiquidationRecordCollateralSentPayload = (overrides?: Partial<LiquidationRecordCollateralSentPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LiquidationRecordCollateralSentPayload' } & LiquidationRecordCollateralSentPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LiquidationRecordCollateralSentPayload');
    return {
        __typename: 'LiquidationRecordCollateralSentPayload',
        liquidation: overrides && overrides.hasOwnProperty('liquidation') ? overrides.liquidation! : relationshipsToOmit.has('Liquidation') ? {} as Liquidation : mockLiquidation({}, relationshipsToOmit),
    };
};

export const mockLiquidationRecordPaymentReceivedInput = (overrides?: Partial<LiquidationRecordPaymentReceivedInput>, _relationshipsToOmit: Set<string> = new Set()): LiquidationRecordPaymentReceivedInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LiquidationRecordPaymentReceivedInput');
    return {
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.usdCents(),
        liquidationId: overrides && overrides.hasOwnProperty('liquidationId') ? overrides.liquidationId! : generateMockValue.uuid(),
    };
};

export const mockLiquidationRecordPaymentReceivedPayload = (overrides?: Partial<LiquidationRecordPaymentReceivedPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LiquidationRecordPaymentReceivedPayload' } & LiquidationRecordPaymentReceivedPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LiquidationRecordPaymentReceivedPayload');
    return {
        __typename: 'LiquidationRecordPaymentReceivedPayload',
        liquidation: overrides && overrides.hasOwnProperty('liquidation') ? overrides.liquidation! : relationshipsToOmit.has('Liquidation') ? {} as Liquidation : mockLiquidation({}, relationshipsToOmit),
    };
};

export const mockLoan = (overrides?: Partial<Loan>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Loan' } & Loan => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Loan');
    return {
        __typename: 'Loan',
        collateralToMatchInitialCvl: overrides && overrides.hasOwnProperty('collateralToMatchInitialCvl') ? overrides.collateralToMatchInitialCvl! : generateMockValue.satoshis(),
    };
};

export const mockLoanAgreement = (overrides?: Partial<LoanAgreement>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LoanAgreement' } & LoanAgreement => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LoanAgreement');
    return {
        __typename: 'LoanAgreement',
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : LoanAgreementStatus.Completed,
    };
};

export const mockLoanAgreementDownloadLinksGenerateInput = (overrides?: Partial<LoanAgreementDownloadLinksGenerateInput>, _relationshipsToOmit: Set<string> = new Set()): LoanAgreementDownloadLinksGenerateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LoanAgreementDownloadLinksGenerateInput');
    return {
        loanAgreementId: overrides && overrides.hasOwnProperty('loanAgreementId') ? overrides.loanAgreementId! : generateMockValue.uuid(),
    };
};

export const mockLoanAgreementDownloadLinksGeneratePayload = (overrides?: Partial<LoanAgreementDownloadLinksGeneratePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LoanAgreementDownloadLinksGeneratePayload' } & LoanAgreementDownloadLinksGeneratePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LoanAgreementDownloadLinksGeneratePayload');
    return {
        __typename: 'LoanAgreementDownloadLinksGeneratePayload',
        link: overrides && overrides.hasOwnProperty('link') ? overrides.link! : faker.lorem.word(),
        loanAgreementId: overrides && overrides.hasOwnProperty('loanAgreementId') ? overrides.loanAgreementId! : generateMockValue.uuid(),
    };
};

export const mockLoanAgreementGenerateInput = (overrides?: Partial<LoanAgreementGenerateInput>, _relationshipsToOmit: Set<string> = new Set()): LoanAgreementGenerateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LoanAgreementGenerateInput');
    return {
        customerId: overrides && overrides.hasOwnProperty('customerId') ? overrides.customerId! : generateMockValue.uuid(),
    };
};

export const mockLoanAgreementGeneratePayload = (overrides?: Partial<LoanAgreementGeneratePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LoanAgreementGeneratePayload' } & LoanAgreementGeneratePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LoanAgreementGeneratePayload');
    return {
        __typename: 'LoanAgreementGeneratePayload',
        loanAgreement: overrides && overrides.hasOwnProperty('loanAgreement') ? overrides.loanAgreement! : relationshipsToOmit.has('LoanAgreement') ? {} as LoanAgreement : mockLoanAgreement({}, relationshipsToOmit),
    };
};

export const mockManualTransactionEntryInput = (overrides?: Partial<ManualTransactionEntryInput>, _relationshipsToOmit: Set<string> = new Set()): ManualTransactionEntryInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ManualTransactionEntryInput');
    return {
        accountRef: overrides && overrides.hasOwnProperty('accountRef') ? overrides.accountRef! : faker.lorem.word(),
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : faker.lorem.word(),
        currency: overrides && overrides.hasOwnProperty('currency') ? overrides.currency! : faker.lorem.word(),
        description: overrides && overrides.hasOwnProperty('description') ? overrides.description! : generateMockValue.description(),
        direction: overrides && overrides.hasOwnProperty('direction') ? overrides.direction! : DebitOrCredit.Credit,
    };
};

export const mockManualTransactionExecuteInput = (overrides?: Partial<ManualTransactionExecuteInput>, _relationshipsToOmit: Set<string> = new Set()): ManualTransactionExecuteInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ManualTransactionExecuteInput');
    return {
        description: overrides && overrides.hasOwnProperty('description') ? overrides.description! : generateMockValue.description(),
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        entries: overrides && overrides.hasOwnProperty('entries') ? overrides.entries! : [relationshipsToOmit.has('ManualTransactionEntryInput') ? {} as ManualTransactionEntryInput : mockManualTransactionEntryInput({}, relationshipsToOmit)],
        reference: overrides && overrides.hasOwnProperty('reference') ? overrides.reference! : generateMockValue.reference(),
    };
};

export const mockManualTransactionExecutePayload = (overrides?: Partial<ManualTransactionExecutePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ManualTransactionExecutePayload' } & ManualTransactionExecutePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ManualTransactionExecutePayload');
    return {
        __typename: 'ManualTransactionExecutePayload',
        transaction: overrides && overrides.hasOwnProperty('transaction') ? overrides.transaction! : relationshipsToOmit.has('LedgerTransaction') ? {} as LedgerTransaction : mockLedgerTransaction({}, relationshipsToOmit),
    };
};

export const mockMe = (overrides?: Partial<Me>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Me' } & Me => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Me');
    return {
        __typename: 'Me',
        user: overrides && overrides.hasOwnProperty('user') ? overrides.user! : relationshipsToOmit.has('User') ? {} as User : mockUser({}, relationshipsToOmit),
        userCanCreateCustomer: overrides && overrides.hasOwnProperty('userCanCreateCustomer') ? overrides.userCanCreateCustomer! : faker.datatype.boolean(),
        userCanCreateTermsTemplate: overrides && overrides.hasOwnProperty('userCanCreateTermsTemplate') ? overrides.userCanCreateTermsTemplate! : faker.datatype.boolean(),
        userCanCreateUser: overrides && overrides.hasOwnProperty('userCanCreateUser') ? overrides.userCanCreateUser! : faker.datatype.boolean(),
        visibleNavigationItems: overrides && overrides.hasOwnProperty('visibleNavigationItems') ? overrides.visibleNavigationItems! : relationshipsToOmit.has('VisibleNavigationItems') ? {} as VisibleNavigationItems : mockVisibleNavigationItems({}, relationshipsToOmit),
    };
};

export const mockMutation = (overrides?: Partial<Mutation>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Mutation' } & Mutation => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Mutation');
    return {
        __typename: 'Mutation',
        accountingCsvDownloadLinkGenerate: overrides && overrides.hasOwnProperty('accountingCsvDownloadLinkGenerate') ? overrides.accountingCsvDownloadLinkGenerate! : relationshipsToOmit.has('AccountingCsvDownloadLinkGeneratePayload') ? {} as AccountingCsvDownloadLinkGeneratePayload : mockAccountingCsvDownloadLinkGeneratePayload({}, relationshipsToOmit),
        approvalProcessApprove: overrides && overrides.hasOwnProperty('approvalProcessApprove') ? overrides.approvalProcessApprove! : relationshipsToOmit.has('ApprovalProcessApprovePayload') ? {} as ApprovalProcessApprovePayload : mockApprovalProcessApprovePayload({}, relationshipsToOmit),
        approvalProcessDeny: overrides && overrides.hasOwnProperty('approvalProcessDeny') ? overrides.approvalProcessDeny! : relationshipsToOmit.has('ApprovalProcessDenyPayload') ? {} as ApprovalProcessDenyPayload : mockApprovalProcessDenyPayload({}, relationshipsToOmit),
        balanceSheetConfigure: overrides && overrides.hasOwnProperty('balanceSheetConfigure') ? overrides.balanceSheetConfigure! : relationshipsToOmit.has('BalanceSheetModuleConfigurePayload') ? {} as BalanceSheetModuleConfigurePayload : mockBalanceSheetModuleConfigurePayload({}, relationshipsToOmit),
        chartOfAccountsAddChildNode: overrides && overrides.hasOwnProperty('chartOfAccountsAddChildNode') ? overrides.chartOfAccountsAddChildNode! : relationshipsToOmit.has('ChartOfAccountsAddChildNodePayload') ? {} as ChartOfAccountsAddChildNodePayload : mockChartOfAccountsAddChildNodePayload({}, relationshipsToOmit),
        chartOfAccountsAddRootNode: overrides && overrides.hasOwnProperty('chartOfAccountsAddRootNode') ? overrides.chartOfAccountsAddRootNode! : relationshipsToOmit.has('ChartOfAccountsAddRootNodePayload') ? {} as ChartOfAccountsAddRootNodePayload : mockChartOfAccountsAddRootNodePayload({}, relationshipsToOmit),
        chartOfAccountsCsvImport: overrides && overrides.hasOwnProperty('chartOfAccountsCsvImport') ? overrides.chartOfAccountsCsvImport! : relationshipsToOmit.has('ChartOfAccountsCsvImportPayload') ? {} as ChartOfAccountsCsvImportPayload : mockChartOfAccountsCsvImportPayload({}, relationshipsToOmit),
        committeeAddUser: overrides && overrides.hasOwnProperty('committeeAddUser') ? overrides.committeeAddUser! : relationshipsToOmit.has('CommitteeAddUserPayload') ? {} as CommitteeAddUserPayload : mockCommitteeAddUserPayload({}, relationshipsToOmit),
        committeeCreate: overrides && overrides.hasOwnProperty('committeeCreate') ? overrides.committeeCreate! : relationshipsToOmit.has('CommitteeCreatePayload') ? {} as CommitteeCreatePayload : mockCommitteeCreatePayload({}, relationshipsToOmit),
        committeeRemoveUser: overrides && overrides.hasOwnProperty('committeeRemoveUser') ? overrides.committeeRemoveUser! : relationshipsToOmit.has('CommitteeRemoveUserPayload') ? {} as CommitteeRemoveUserPayload : mockCommitteeRemoveUserPayload({}, relationshipsToOmit),
        creditFacilityCollateralUpdate: overrides && overrides.hasOwnProperty('creditFacilityCollateralUpdate') ? overrides.creditFacilityCollateralUpdate! : relationshipsToOmit.has('CreditFacilityCollateralUpdatePayload') ? {} as CreditFacilityCollateralUpdatePayload : mockCreditFacilityCollateralUpdatePayload({}, relationshipsToOmit),
        creditFacilityComplete: overrides && overrides.hasOwnProperty('creditFacilityComplete') ? overrides.creditFacilityComplete! : relationshipsToOmit.has('CreditFacilityCompletePayload') ? {} as CreditFacilityCompletePayload : mockCreditFacilityCompletePayload({}, relationshipsToOmit),
        creditFacilityDisbursalInitiate: overrides && overrides.hasOwnProperty('creditFacilityDisbursalInitiate') ? overrides.creditFacilityDisbursalInitiate! : relationshipsToOmit.has('CreditFacilityDisbursalInitiatePayload') ? {} as CreditFacilityDisbursalInitiatePayload : mockCreditFacilityDisbursalInitiatePayload({}, relationshipsToOmit),
        creditFacilityPartialPaymentRecord: overrides && overrides.hasOwnProperty('creditFacilityPartialPaymentRecord') ? overrides.creditFacilityPartialPaymentRecord! : relationshipsToOmit.has('CreditFacilityPartialPaymentRecordPayload') ? {} as CreditFacilityPartialPaymentRecordPayload : mockCreditFacilityPartialPaymentRecordPayload({}, relationshipsToOmit),
        creditFacilityPartialPaymentWithDateRecord: overrides && overrides.hasOwnProperty('creditFacilityPartialPaymentWithDateRecord') ? overrides.creditFacilityPartialPaymentWithDateRecord! : relationshipsToOmit.has('CreditFacilityPartialPaymentRecordPayload') ? {} as CreditFacilityPartialPaymentRecordPayload : mockCreditFacilityPartialPaymentRecordPayload({}, relationshipsToOmit),
        creditFacilityProposalCreate: overrides && overrides.hasOwnProperty('creditFacilityProposalCreate') ? overrides.creditFacilityProposalCreate! : relationshipsToOmit.has('CreditFacilityProposalCreatePayload') ? {} as CreditFacilityProposalCreatePayload : mockCreditFacilityProposalCreatePayload({}, relationshipsToOmit),
        creditFacilityProposalCustomerApprovalConclude: overrides && overrides.hasOwnProperty('creditFacilityProposalCustomerApprovalConclude') ? overrides.creditFacilityProposalCustomerApprovalConclude! : relationshipsToOmit.has('CreditFacilityProposalCustomerApprovalConcludePayload') ? {} as CreditFacilityProposalCustomerApprovalConcludePayload : mockCreditFacilityProposalCustomerApprovalConcludePayload({}, relationshipsToOmit),
        creditModuleConfigure: overrides && overrides.hasOwnProperty('creditModuleConfigure') ? overrides.creditModuleConfigure! : relationshipsToOmit.has('CreditModuleConfigurePayload') ? {} as CreditModuleConfigurePayload : mockCreditModuleConfigurePayload({}, relationshipsToOmit),
        custodianConfigUpdate: overrides && overrides.hasOwnProperty('custodianConfigUpdate') ? overrides.custodianConfigUpdate! : relationshipsToOmit.has('CustodianConfigUpdatePayload') ? {} as CustodianConfigUpdatePayload : mockCustodianConfigUpdatePayload({}, relationshipsToOmit),
        custodianCreate: overrides && overrides.hasOwnProperty('custodianCreate') ? overrides.custodianCreate! : relationshipsToOmit.has('CustodianCreatePayload') ? {} as CustodianCreatePayload : mockCustodianCreatePayload({}, relationshipsToOmit),
        customerCreate: overrides && overrides.hasOwnProperty('customerCreate') ? overrides.customerCreate! : relationshipsToOmit.has('CustomerCreatePayload') ? {} as CustomerCreatePayload : mockCustomerCreatePayload({}, relationshipsToOmit),
        customerDocumentArchive: overrides && overrides.hasOwnProperty('customerDocumentArchive') ? overrides.customerDocumentArchive! : relationshipsToOmit.has('CustomerDocumentArchivePayload') ? {} as CustomerDocumentArchivePayload : mockCustomerDocumentArchivePayload({}, relationshipsToOmit),
        customerDocumentAttach: overrides && overrides.hasOwnProperty('customerDocumentAttach') ? overrides.customerDocumentAttach! : relationshipsToOmit.has('CustomerDocumentCreatePayload') ? {} as CustomerDocumentCreatePayload : mockCustomerDocumentCreatePayload({}, relationshipsToOmit),
        customerDocumentDelete: overrides && overrides.hasOwnProperty('customerDocumentDelete') ? overrides.customerDocumentDelete! : relationshipsToOmit.has('CustomerDocumentDeletePayload') ? {} as CustomerDocumentDeletePayload : mockCustomerDocumentDeletePayload({}, relationshipsToOmit),
        customerDocumentDownloadLinkGenerate: overrides && overrides.hasOwnProperty('customerDocumentDownloadLinkGenerate') ? overrides.customerDocumentDownloadLinkGenerate! : relationshipsToOmit.has('CustomerDocumentDownloadLinksGeneratePayload') ? {} as CustomerDocumentDownloadLinksGeneratePayload : mockCustomerDocumentDownloadLinksGeneratePayload({}, relationshipsToOmit),
        customerEmailUpdate: overrides && overrides.hasOwnProperty('customerEmailUpdate') ? overrides.customerEmailUpdate! : relationshipsToOmit.has('CustomerEmailUpdatePayload') ? {} as CustomerEmailUpdatePayload : mockCustomerEmailUpdatePayload({}, relationshipsToOmit),
        customerTelegramIdUpdate: overrides && overrides.hasOwnProperty('customerTelegramIdUpdate') ? overrides.customerTelegramIdUpdate! : relationshipsToOmit.has('CustomerTelegramIdUpdatePayload') ? {} as CustomerTelegramIdUpdatePayload : mockCustomerTelegramIdUpdatePayload({}, relationshipsToOmit),
        depositAccountClose: overrides && overrides.hasOwnProperty('depositAccountClose') ? overrides.depositAccountClose! : relationshipsToOmit.has('DepositAccountClosePayload') ? {} as DepositAccountClosePayload : mockDepositAccountClosePayload({}, relationshipsToOmit),
        depositAccountCreate: overrides && overrides.hasOwnProperty('depositAccountCreate') ? overrides.depositAccountCreate! : relationshipsToOmit.has('DepositAccountCreatePayload') ? {} as DepositAccountCreatePayload : mockDepositAccountCreatePayload({}, relationshipsToOmit),
        depositAccountFreeze: overrides && overrides.hasOwnProperty('depositAccountFreeze') ? overrides.depositAccountFreeze! : relationshipsToOmit.has('DepositAccountFreezePayload') ? {} as DepositAccountFreezePayload : mockDepositAccountFreezePayload({}, relationshipsToOmit),
        depositAccountUnfreeze: overrides && overrides.hasOwnProperty('depositAccountUnfreeze') ? overrides.depositAccountUnfreeze! : relationshipsToOmit.has('DepositAccountUnfreezePayload') ? {} as DepositAccountUnfreezePayload : mockDepositAccountUnfreezePayload({}, relationshipsToOmit),
        depositModuleConfigure: overrides && overrides.hasOwnProperty('depositModuleConfigure') ? overrides.depositModuleConfigure! : relationshipsToOmit.has('DepositModuleConfigurePayload') ? {} as DepositModuleConfigurePayload : mockDepositModuleConfigurePayload({}, relationshipsToOmit),
        depositRecord: overrides && overrides.hasOwnProperty('depositRecord') ? overrides.depositRecord! : relationshipsToOmit.has('DepositRecordPayload') ? {} as DepositRecordPayload : mockDepositRecordPayload({}, relationshipsToOmit),
        depositRevert: overrides && overrides.hasOwnProperty('depositRevert') ? overrides.depositRevert! : relationshipsToOmit.has('DepositRevertPayload') ? {} as DepositRevertPayload : mockDepositRevertPayload({}, relationshipsToOmit),
        fiscalYearClose: overrides && overrides.hasOwnProperty('fiscalYearClose') ? overrides.fiscalYearClose! : relationshipsToOmit.has('FiscalYearClosePayload') ? {} as FiscalYearClosePayload : mockFiscalYearClosePayload({}, relationshipsToOmit),
        fiscalYearCloseMonth: overrides && overrides.hasOwnProperty('fiscalYearCloseMonth') ? overrides.fiscalYearCloseMonth! : relationshipsToOmit.has('FiscalYearCloseMonthPayload') ? {} as FiscalYearCloseMonthPayload : mockFiscalYearCloseMonthPayload({}, relationshipsToOmit),
        fiscalYearInit: overrides && overrides.hasOwnProperty('fiscalYearInit') ? overrides.fiscalYearInit! : relationshipsToOmit.has('FiscalYearInitPayload') ? {} as FiscalYearInitPayload : mockFiscalYearInitPayload({}, relationshipsToOmit),
        fiscalYearOpenNext: overrides && overrides.hasOwnProperty('fiscalYearOpenNext') ? overrides.fiscalYearOpenNext! : relationshipsToOmit.has('FiscalYearOpenNextPayload') ? {} as FiscalYearOpenNextPayload : mockFiscalYearOpenNextPayload({}, relationshipsToOmit),
        ledgerAccountCsvCreate: overrides && overrides.hasOwnProperty('ledgerAccountCsvCreate') ? overrides.ledgerAccountCsvCreate! : relationshipsToOmit.has('LedgerAccountCsvCreatePayload') ? {} as LedgerAccountCsvCreatePayload : mockLedgerAccountCsvCreatePayload({}, relationshipsToOmit),
        liquidationRecordCollateralSent: overrides && overrides.hasOwnProperty('liquidationRecordCollateralSent') ? overrides.liquidationRecordCollateralSent! : relationshipsToOmit.has('LiquidationRecordCollateralSentPayload') ? {} as LiquidationRecordCollateralSentPayload : mockLiquidationRecordCollateralSentPayload({}, relationshipsToOmit),
        liquidationRecordPaymentReceived: overrides && overrides.hasOwnProperty('liquidationRecordPaymentReceived') ? overrides.liquidationRecordPaymentReceived! : relationshipsToOmit.has('LiquidationRecordPaymentReceivedPayload') ? {} as LiquidationRecordPaymentReceivedPayload : mockLiquidationRecordPaymentReceivedPayload({}, relationshipsToOmit),
        loanAgreementDownloadLinkGenerate: overrides && overrides.hasOwnProperty('loanAgreementDownloadLinkGenerate') ? overrides.loanAgreementDownloadLinkGenerate! : relationshipsToOmit.has('LoanAgreementDownloadLinksGeneratePayload') ? {} as LoanAgreementDownloadLinksGeneratePayload : mockLoanAgreementDownloadLinksGeneratePayload({}, relationshipsToOmit),
        loanAgreementGenerate: overrides && overrides.hasOwnProperty('loanAgreementGenerate') ? overrides.loanAgreementGenerate! : relationshipsToOmit.has('LoanAgreementGeneratePayload') ? {} as LoanAgreementGeneratePayload : mockLoanAgreementGeneratePayload({}, relationshipsToOmit),
        manualTransactionExecute: overrides && overrides.hasOwnProperty('manualTransactionExecute') ? overrides.manualTransactionExecute! : relationshipsToOmit.has('ManualTransactionExecutePayload') ? {} as ManualTransactionExecutePayload : mockManualTransactionExecutePayload({}, relationshipsToOmit),
        pendingCreditFacilityCollateralUpdate: overrides && overrides.hasOwnProperty('pendingCreditFacilityCollateralUpdate') ? overrides.pendingCreditFacilityCollateralUpdate! : relationshipsToOmit.has('PendingCreditFacilityCollateralUpdatePayload') ? {} as PendingCreditFacilityCollateralUpdatePayload : mockPendingCreditFacilityCollateralUpdatePayload({}, relationshipsToOmit),
        policyAssignCommittee: overrides && overrides.hasOwnProperty('policyAssignCommittee') ? overrides.policyAssignCommittee! : relationshipsToOmit.has('PolicyAssignCommitteePayload') ? {} as PolicyAssignCommitteePayload : mockPolicyAssignCommitteePayload({}, relationshipsToOmit),
        profitAndLossStatementConfigure: overrides && overrides.hasOwnProperty('profitAndLossStatementConfigure') ? overrides.profitAndLossStatementConfigure! : relationshipsToOmit.has('ProfitAndLossStatementModuleConfigurePayload') ? {} as ProfitAndLossStatementModuleConfigurePayload : mockProfitAndLossStatementModuleConfigurePayload({}, relationshipsToOmit),
        reportFileGenerateDownloadLink: overrides && overrides.hasOwnProperty('reportFileGenerateDownloadLink') ? overrides.reportFileGenerateDownloadLink! : relationshipsToOmit.has('ReportFileGenerateDownloadLinkPayload') ? {} as ReportFileGenerateDownloadLinkPayload : mockReportFileGenerateDownloadLinkPayload({}, relationshipsToOmit),
        roleAddPermissionSets: overrides && overrides.hasOwnProperty('roleAddPermissionSets') ? overrides.roleAddPermissionSets! : relationshipsToOmit.has('RoleAddPermissionSetsPayload') ? {} as RoleAddPermissionSetsPayload : mockRoleAddPermissionSetsPayload({}, relationshipsToOmit),
        roleCreate: overrides && overrides.hasOwnProperty('roleCreate') ? overrides.roleCreate! : relationshipsToOmit.has('RoleCreatePayload') ? {} as RoleCreatePayload : mockRoleCreatePayload({}, relationshipsToOmit),
        roleRemovePermissionSets: overrides && overrides.hasOwnProperty('roleRemovePermissionSets') ? overrides.roleRemovePermissionSets! : relationshipsToOmit.has('RoleRemovePermissionSetsPayload') ? {} as RoleRemovePermissionSetsPayload : mockRoleRemovePermissionSetsPayload({}, relationshipsToOmit),
        sumsubPermalinkCreate: overrides && overrides.hasOwnProperty('sumsubPermalinkCreate') ? overrides.sumsubPermalinkCreate! : relationshipsToOmit.has('SumsubPermalinkCreatePayload') ? {} as SumsubPermalinkCreatePayload : mockSumsubPermalinkCreatePayload({}, relationshipsToOmit),
        termsTemplateCreate: overrides && overrides.hasOwnProperty('termsTemplateCreate') ? overrides.termsTemplateCreate! : relationshipsToOmit.has('TermsTemplateCreatePayload') ? {} as TermsTemplateCreatePayload : mockTermsTemplateCreatePayload({}, relationshipsToOmit),
        termsTemplateUpdate: overrides && overrides.hasOwnProperty('termsTemplateUpdate') ? overrides.termsTemplateUpdate! : relationshipsToOmit.has('TermsTemplateUpdatePayload') ? {} as TermsTemplateUpdatePayload : mockTermsTemplateUpdatePayload({}, relationshipsToOmit),
        triggerReportRun: overrides && overrides.hasOwnProperty('triggerReportRun') ? overrides.triggerReportRun! : relationshipsToOmit.has('ReportRunCreatePayload') ? {} as ReportRunCreatePayload : mockReportRunCreatePayload({}, relationshipsToOmit),
        updateExposedConfig: overrides && overrides.hasOwnProperty('updateExposedConfig') ? overrides.updateExposedConfig! : relationshipsToOmit.has('ExposedConfigUpdatePayload') ? {} as ExposedConfigUpdatePayload : mockExposedConfigUpdatePayload({}, relationshipsToOmit),
        userCreate: overrides && overrides.hasOwnProperty('userCreate') ? overrides.userCreate! : relationshipsToOmit.has('UserCreatePayload') ? {} as UserCreatePayload : mockUserCreatePayload({}, relationshipsToOmit),
        userUpdateRole: overrides && overrides.hasOwnProperty('userUpdateRole') ? overrides.userUpdateRole! : relationshipsToOmit.has('UserUpdateRolePayload') ? {} as UserUpdateRolePayload : mockUserUpdateRolePayload({}, relationshipsToOmit),
        withdrawalCancel: overrides && overrides.hasOwnProperty('withdrawalCancel') ? overrides.withdrawalCancel! : relationshipsToOmit.has('WithdrawalCancelPayload') ? {} as WithdrawalCancelPayload : mockWithdrawalCancelPayload({}, relationshipsToOmit),
        withdrawalConfirm: overrides && overrides.hasOwnProperty('withdrawalConfirm') ? overrides.withdrawalConfirm! : relationshipsToOmit.has('WithdrawalConfirmPayload') ? {} as WithdrawalConfirmPayload : mockWithdrawalConfirmPayload({}, relationshipsToOmit),
        withdrawalInitiate: overrides && overrides.hasOwnProperty('withdrawalInitiate') ? overrides.withdrawalInitiate! : relationshipsToOmit.has('WithdrawalInitiatePayload') ? {} as WithdrawalInitiatePayload : mockWithdrawalInitiatePayload({}, relationshipsToOmit),
        withdrawalRevert: overrides && overrides.hasOwnProperty('withdrawalRevert') ? overrides.withdrawalRevert! : relationshipsToOmit.has('WithdrawalRevertPayload') ? {} as WithdrawalRevertPayload : mockWithdrawalRevertPayload({}, relationshipsToOmit),
    };
};

export const mockOutstanding = (overrides?: Partial<Outstanding>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Outstanding' } & Outstanding => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Outstanding');
    return {
        __typename: 'Outstanding',
        usdBalance: overrides && overrides.hasOwnProperty('usdBalance') ? overrides.usdBalance! : generateMockValue.usdCents(),
    };
};

export const mockPageInfo = (overrides?: Partial<PageInfo>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'PageInfo' } & PageInfo => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PageInfo');
    return {
        __typename: 'PageInfo',
        endCursor: overrides && overrides.hasOwnProperty('endCursor') ? overrides.endCursor! : generateMockValue.cursor(),
        hasNextPage: overrides && overrides.hasOwnProperty('hasNextPage') ? overrides.hasNextPage! : generateMockValue.boolean(),
        hasPreviousPage: overrides && overrides.hasOwnProperty('hasPreviousPage') ? overrides.hasPreviousPage! : generateMockValue.boolean(),
        startCursor: overrides && overrides.hasOwnProperty('startCursor') ? overrides.startCursor! : generateMockValue.cursor(),
    };
};

export const mockPaymentEntry = (overrides?: Partial<PaymentEntry>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'PaymentEntry' } & PaymentEntry => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PaymentEntry');
    return {
        __typename: 'PaymentEntry',
        payment: overrides && overrides.hasOwnProperty('payment') ? overrides.payment! : relationshipsToOmit.has('CreditFacilityPaymentAllocation') ? {} as CreditFacilityPaymentAllocation : mockCreditFacilityPaymentAllocation({}, relationshipsToOmit),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
    };
};

export const mockPendingCreditFacility = (overrides?: Partial<PendingCreditFacility>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'PendingCreditFacility' } & PendingCreditFacility => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PendingCreditFacility');
    return {
        __typename: 'PendingCreditFacility',
        approvalProcess: overrides && overrides.hasOwnProperty('approvalProcess') ? overrides.approvalProcess! : relationshipsToOmit.has('ApprovalProcess') ? {} as ApprovalProcess : mockApprovalProcess({}, relationshipsToOmit),
        approvalProcessId: overrides && overrides.hasOwnProperty('approvalProcessId') ? overrides.approvalProcessId! : generateMockValue.uuid(),
        collateral: overrides && overrides.hasOwnProperty('collateral') ? overrides.collateral! : relationshipsToOmit.has('CollateralBalance') ? {} as CollateralBalance : mockCollateralBalance({}, relationshipsToOmit),
        collateralToMatchInitialCvl: overrides && overrides.hasOwnProperty('collateralToMatchInitialCvl') ? overrides.collateralToMatchInitialCvl! : generateMockValue.satoshis(),
        collateralizationState: overrides && overrides.hasOwnProperty('collateralizationState') ? overrides.collateralizationState! : PendingCreditFacilityCollateralizationState.FullyCollateralized,
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        creditFacilityTerms: overrides && overrides.hasOwnProperty('creditFacilityTerms') ? overrides.creditFacilityTerms! : relationshipsToOmit.has('TermValues') ? {} as TermValues : mockTermValues({}, relationshipsToOmit),
        customer: overrides && overrides.hasOwnProperty('customer') ? overrides.customer! : relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit),
        facilityAmount: overrides && overrides.hasOwnProperty('facilityAmount') ? overrides.facilityAmount! : generateMockValue.usdCents(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        pendingCreditFacilityId: overrides && overrides.hasOwnProperty('pendingCreditFacilityId') ? overrides.pendingCreditFacilityId! : generateMockValue.uuid(),
        repaymentPlan: overrides && overrides.hasOwnProperty('repaymentPlan') ? overrides.repaymentPlan! : [relationshipsToOmit.has('CreditFacilityRepaymentPlanEntry') ? {} as CreditFacilityRepaymentPlanEntry : mockCreditFacilityRepaymentPlanEntry({}, relationshipsToOmit)],
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : PendingCreditFacilityStatus.Completed,
        wallet: overrides && overrides.hasOwnProperty('wallet') ? overrides.wallet! : relationshipsToOmit.has('Wallet') ? {} as Wallet : mockWallet({}, relationshipsToOmit),
    };
};

export const mockPendingCreditFacilityCollateralUpdateInput = (overrides?: Partial<PendingCreditFacilityCollateralUpdateInput>, _relationshipsToOmit: Set<string> = new Set()): PendingCreditFacilityCollateralUpdateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PendingCreditFacilityCollateralUpdateInput');
    return {
        collateral: overrides && overrides.hasOwnProperty('collateral') ? overrides.collateral! : generateMockValue.satoshis(),
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        pendingCreditFacilityId: overrides && overrides.hasOwnProperty('pendingCreditFacilityId') ? overrides.pendingCreditFacilityId! : generateMockValue.uuid(),
    };
};

export const mockPendingCreditFacilityCollateralUpdatePayload = (overrides?: Partial<PendingCreditFacilityCollateralUpdatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'PendingCreditFacilityCollateralUpdatePayload' } & PendingCreditFacilityCollateralUpdatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PendingCreditFacilityCollateralUpdatePayload');
    return {
        __typename: 'PendingCreditFacilityCollateralUpdatePayload',
        pendingCreditFacility: overrides && overrides.hasOwnProperty('pendingCreditFacility') ? overrides.pendingCreditFacility! : relationshipsToOmit.has('PendingCreditFacility') ? {} as PendingCreditFacility : mockPendingCreditFacility({}, relationshipsToOmit),
    };
};

export const mockPendingCreditFacilityCollateralizationUpdated = (overrides?: Partial<PendingCreditFacilityCollateralizationUpdated>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'PendingCreditFacilityCollateralizationUpdated' } & PendingCreditFacilityCollateralizationUpdated => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PendingCreditFacilityCollateralizationUpdated');
    return {
        __typename: 'PendingCreditFacilityCollateralizationUpdated',
        collateral: overrides && overrides.hasOwnProperty('collateral') ? overrides.collateral! : generateMockValue.satoshis(),
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
        price: overrides && overrides.hasOwnProperty('price') ? overrides.price! : generateMockValue.usdCents(),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        state: overrides && overrides.hasOwnProperty('state') ? overrides.state! : PendingCreditFacilityCollateralizationState.FullyCollateralized,
    };
};

export const mockPendingCreditFacilityConnection = (overrides?: Partial<PendingCreditFacilityConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'PendingCreditFacilityConnection' } & PendingCreditFacilityConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PendingCreditFacilityConnection');
    return {
        __typename: 'PendingCreditFacilityConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('PendingCreditFacilityEdge') ? {} as PendingCreditFacilityEdge : mockPendingCreditFacilityEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('PendingCreditFacility') ? {} as PendingCreditFacility : mockPendingCreditFacility({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockPendingCreditFacilityEdge = (overrides?: Partial<PendingCreditFacilityEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'PendingCreditFacilityEdge' } & PendingCreditFacilityEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PendingCreditFacilityEdge');
    return {
        __typename: 'PendingCreditFacilityEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('PendingCreditFacility') ? {} as PendingCreditFacility : mockPendingCreditFacility({}, relationshipsToOmit),
    };
};

export const mockPermissionSet = (overrides?: Partial<PermissionSet>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'PermissionSet' } & PermissionSet => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PermissionSet');
    return {
        __typename: 'PermissionSet',
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        permissionSetId: overrides && overrides.hasOwnProperty('permissionSetId') ? overrides.permissionSetId! : generateMockValue.uuid(),
    };
};

export const mockPermissionSetConnection = (overrides?: Partial<PermissionSetConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'PermissionSetConnection' } & PermissionSetConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PermissionSetConnection');
    return {
        __typename: 'PermissionSetConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('PermissionSetEdge') ? {} as PermissionSetEdge : mockPermissionSetEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('PermissionSet') ? {} as PermissionSet : mockPermissionSet({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockPermissionSetEdge = (overrides?: Partial<PermissionSetEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'PermissionSetEdge' } & PermissionSetEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PermissionSetEdge');
    return {
        __typename: 'PermissionSetEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('PermissionSet') ? {} as PermissionSet : mockPermissionSet({}, relationshipsToOmit),
    };
};

export const mockPolicy = (overrides?: Partial<Policy>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Policy' } & Policy => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Policy');
    return {
        __typename: 'Policy',
        approvalProcessType: overrides && overrides.hasOwnProperty('approvalProcessType') ? overrides.approvalProcessType! : ApprovalProcessType.CreditFacilityProposalApproval,
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        policyId: overrides && overrides.hasOwnProperty('policyId') ? overrides.policyId! : generateMockValue.uuid(),
        rules: overrides && overrides.hasOwnProperty('rules') ? overrides.rules! : relationshipsToOmit.has('CommitteeThreshold') ? {} as CommitteeThreshold : mockCommitteeThreshold({}, relationshipsToOmit),
    };
};

export const mockPolicyAssignCommitteeInput = (overrides?: Partial<PolicyAssignCommitteeInput>, _relationshipsToOmit: Set<string> = new Set()): PolicyAssignCommitteeInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PolicyAssignCommitteeInput');
    return {
        committeeId: overrides && overrides.hasOwnProperty('committeeId') ? overrides.committeeId! : generateMockValue.uuid(),
        policyId: overrides && overrides.hasOwnProperty('policyId') ? overrides.policyId! : generateMockValue.uuid(),
        threshold: overrides && overrides.hasOwnProperty('threshold') ? overrides.threshold! : faker.number.int({ min: 0, max: 9999 }),
    };
};

export const mockPolicyAssignCommitteePayload = (overrides?: Partial<PolicyAssignCommitteePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'PolicyAssignCommitteePayload' } & PolicyAssignCommitteePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PolicyAssignCommitteePayload');
    return {
        __typename: 'PolicyAssignCommitteePayload',
        policy: overrides && overrides.hasOwnProperty('policy') ? overrides.policy! : relationshipsToOmit.has('Policy') ? {} as Policy : mockPolicy({}, relationshipsToOmit),
    };
};

export const mockPolicyConnection = (overrides?: Partial<PolicyConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'PolicyConnection' } & PolicyConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PolicyConnection');
    return {
        __typename: 'PolicyConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('PolicyEdge') ? {} as PolicyEdge : mockPolicyEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('Policy') ? {} as Policy : mockPolicy({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockPolicyEdge = (overrides?: Partial<PolicyEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'PolicyEdge' } & PolicyEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('PolicyEdge');
    return {
        __typename: 'PolicyEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('Policy') ? {} as Policy : mockPolicy({}, relationshipsToOmit),
    };
};

export const mockProfitAndLossModuleConfigureInput = (overrides?: Partial<ProfitAndLossModuleConfigureInput>, _relationshipsToOmit: Set<string> = new Set()): ProfitAndLossModuleConfigureInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ProfitAndLossModuleConfigureInput');
    return {
        chartOfAccountsCostOfRevenueCode: overrides && overrides.hasOwnProperty('chartOfAccountsCostOfRevenueCode') ? overrides.chartOfAccountsCostOfRevenueCode! : faker.lorem.word(),
        chartOfAccountsExpensesCode: overrides && overrides.hasOwnProperty('chartOfAccountsExpensesCode') ? overrides.chartOfAccountsExpensesCode! : faker.lorem.word(),
        chartOfAccountsRevenueCode: overrides && overrides.hasOwnProperty('chartOfAccountsRevenueCode') ? overrides.chartOfAccountsRevenueCode! : faker.lorem.word(),
    };
};

export const mockProfitAndLossStatement = (overrides?: Partial<ProfitAndLossStatement>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ProfitAndLossStatement' } & ProfitAndLossStatement => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ProfitAndLossStatement');
    return {
        __typename: 'ProfitAndLossStatement',
        categories: overrides && overrides.hasOwnProperty('categories') ? overrides.categories! : [relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit)],
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        total: overrides && overrides.hasOwnProperty('total') ? overrides.total! : relationshipsToOmit.has('LedgerAccountBalanceRangeByCurrency') ? {} as LedgerAccountBalanceRangeByCurrency : mockLedgerAccountBalanceRangeByCurrency({}, relationshipsToOmit),
    };
};

export const mockProfitAndLossStatementModuleConfig = (overrides?: Partial<ProfitAndLossStatementModuleConfig>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ProfitAndLossStatementModuleConfig' } & ProfitAndLossStatementModuleConfig => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ProfitAndLossStatementModuleConfig');
    return {
        __typename: 'ProfitAndLossStatementModuleConfig',
        chartOfAccountsCostOfRevenueCode: overrides && overrides.hasOwnProperty('chartOfAccountsCostOfRevenueCode') ? overrides.chartOfAccountsCostOfRevenueCode! : faker.lorem.word(),
        chartOfAccountsExpensesCode: overrides && overrides.hasOwnProperty('chartOfAccountsExpensesCode') ? overrides.chartOfAccountsExpensesCode! : faker.lorem.word(),
        chartOfAccountsId: overrides && overrides.hasOwnProperty('chartOfAccountsId') ? overrides.chartOfAccountsId! : generateMockValue.uuid(),
        chartOfAccountsRevenueCode: overrides && overrides.hasOwnProperty('chartOfAccountsRevenueCode') ? overrides.chartOfAccountsRevenueCode! : faker.lorem.word(),
    };
};

export const mockProfitAndLossStatementModuleConfigurePayload = (overrides?: Partial<ProfitAndLossStatementModuleConfigurePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ProfitAndLossStatementModuleConfigurePayload' } & ProfitAndLossStatementModuleConfigurePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ProfitAndLossStatementModuleConfigurePayload');
    return {
        __typename: 'ProfitAndLossStatementModuleConfigurePayload',
        profitAndLossConfig: overrides && overrides.hasOwnProperty('profitAndLossConfig') ? overrides.profitAndLossConfig! : relationshipsToOmit.has('ProfitAndLossStatementModuleConfig') ? {} as ProfitAndLossStatementModuleConfig : mockProfitAndLossStatementModuleConfig({}, relationshipsToOmit),
    };
};

export const mockQuery = (overrides?: Partial<Query>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Query' } & Query => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Query');
    return {
        __typename: 'Query',
        accountEntryCsv: overrides && overrides.hasOwnProperty('accountEntryCsv') ? overrides.accountEntryCsv! : relationshipsToOmit.has('AccountingCsvDocument') ? {} as AccountingCsvDocument : mockAccountingCsvDocument({}, relationshipsToOmit),
        approvalProcess: overrides && overrides.hasOwnProperty('approvalProcess') ? overrides.approvalProcess! : relationshipsToOmit.has('ApprovalProcess') ? {} as ApprovalProcess : mockApprovalProcess({}, relationshipsToOmit),
        approvalProcesses: overrides && overrides.hasOwnProperty('approvalProcesses') ? overrides.approvalProcesses! : relationshipsToOmit.has('ApprovalProcessConnection') ? {} as ApprovalProcessConnection : mockApprovalProcessConnection({}, relationshipsToOmit),
        audit: overrides && overrides.hasOwnProperty('audit') ? overrides.audit! : relationshipsToOmit.has('AuditEntryConnection') ? {} as AuditEntryConnection : mockAuditEntryConnection({}, relationshipsToOmit),
        balanceSheet: overrides && overrides.hasOwnProperty('balanceSheet') ? overrides.balanceSheet! : relationshipsToOmit.has('BalanceSheet') ? {} as BalanceSheet : mockBalanceSheet({}, relationshipsToOmit),
        balanceSheetConfig: overrides && overrides.hasOwnProperty('balanceSheetConfig') ? overrides.balanceSheetConfig! : relationshipsToOmit.has('BalanceSheetModuleConfig') ? {} as BalanceSheetModuleConfig : mockBalanceSheetModuleConfig({}, relationshipsToOmit),
        chartOfAccounts: overrides && overrides.hasOwnProperty('chartOfAccounts') ? overrides.chartOfAccounts! : relationshipsToOmit.has('ChartOfAccounts') ? {} as ChartOfAccounts : mockChartOfAccounts({}, relationshipsToOmit),
        committee: overrides && overrides.hasOwnProperty('committee') ? overrides.committee! : relationshipsToOmit.has('Committee') ? {} as Committee : mockCommittee({}, relationshipsToOmit),
        committees: overrides && overrides.hasOwnProperty('committees') ? overrides.committees! : relationshipsToOmit.has('CommitteeConnection') ? {} as CommitteeConnection : mockCommitteeConnection({}, relationshipsToOmit),
        creditConfig: overrides && overrides.hasOwnProperty('creditConfig') ? overrides.creditConfig! : relationshipsToOmit.has('CreditModuleConfig') ? {} as CreditModuleConfig : mockCreditModuleConfig({}, relationshipsToOmit),
        creditFacilities: overrides && overrides.hasOwnProperty('creditFacilities') ? overrides.creditFacilities! : relationshipsToOmit.has('CreditFacilityConnection') ? {} as CreditFacilityConnection : mockCreditFacilityConnection({}, relationshipsToOmit),
        creditFacility: overrides && overrides.hasOwnProperty('creditFacility') ? overrides.creditFacility! : relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit),
        creditFacilityByPublicId: overrides && overrides.hasOwnProperty('creditFacilityByPublicId') ? overrides.creditFacilityByPublicId! : relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit),
        creditFacilityProposal: overrides && overrides.hasOwnProperty('creditFacilityProposal') ? overrides.creditFacilityProposal! : relationshipsToOmit.has('CreditFacilityProposal') ? {} as CreditFacilityProposal : mockCreditFacilityProposal({}, relationshipsToOmit),
        creditFacilityProposals: overrides && overrides.hasOwnProperty('creditFacilityProposals') ? overrides.creditFacilityProposals! : relationshipsToOmit.has('CreditFacilityProposalConnection') ? {} as CreditFacilityProposalConnection : mockCreditFacilityProposalConnection({}, relationshipsToOmit),
        custodians: overrides && overrides.hasOwnProperty('custodians') ? overrides.custodians! : relationshipsToOmit.has('CustodianConnection') ? {} as CustodianConnection : mockCustodianConnection({}, relationshipsToOmit),
        customer: overrides && overrides.hasOwnProperty('customer') ? overrides.customer! : relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit),
        customerByEmail: overrides && overrides.hasOwnProperty('customerByEmail') ? overrides.customerByEmail! : relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit),
        customerByPublicId: overrides && overrides.hasOwnProperty('customerByPublicId') ? overrides.customerByPublicId! : relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit),
        customerDocument: overrides && overrides.hasOwnProperty('customerDocument') ? overrides.customerDocument! : relationshipsToOmit.has('CustomerDocument') ? {} as CustomerDocument : mockCustomerDocument({}, relationshipsToOmit),
        customers: overrides && overrides.hasOwnProperty('customers') ? overrides.customers! : relationshipsToOmit.has('CustomerConnection') ? {} as CustomerConnection : mockCustomerConnection({}, relationshipsToOmit),
        dashboard: overrides && overrides.hasOwnProperty('dashboard') ? overrides.dashboard! : relationshipsToOmit.has('Dashboard') ? {} as Dashboard : mockDashboard({}, relationshipsToOmit),
        deposit: overrides && overrides.hasOwnProperty('deposit') ? overrides.deposit! : relationshipsToOmit.has('Deposit') ? {} as Deposit : mockDeposit({}, relationshipsToOmit),
        depositAccount: overrides && overrides.hasOwnProperty('depositAccount') ? overrides.depositAccount! : relationshipsToOmit.has('DepositAccount') ? {} as DepositAccount : mockDepositAccount({}, relationshipsToOmit),
        depositAccountByPublicId: overrides && overrides.hasOwnProperty('depositAccountByPublicId') ? overrides.depositAccountByPublicId! : relationshipsToOmit.has('DepositAccount') ? {} as DepositAccount : mockDepositAccount({}, relationshipsToOmit),
        depositAccounts: overrides && overrides.hasOwnProperty('depositAccounts') ? overrides.depositAccounts! : relationshipsToOmit.has('DepositAccountConnection') ? {} as DepositAccountConnection : mockDepositAccountConnection({}, relationshipsToOmit),
        depositByPublicId: overrides && overrides.hasOwnProperty('depositByPublicId') ? overrides.depositByPublicId! : relationshipsToOmit.has('Deposit') ? {} as Deposit : mockDeposit({}, relationshipsToOmit),
        depositConfig: overrides && overrides.hasOwnProperty('depositConfig') ? overrides.depositConfig! : relationshipsToOmit.has('DepositModuleConfig') ? {} as DepositModuleConfig : mockDepositModuleConfig({}, relationshipsToOmit),
        deposits: overrides && overrides.hasOwnProperty('deposits') ? overrides.deposits! : relationshipsToOmit.has('DepositConnection') ? {} as DepositConnection : mockDepositConnection({}, relationshipsToOmit),
        disbursal: overrides && overrides.hasOwnProperty('disbursal') ? overrides.disbursal! : relationshipsToOmit.has('CreditFacilityDisbursal') ? {} as CreditFacilityDisbursal : mockCreditFacilityDisbursal({}, relationshipsToOmit),
        disbursalByPublicId: overrides && overrides.hasOwnProperty('disbursalByPublicId') ? overrides.disbursalByPublicId! : relationshipsToOmit.has('CreditFacilityDisbursal') ? {} as CreditFacilityDisbursal : mockCreditFacilityDisbursal({}, relationshipsToOmit),
        disbursals: overrides && overrides.hasOwnProperty('disbursals') ? overrides.disbursals! : relationshipsToOmit.has('CreditFacilityDisbursalConnection') ? {} as CreditFacilityDisbursalConnection : mockCreditFacilityDisbursalConnection({}, relationshipsToOmit),
        exposedConfigs: overrides && overrides.hasOwnProperty('exposedConfigs') ? overrides.exposedConfigs! : [relationshipsToOmit.has('ExposedConfigItem') ? {} as ExposedConfigItem : mockExposedConfigItem({}, relationshipsToOmit)],
        fiscalYear: overrides && overrides.hasOwnProperty('fiscalYear') ? overrides.fiscalYear! : relationshipsToOmit.has('FiscalYear') ? {} as FiscalYear : mockFiscalYear({}, relationshipsToOmit),
        fiscalYearByYear: overrides && overrides.hasOwnProperty('fiscalYearByYear') ? overrides.fiscalYearByYear! : relationshipsToOmit.has('FiscalYear') ? {} as FiscalYear : mockFiscalYear({}, relationshipsToOmit),
        fiscalYears: overrides && overrides.hasOwnProperty('fiscalYears') ? overrides.fiscalYears! : relationshipsToOmit.has('FiscalYearConnection') ? {} as FiscalYearConnection : mockFiscalYearConnection({}, relationshipsToOmit),
        journalEntries: overrides && overrides.hasOwnProperty('journalEntries') ? overrides.journalEntries! : relationshipsToOmit.has('JournalEntryConnection') ? {} as JournalEntryConnection : mockJournalEntryConnection({}, relationshipsToOmit),
        ledgerAccount: overrides && overrides.hasOwnProperty('ledgerAccount') ? overrides.ledgerAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        ledgerAccountByCode: overrides && overrides.hasOwnProperty('ledgerAccountByCode') ? overrides.ledgerAccountByCode! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        ledgerTransaction: overrides && overrides.hasOwnProperty('ledgerTransaction') ? overrides.ledgerTransaction! : relationshipsToOmit.has('LedgerTransaction') ? {} as LedgerTransaction : mockLedgerTransaction({}, relationshipsToOmit),
        ledgerTransactionsForTemplateCode: overrides && overrides.hasOwnProperty('ledgerTransactionsForTemplateCode') ? overrides.ledgerTransactionsForTemplateCode! : relationshipsToOmit.has('LedgerTransactionConnection') ? {} as LedgerTransactionConnection : mockLedgerTransactionConnection({}, relationshipsToOmit),
        liquidation: overrides && overrides.hasOwnProperty('liquidation') ? overrides.liquidation! : relationshipsToOmit.has('Liquidation') ? {} as Liquidation : mockLiquidation({}, relationshipsToOmit),
        liquidations: overrides && overrides.hasOwnProperty('liquidations') ? overrides.liquidations! : relationshipsToOmit.has('LiquidationConnection') ? {} as LiquidationConnection : mockLiquidationConnection({}, relationshipsToOmit),
        loanAgreement: overrides && overrides.hasOwnProperty('loanAgreement') ? overrides.loanAgreement! : relationshipsToOmit.has('LoanAgreement') ? {} as LoanAgreement : mockLoanAgreement({}, relationshipsToOmit),
        me: overrides && overrides.hasOwnProperty('me') ? overrides.me! : relationshipsToOmit.has('Me') ? {} as Me : mockMe({}, relationshipsToOmit),
        pendingCreditFacilities: overrides && overrides.hasOwnProperty('pendingCreditFacilities') ? overrides.pendingCreditFacilities! : relationshipsToOmit.has('PendingCreditFacilityConnection') ? {} as PendingCreditFacilityConnection : mockPendingCreditFacilityConnection({}, relationshipsToOmit),
        pendingCreditFacility: overrides && overrides.hasOwnProperty('pendingCreditFacility') ? overrides.pendingCreditFacility! : relationshipsToOmit.has('PendingCreditFacility') ? {} as PendingCreditFacility : mockPendingCreditFacility({}, relationshipsToOmit),
        permissionSets: overrides && overrides.hasOwnProperty('permissionSets') ? overrides.permissionSets! : relationshipsToOmit.has('PermissionSetConnection') ? {} as PermissionSetConnection : mockPermissionSetConnection({}, relationshipsToOmit),
        policies: overrides && overrides.hasOwnProperty('policies') ? overrides.policies! : relationshipsToOmit.has('PolicyConnection') ? {} as PolicyConnection : mockPolicyConnection({}, relationshipsToOmit),
        policy: overrides && overrides.hasOwnProperty('policy') ? overrides.policy! : relationshipsToOmit.has('Policy') ? {} as Policy : mockPolicy({}, relationshipsToOmit),
        profitAndLossStatement: overrides && overrides.hasOwnProperty('profitAndLossStatement') ? overrides.profitAndLossStatement! : relationshipsToOmit.has('ProfitAndLossStatement') ? {} as ProfitAndLossStatement : mockProfitAndLossStatement({}, relationshipsToOmit),
        profitAndLossStatementConfig: overrides && overrides.hasOwnProperty('profitAndLossStatementConfig') ? overrides.profitAndLossStatementConfig! : relationshipsToOmit.has('ProfitAndLossStatementModuleConfig') ? {} as ProfitAndLossStatementModuleConfig : mockProfitAndLossStatementModuleConfig({}, relationshipsToOmit),
        publicIdTarget: overrides && overrides.hasOwnProperty('publicIdTarget') ? overrides.publicIdTarget! : relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit),
        realtimePrice: overrides && overrides.hasOwnProperty('realtimePrice') ? overrides.realtimePrice! : relationshipsToOmit.has('RealtimePrice') ? {} as RealtimePrice : mockRealtimePrice({}, relationshipsToOmit),
        reportRun: overrides && overrides.hasOwnProperty('reportRun') ? overrides.reportRun! : relationshipsToOmit.has('ReportRun') ? {} as ReportRun : mockReportRun({}, relationshipsToOmit),
        reportRuns: overrides && overrides.hasOwnProperty('reportRuns') ? overrides.reportRuns! : relationshipsToOmit.has('ReportRunConnection') ? {} as ReportRunConnection : mockReportRunConnection({}, relationshipsToOmit),
        role: overrides && overrides.hasOwnProperty('role') ? overrides.role! : relationshipsToOmit.has('Role') ? {} as Role : mockRole({}, relationshipsToOmit),
        roles: overrides && overrides.hasOwnProperty('roles') ? overrides.roles! : relationshipsToOmit.has('RoleConnection') ? {} as RoleConnection : mockRoleConnection({}, relationshipsToOmit),
        termsTemplate: overrides && overrides.hasOwnProperty('termsTemplate') ? overrides.termsTemplate! : relationshipsToOmit.has('TermsTemplate') ? {} as TermsTemplate : mockTermsTemplate({}, relationshipsToOmit),
        termsTemplates: overrides && overrides.hasOwnProperty('termsTemplates') ? overrides.termsTemplates! : [relationshipsToOmit.has('TermsTemplate') ? {} as TermsTemplate : mockTermsTemplate({}, relationshipsToOmit)],
        transactionTemplates: overrides && overrides.hasOwnProperty('transactionTemplates') ? overrides.transactionTemplates! : relationshipsToOmit.has('TransactionTemplateConnection') ? {} as TransactionTemplateConnection : mockTransactionTemplateConnection({}, relationshipsToOmit),
        trialBalance: overrides && overrides.hasOwnProperty('trialBalance') ? overrides.trialBalance! : relationshipsToOmit.has('TrialBalance') ? {} as TrialBalance : mockTrialBalance({}, relationshipsToOmit),
        user: overrides && overrides.hasOwnProperty('user') ? overrides.user! : relationshipsToOmit.has('User') ? {} as User : mockUser({}, relationshipsToOmit),
        users: overrides && overrides.hasOwnProperty('users') ? overrides.users! : [relationshipsToOmit.has('User') ? {} as User : mockUser({}, relationshipsToOmit)],
        withdrawal: overrides && overrides.hasOwnProperty('withdrawal') ? overrides.withdrawal! : relationshipsToOmit.has('Withdrawal') ? {} as Withdrawal : mockWithdrawal({}, relationshipsToOmit),
        withdrawalByPublicId: overrides && overrides.hasOwnProperty('withdrawalByPublicId') ? overrides.withdrawalByPublicId! : relationshipsToOmit.has('Withdrawal') ? {} as Withdrawal : mockWithdrawal({}, relationshipsToOmit),
        withdrawals: overrides && overrides.hasOwnProperty('withdrawals') ? overrides.withdrawals! : relationshipsToOmit.has('WithdrawalConnection') ? {} as WithdrawalConnection : mockWithdrawalConnection({}, relationshipsToOmit),
    };
};

export const mockRealtimePrice = (overrides?: Partial<RealtimePrice>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'RealtimePrice' } & RealtimePrice => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('RealtimePrice');
    return {
        __typename: 'RealtimePrice',
        usdCentsPerBtc: overrides && overrides.hasOwnProperty('usdCentsPerBtc') ? overrides.usdCentsPerBtc! : generateMockValue.usdCents(),
    };
};

export const mockReport = (overrides?: Partial<Report>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Report' } & Report => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Report');
    return {
        __typename: 'Report',
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        externalId: overrides && overrides.hasOwnProperty('externalId') ? overrides.externalId! : faker.lorem.word(),
        files: overrides && overrides.hasOwnProperty('files') ? overrides.files! : [relationshipsToOmit.has('ReportFile') ? {} as ReportFile : mockReportFile({}, relationshipsToOmit)],
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        norm: overrides && overrides.hasOwnProperty('norm') ? overrides.norm! : faker.lorem.word(),
        reportId: overrides && overrides.hasOwnProperty('reportId') ? overrides.reportId! : generateMockValue.uuid(),
        reportRun: overrides && overrides.hasOwnProperty('reportRun') ? overrides.reportRun! : relationshipsToOmit.has('ReportRun') ? {} as ReportRun : mockReportRun({}, relationshipsToOmit),
        runId: overrides && overrides.hasOwnProperty('runId') ? overrides.runId! : generateMockValue.uuid(),
    };
};

export const mockReportFile = (overrides?: Partial<ReportFile>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ReportFile' } & ReportFile => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ReportFile');
    return {
        __typename: 'ReportFile',
        extension: overrides && overrides.hasOwnProperty('extension') ? overrides.extension! : faker.lorem.word(),
    };
};

export const mockReportFileGenerateDownloadLinkInput = (overrides?: Partial<ReportFileGenerateDownloadLinkInput>, _relationshipsToOmit: Set<string> = new Set()): ReportFileGenerateDownloadLinkInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ReportFileGenerateDownloadLinkInput');
    return {
        extension: overrides && overrides.hasOwnProperty('extension') ? overrides.extension! : faker.lorem.word(),
        reportId: overrides && overrides.hasOwnProperty('reportId') ? overrides.reportId! : generateMockValue.uuid(),
    };
};

export const mockReportFileGenerateDownloadLinkPayload = (overrides?: Partial<ReportFileGenerateDownloadLinkPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ReportFileGenerateDownloadLinkPayload' } & ReportFileGenerateDownloadLinkPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ReportFileGenerateDownloadLinkPayload');
    return {
        __typename: 'ReportFileGenerateDownloadLinkPayload',
        url: overrides && overrides.hasOwnProperty('url') ? overrides.url! : generateMockValue.url(),
    };
};

export const mockReportRun = (overrides?: Partial<ReportRun>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ReportRun' } & ReportRun => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ReportRun');
    return {
        __typename: 'ReportRun',
        endDate: overrides && overrides.hasOwnProperty('endDate') ? overrides.endDate! : generateMockValue.timestamp(),
        executionDate: overrides && overrides.hasOwnProperty('executionDate') ? overrides.executionDate! : generateMockValue.timestamp(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        reportRunId: overrides && overrides.hasOwnProperty('reportRunId') ? overrides.reportRunId! : generateMockValue.uuid(),
        reports: overrides && overrides.hasOwnProperty('reports') ? overrides.reports! : [relationshipsToOmit.has('Report') ? {} as Report : mockReport({}, relationshipsToOmit)],
        runType: overrides && overrides.hasOwnProperty('runType') ? overrides.runType! : ReportRunType.Manual,
        startDate: overrides && overrides.hasOwnProperty('startDate') ? overrides.startDate! : generateMockValue.timestamp(),
        state: overrides && overrides.hasOwnProperty('state') ? overrides.state! : ReportRunState.Failed,
    };
};

export const mockReportRunConnection = (overrides?: Partial<ReportRunConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ReportRunConnection' } & ReportRunConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ReportRunConnection');
    return {
        __typename: 'ReportRunConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('ReportRunEdge') ? {} as ReportRunEdge : mockReportRunEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('ReportRun') ? {} as ReportRun : mockReportRun({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockReportRunCreatePayload = (overrides?: Partial<ReportRunCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ReportRunCreatePayload' } & ReportRunCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ReportRunCreatePayload');
    return {
        __typename: 'ReportRunCreatePayload',
        jobId: overrides && overrides.hasOwnProperty('jobId') ? overrides.jobId! : faker.lorem.word(),
    };
};

export const mockReportRunEdge = (overrides?: Partial<ReportRunEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ReportRunEdge' } & ReportRunEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ReportRunEdge');
    return {
        __typename: 'ReportRunEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('ReportRun') ? {} as ReportRun : mockReportRun({}, relationshipsToOmit),
    };
};

export const mockRole = (overrides?: Partial<Role>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Role' } & Role => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Role');
    return {
        __typename: 'Role',
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        permissionSets: overrides && overrides.hasOwnProperty('permissionSets') ? overrides.permissionSets! : [relationshipsToOmit.has('PermissionSet') ? {} as PermissionSet : mockPermissionSet({}, relationshipsToOmit)],
        roleId: overrides && overrides.hasOwnProperty('roleId') ? overrides.roleId! : generateMockValue.uuid(),
    };
};

export const mockRoleAddPermissionSetsInput = (overrides?: Partial<RoleAddPermissionSetsInput>, _relationshipsToOmit: Set<string> = new Set()): RoleAddPermissionSetsInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('RoleAddPermissionSetsInput');
    return {
        permissionSetIds: overrides && overrides.hasOwnProperty('permissionSetIds') ? overrides.permissionSetIds! : [generateMockValue.uuid()],
        roleId: overrides && overrides.hasOwnProperty('roleId') ? overrides.roleId! : generateMockValue.uuid(),
    };
};

export const mockRoleAddPermissionSetsPayload = (overrides?: Partial<RoleAddPermissionSetsPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'RoleAddPermissionSetsPayload' } & RoleAddPermissionSetsPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('RoleAddPermissionSetsPayload');
    return {
        __typename: 'RoleAddPermissionSetsPayload',
        role: overrides && overrides.hasOwnProperty('role') ? overrides.role! : relationshipsToOmit.has('Role') ? {} as Role : mockRole({}, relationshipsToOmit),
    };
};

export const mockRoleConnection = (overrides?: Partial<RoleConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'RoleConnection' } & RoleConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('RoleConnection');
    return {
        __typename: 'RoleConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('RoleEdge') ? {} as RoleEdge : mockRoleEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('Role') ? {} as Role : mockRole({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockRoleCreateInput = (overrides?: Partial<RoleCreateInput>, _relationshipsToOmit: Set<string> = new Set()): RoleCreateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('RoleCreateInput');
    return {
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        permissionSetIds: overrides && overrides.hasOwnProperty('permissionSetIds') ? overrides.permissionSetIds! : [generateMockValue.uuid()],
    };
};

export const mockRoleCreatePayload = (overrides?: Partial<RoleCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'RoleCreatePayload' } & RoleCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('RoleCreatePayload');
    return {
        __typename: 'RoleCreatePayload',
        role: overrides && overrides.hasOwnProperty('role') ? overrides.role! : relationshipsToOmit.has('Role') ? {} as Role : mockRole({}, relationshipsToOmit),
    };
};

export const mockRoleEdge = (overrides?: Partial<RoleEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'RoleEdge' } & RoleEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('RoleEdge');
    return {
        __typename: 'RoleEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('Role') ? {} as Role : mockRole({}, relationshipsToOmit),
    };
};

export const mockRoleRemovePermissionSetsInput = (overrides?: Partial<RoleRemovePermissionSetsInput>, _relationshipsToOmit: Set<string> = new Set()): RoleRemovePermissionSetsInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('RoleRemovePermissionSetsInput');
    return {
        permissionSetIds: overrides && overrides.hasOwnProperty('permissionSetIds') ? overrides.permissionSetIds! : [generateMockValue.uuid()],
        roleId: overrides && overrides.hasOwnProperty('roleId') ? overrides.roleId! : generateMockValue.uuid(),
    };
};

export const mockRoleRemovePermissionSetsPayload = (overrides?: Partial<RoleRemovePermissionSetsPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'RoleRemovePermissionSetsPayload' } & RoleRemovePermissionSetsPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('RoleRemovePermissionSetsPayload');
    return {
        __typename: 'RoleRemovePermissionSetsPayload',
        role: overrides && overrides.hasOwnProperty('role') ? overrides.role! : relationshipsToOmit.has('Role') ? {} as Role : mockRole({}, relationshipsToOmit),
    };
};

export const mockSumsubPermalinkCreateInput = (overrides?: Partial<SumsubPermalinkCreateInput>, _relationshipsToOmit: Set<string> = new Set()): SumsubPermalinkCreateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('SumsubPermalinkCreateInput');
    return {
        customerId: overrides && overrides.hasOwnProperty('customerId') ? overrides.customerId! : generateMockValue.uuid(),
    };
};

export const mockSumsubPermalinkCreatePayload = (overrides?: Partial<SumsubPermalinkCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'SumsubPermalinkCreatePayload' } & SumsubPermalinkCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('SumsubPermalinkCreatePayload');
    return {
        __typename: 'SumsubPermalinkCreatePayload',
        url: overrides && overrides.hasOwnProperty('url') ? overrides.url! : generateMockValue.url(),
    };
};

export const mockSystem = (overrides?: Partial<System>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'System' } & System => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('System');
    return {
        __typename: 'System',
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
    };
};

export const mockSystemApproval = (overrides?: Partial<SystemApproval>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'SystemApproval' } & SystemApproval => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('SystemApproval');
    return {
        __typename: 'SystemApproval',
        autoApprove: overrides && overrides.hasOwnProperty('autoApprove') ? overrides.autoApprove! : faker.datatype.boolean(),
    };
};

export const mockTermValues = (overrides?: Partial<TermValues>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'TermValues' } & TermValues => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('TermValues');
    return {
        __typename: 'TermValues',
        accrualCycleInterval: overrides && overrides.hasOwnProperty('accrualCycleInterval') ? overrides.accrualCycleInterval! : mockEnums.interestInterval(),
        accrualInterval: overrides && overrides.hasOwnProperty('accrualInterval') ? overrides.accrualInterval! : mockEnums.interestInterval(),
        annualRate: overrides && overrides.hasOwnProperty('annualRate') ? overrides.annualRate! : faker.number.int({ min: 5, max: 20 }),
        disbursalPolicy: overrides && overrides.hasOwnProperty('disbursalPolicy') ? overrides.disbursalPolicy! : DisbursalPolicy.MultipleDisbursal,
        duration: overrides && overrides.hasOwnProperty('duration') ? overrides.duration! : relationshipsToOmit.has('Duration') ? {} as Duration : mockDuration({}, relationshipsToOmit),
        initialCvl: overrides && overrides.hasOwnProperty('initialCvl') ? overrides.initialCvl! : faker.number.int({ min: 95, max: 98 }),
        liquidationCvl: overrides && overrides.hasOwnProperty('liquidationCvl') ? overrides.liquidationCvl! : faker.number.int({ min: 85, max: 88 }),
        marginCallCvl: overrides && overrides.hasOwnProperty('marginCallCvl') ? overrides.marginCallCvl! : faker.number.int({ min: 90, max: 92 }),
        oneTimeFeeRate: overrides && overrides.hasOwnProperty('oneTimeFeeRate') ? overrides.oneTimeFeeRate! : generateMockValue.oneTimeFeeRate(),
    };
};

export const mockTermsInput = (overrides?: Partial<TermsInput>, _relationshipsToOmit: Set<string> = new Set()): TermsInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('TermsInput');
    return {
        accrualCycleInterval: overrides && overrides.hasOwnProperty('accrualCycleInterval') ? overrides.accrualCycleInterval! : InterestInterval.EndOfDay,
        accrualInterval: overrides && overrides.hasOwnProperty('accrualInterval') ? overrides.accrualInterval! : InterestInterval.EndOfDay,
        annualRate: overrides && overrides.hasOwnProperty('annualRate') ? overrides.annualRate! : generateMockValue.int(),
        disbursalPolicy: overrides && overrides.hasOwnProperty('disbursalPolicy') ? overrides.disbursalPolicy! : DisbursalPolicy.MultipleDisbursal,
        duration: overrides && overrides.hasOwnProperty('duration') ? overrides.duration! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        initialCvl: overrides && overrides.hasOwnProperty('initialCvl') ? overrides.initialCvl! : faker.lorem.word(),
        interestDueDurationFromAccrual: overrides && overrides.hasOwnProperty('interestDueDurationFromAccrual') ? overrides.interestDueDurationFromAccrual! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        liquidationCvl: overrides && overrides.hasOwnProperty('liquidationCvl') ? overrides.liquidationCvl! : faker.lorem.word(),
        marginCallCvl: overrides && overrides.hasOwnProperty('marginCallCvl') ? overrides.marginCallCvl! : faker.lorem.word(),
        obligationLiquidationDurationFromDue: overrides && overrides.hasOwnProperty('obligationLiquidationDurationFromDue') ? overrides.obligationLiquidationDurationFromDue! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        obligationOverdueDurationFromDue: overrides && overrides.hasOwnProperty('obligationOverdueDurationFromDue') ? overrides.obligationOverdueDurationFromDue! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        oneTimeFeeRate: overrides && overrides.hasOwnProperty('oneTimeFeeRate') ? overrides.oneTimeFeeRate! : faker.lorem.word(),
    };
};

export const mockTermsTemplate = (overrides?: Partial<TermsTemplate>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'TermsTemplate' } & TermsTemplate => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('TermsTemplate');
    return {
        __typename: 'TermsTemplate',
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        termsId: overrides && overrides.hasOwnProperty('termsId') ? overrides.termsId! : generateMockValue.uuid(),
        userCanUpdateTermsTemplate: overrides && overrides.hasOwnProperty('userCanUpdateTermsTemplate') ? overrides.userCanUpdateTermsTemplate! : faker.datatype.boolean(),
        values: overrides && overrides.hasOwnProperty('values') ? overrides.values! : relationshipsToOmit.has('TermValues') ? {} as TermValues : mockTermValues({}, relationshipsToOmit),
    };
};

export const mockTermsTemplateCreateInput = (overrides?: Partial<TermsTemplateCreateInput>, _relationshipsToOmit: Set<string> = new Set()): TermsTemplateCreateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('TermsTemplateCreateInput');
    return {
        accrualCycleInterval: overrides && overrides.hasOwnProperty('accrualCycleInterval') ? overrides.accrualCycleInterval! : InterestInterval.EndOfDay,
        accrualInterval: overrides && overrides.hasOwnProperty('accrualInterval') ? overrides.accrualInterval! : InterestInterval.EndOfDay,
        annualRate: overrides && overrides.hasOwnProperty('annualRate') ? overrides.annualRate! : generateMockValue.int(),
        disbursalPolicy: overrides && overrides.hasOwnProperty('disbursalPolicy') ? overrides.disbursalPolicy! : DisbursalPolicy.MultipleDisbursal,
        duration: overrides && overrides.hasOwnProperty('duration') ? overrides.duration! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        initialCvl: overrides && overrides.hasOwnProperty('initialCvl') ? overrides.initialCvl! : faker.lorem.word(),
        interestDueDurationFromAccrual: overrides && overrides.hasOwnProperty('interestDueDurationFromAccrual') ? overrides.interestDueDurationFromAccrual! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        liquidationCvl: overrides && overrides.hasOwnProperty('liquidationCvl') ? overrides.liquidationCvl! : faker.lorem.word(),
        marginCallCvl: overrides && overrides.hasOwnProperty('marginCallCvl') ? overrides.marginCallCvl! : faker.lorem.word(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        obligationLiquidationDurationFromDue: overrides && overrides.hasOwnProperty('obligationLiquidationDurationFromDue') ? overrides.obligationLiquidationDurationFromDue! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        obligationOverdueDurationFromDue: overrides && overrides.hasOwnProperty('obligationOverdueDurationFromDue') ? overrides.obligationOverdueDurationFromDue! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        oneTimeFeeRate: overrides && overrides.hasOwnProperty('oneTimeFeeRate') ? overrides.oneTimeFeeRate! : faker.lorem.word(),
    };
};

export const mockTermsTemplateCreatePayload = (overrides?: Partial<TermsTemplateCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'TermsTemplateCreatePayload' } & TermsTemplateCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('TermsTemplateCreatePayload');
    return {
        __typename: 'TermsTemplateCreatePayload',
        termsTemplate: overrides && overrides.hasOwnProperty('termsTemplate') ? overrides.termsTemplate! : relationshipsToOmit.has('TermsTemplate') ? {} as TermsTemplate : mockTermsTemplate({}, relationshipsToOmit),
    };
};

export const mockTermsTemplateUpdateInput = (overrides?: Partial<TermsTemplateUpdateInput>, _relationshipsToOmit: Set<string> = new Set()): TermsTemplateUpdateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('TermsTemplateUpdateInput');
    return {
        accrualCycleInterval: overrides && overrides.hasOwnProperty('accrualCycleInterval') ? overrides.accrualCycleInterval! : InterestInterval.EndOfDay,
        accrualInterval: overrides && overrides.hasOwnProperty('accrualInterval') ? overrides.accrualInterval! : InterestInterval.EndOfDay,
        annualRate: overrides && overrides.hasOwnProperty('annualRate') ? overrides.annualRate! : generateMockValue.int(),
        disbursalPolicy: overrides && overrides.hasOwnProperty('disbursalPolicy') ? overrides.disbursalPolicy! : DisbursalPolicy.MultipleDisbursal,
        duration: overrides && overrides.hasOwnProperty('duration') ? overrides.duration! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : generateMockValue.uuid(),
        initialCvl: overrides && overrides.hasOwnProperty('initialCvl') ? overrides.initialCvl! : faker.lorem.word(),
        interestDueDurationFromAccrual: overrides && overrides.hasOwnProperty('interestDueDurationFromAccrual') ? overrides.interestDueDurationFromAccrual! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        liquidationCvl: overrides && overrides.hasOwnProperty('liquidationCvl') ? overrides.liquidationCvl! : faker.lorem.word(),
        marginCallCvl: overrides && overrides.hasOwnProperty('marginCallCvl') ? overrides.marginCallCvl! : faker.lorem.word(),
        obligationLiquidationDurationFromDue: overrides && overrides.hasOwnProperty('obligationLiquidationDurationFromDue') ? overrides.obligationLiquidationDurationFromDue! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        obligationOverdueDurationFromDue: overrides && overrides.hasOwnProperty('obligationOverdueDurationFromDue') ? overrides.obligationOverdueDurationFromDue! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        oneTimeFeeRate: overrides && overrides.hasOwnProperty('oneTimeFeeRate') ? overrides.oneTimeFeeRate! : faker.lorem.word(),
    };
};

export const mockTermsTemplateUpdatePayload = (overrides?: Partial<TermsTemplateUpdatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'TermsTemplateUpdatePayload' } & TermsTemplateUpdatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('TermsTemplateUpdatePayload');
    return {
        __typename: 'TermsTemplateUpdatePayload',
        termsTemplate: overrides && overrides.hasOwnProperty('termsTemplate') ? overrides.termsTemplate! : relationshipsToOmit.has('TermsTemplate') ? {} as TermsTemplate : mockTermsTemplate({}, relationshipsToOmit),
    };
};

export const mockTotal = (overrides?: Partial<Total>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Total' } & Total => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Total');
    return {
        __typename: 'Total',
        usdBalance: overrides && overrides.hasOwnProperty('usdBalance') ? overrides.usdBalance! : generateMockValue.usdCents(),
    };
};

export const mockTransactionTemplate = (overrides?: Partial<TransactionTemplate>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'TransactionTemplate' } & TransactionTemplate => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('TransactionTemplate');
    return {
        __typename: 'TransactionTemplate',
        code: overrides && overrides.hasOwnProperty('code') ? overrides.code! : faker.lorem.word(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : generateMockValue.uuid(),
    };
};

export const mockTransactionTemplateConnection = (overrides?: Partial<TransactionTemplateConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'TransactionTemplateConnection' } & TransactionTemplateConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('TransactionTemplateConnection');
    return {
        __typename: 'TransactionTemplateConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('TransactionTemplateEdge') ? {} as TransactionTemplateEdge : mockTransactionTemplateEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('TransactionTemplate') ? {} as TransactionTemplate : mockTransactionTemplate({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockTransactionTemplateEdge = (overrides?: Partial<TransactionTemplateEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'TransactionTemplateEdge' } & TransactionTemplateEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('TransactionTemplateEdge');
    return {
        __typename: 'TransactionTemplateEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('TransactionTemplate') ? {} as TransactionTemplate : mockTransactionTemplate({}, relationshipsToOmit),
    };
};

export const mockTrialBalance = (overrides?: Partial<TrialBalance>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'TrialBalance' } & TrialBalance => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('TrialBalance');
    return {
        __typename: 'TrialBalance',
        accounts: overrides && overrides.hasOwnProperty('accounts') ? overrides.accounts! : [relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit)],
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        total: overrides && overrides.hasOwnProperty('total') ? overrides.total! : relationshipsToOmit.has('LedgerAccountBalanceRangeByCurrency') ? {} as LedgerAccountBalanceRangeByCurrency : mockLedgerAccountBalanceRangeByCurrency({}, relationshipsToOmit),
    };
};

export const mockUnfreezeEntry = (overrides?: Partial<UnfreezeEntry>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'UnfreezeEntry' } & UnfreezeEntry => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UnfreezeEntry');
    return {
        __typename: 'UnfreezeEntry',
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.usdCents(),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        txId: overrides && overrides.hasOwnProperty('txId') ? overrides.txId! : generateMockValue.uuid(),
    };
};

export const mockUnknownEntry = (overrides?: Partial<UnknownEntry>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'UnknownEntry' } & UnknownEntry => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UnknownEntry');
    return {
        __typename: 'UnknownEntry',
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        txId: overrides && overrides.hasOwnProperty('txId') ? overrides.txId! : generateMockValue.uuid(),
    };
};

export const mockUsdAmount = (overrides?: Partial<UsdAmount>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'UsdAmount' } & UsdAmount => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UsdAmount');
    return {
        __typename: 'UsdAmount',
        usd: overrides && overrides.hasOwnProperty('usd') ? overrides.usd! : generateMockValue.usdCents(),
    };
};

export const mockUsdBalanceDetails = (overrides?: Partial<UsdBalanceDetails>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'UsdBalanceDetails' } & UsdBalanceDetails => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UsdBalanceDetails');
    return {
        __typename: 'UsdBalanceDetails',
        credit: overrides && overrides.hasOwnProperty('credit') ? overrides.credit! : generateMockValue.usdCents(),
        debit: overrides && overrides.hasOwnProperty('debit') ? overrides.debit! : generateMockValue.usdCents(),
        net: overrides && overrides.hasOwnProperty('net') ? overrides.net! : generateMockValue.signedUsdCents(),
    };
};

export const mockUsdLedgerAccountBalance = (overrides?: Partial<UsdLedgerAccountBalance>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'UsdLedgerAccountBalance' } & UsdLedgerAccountBalance => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UsdLedgerAccountBalance');
    return {
        __typename: 'UsdLedgerAccountBalance',
        encumbrance: overrides && overrides.hasOwnProperty('encumbrance') ? overrides.encumbrance! : relationshipsToOmit.has('UsdBalanceDetails') ? {} as UsdBalanceDetails : mockUsdBalanceDetails({}, relationshipsToOmit),
        pending: overrides && overrides.hasOwnProperty('pending') ? overrides.pending! : relationshipsToOmit.has('UsdBalanceDetails') ? {} as UsdBalanceDetails : mockUsdBalanceDetails({}, relationshipsToOmit),
        settled: overrides && overrides.hasOwnProperty('settled') ? overrides.settled! : relationshipsToOmit.has('UsdBalanceDetails') ? {} as UsdBalanceDetails : mockUsdBalanceDetails({}, relationshipsToOmit),
    };
};

export const mockUsdLedgerAccountBalanceRange = (overrides?: Partial<UsdLedgerAccountBalanceRange>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'UsdLedgerAccountBalanceRange' } & UsdLedgerAccountBalanceRange => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UsdLedgerAccountBalanceRange');
    return {
        __typename: 'UsdLedgerAccountBalanceRange',
        close: overrides && overrides.hasOwnProperty('close') ? overrides.close! : relationshipsToOmit.has('UsdLedgerAccountBalance') ? {} as UsdLedgerAccountBalance : mockUsdLedgerAccountBalance({}, relationshipsToOmit),
        open: overrides && overrides.hasOwnProperty('open') ? overrides.open! : relationshipsToOmit.has('UsdLedgerAccountBalance') ? {} as UsdLedgerAccountBalance : mockUsdLedgerAccountBalance({}, relationshipsToOmit),
        periodActivity: overrides && overrides.hasOwnProperty('periodActivity') ? overrides.periodActivity! : relationshipsToOmit.has('UsdLedgerAccountBalance') ? {} as UsdLedgerAccountBalance : mockUsdLedgerAccountBalance({}, relationshipsToOmit),
    };
};

export const mockUser = (overrides?: Partial<User>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'User' } & User => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('User');
    return {
        __typename: 'User',
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        email: overrides && overrides.hasOwnProperty('email') ? overrides.email! : generateMockValue.email(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        role: overrides && overrides.hasOwnProperty('role') ? overrides.role! : relationshipsToOmit.has('Role') ? {} as Role : mockRole({}, relationshipsToOmit),
        userCanUpdateRoleOfUser: overrides && overrides.hasOwnProperty('userCanUpdateRoleOfUser') ? overrides.userCanUpdateRoleOfUser! : faker.datatype.boolean(),
        userId: overrides && overrides.hasOwnProperty('userId') ? overrides.userId! : generateMockValue.uuid(),
    };
};

export const mockUserCreateInput = (overrides?: Partial<UserCreateInput>, _relationshipsToOmit: Set<string> = new Set()): UserCreateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UserCreateInput');
    return {
        email: overrides && overrides.hasOwnProperty('email') ? overrides.email! : generateMockValue.email(),
        roleId: overrides && overrides.hasOwnProperty('roleId') ? overrides.roleId! : generateMockValue.uuid(),
    };
};

export const mockUserCreatePayload = (overrides?: Partial<UserCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'UserCreatePayload' } & UserCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UserCreatePayload');
    return {
        __typename: 'UserCreatePayload',
        user: overrides && overrides.hasOwnProperty('user') ? overrides.user! : relationshipsToOmit.has('User') ? {} as User : mockUser({}, relationshipsToOmit),
    };
};

export const mockUserUpdateRoleInput = (overrides?: Partial<UserUpdateRoleInput>, _relationshipsToOmit: Set<string> = new Set()): UserUpdateRoleInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UserUpdateRoleInput');
    return {
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : generateMockValue.uuid(),
        roleId: overrides && overrides.hasOwnProperty('roleId') ? overrides.roleId! : generateMockValue.uuid(),
    };
};

export const mockUserUpdateRolePayload = (overrides?: Partial<UserUpdateRolePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'UserUpdateRolePayload' } & UserUpdateRolePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UserUpdateRolePayload');
    return {
        __typename: 'UserUpdateRolePayload',
        user: overrides && overrides.hasOwnProperty('user') ? overrides.user! : relationshipsToOmit.has('User') ? {} as User : mockUser({}, relationshipsToOmit),
    };
};

export const mockVisibleNavigationItems = (overrides?: Partial<VisibleNavigationItems>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'VisibleNavigationItems' } & VisibleNavigationItems => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('VisibleNavigationItems');
    return {
        __typename: 'VisibleNavigationItems',
        audit: overrides && overrides.hasOwnProperty('audit') ? overrides.audit! : faker.datatype.boolean(),
        creditFacilities: overrides && overrides.hasOwnProperty('creditFacilities') ? overrides.creditFacilities! : faker.datatype.boolean(),
        customer: overrides && overrides.hasOwnProperty('customer') ? overrides.customer! : faker.datatype.boolean(),
        deposit: overrides && overrides.hasOwnProperty('deposit') ? overrides.deposit! : faker.datatype.boolean(),
        financials: overrides && overrides.hasOwnProperty('financials') ? overrides.financials! : faker.datatype.boolean(),
        governance: overrides && overrides.hasOwnProperty('governance') ? overrides.governance! : relationshipsToOmit.has('GovernanceNavigationItems') ? {} as GovernanceNavigationItems : mockGovernanceNavigationItems({}, relationshipsToOmit),
        term: overrides && overrides.hasOwnProperty('term') ? overrides.term! : faker.datatype.boolean(),
        user: overrides && overrides.hasOwnProperty('user') ? overrides.user! : faker.datatype.boolean(),
        withdraw: overrides && overrides.hasOwnProperty('withdraw') ? overrides.withdraw! : faker.datatype.boolean(),
    };
};

export const mockWallet = (overrides?: Partial<Wallet>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Wallet' } & Wallet => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Wallet');
    return {
        __typename: 'Wallet',
        address: overrides && overrides.hasOwnProperty('address') ? overrides.address! : faker.lorem.word(),
        custodian: overrides && overrides.hasOwnProperty('custodian') ? overrides.custodian! : relationshipsToOmit.has('Custodian') ? {} as Custodian : mockCustodian({}, relationshipsToOmit),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        network: overrides && overrides.hasOwnProperty('network') ? overrides.network! : WalletNetwork.Mainnet,
        walletId: overrides && overrides.hasOwnProperty('walletId') ? overrides.walletId! : generateMockValue.uuid(),
    };
};

export const mockWithdrawal = (overrides?: Partial<Withdrawal>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Withdrawal' } & Withdrawal => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Withdrawal');
    return {
        __typename: 'Withdrawal',
        account: overrides && overrides.hasOwnProperty('account') ? overrides.account! : relationshipsToOmit.has('DepositAccount') ? {} as DepositAccount : mockDepositAccount({}, relationshipsToOmit),
        accountId: overrides && overrides.hasOwnProperty('accountId') ? overrides.accountId! : generateMockValue.uuid(),
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.usdCents(),
        approvalProcess: overrides && overrides.hasOwnProperty('approvalProcess') ? overrides.approvalProcess! : relationshipsToOmit.has('ApprovalProcess') ? {} as ApprovalProcess : mockApprovalProcess({}, relationshipsToOmit),
        approvalProcessId: overrides && overrides.hasOwnProperty('approvalProcessId') ? overrides.approvalProcessId! : generateMockValue.uuid(),
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        ledgerTransactions: overrides && overrides.hasOwnProperty('ledgerTransactions') ? overrides.ledgerTransactions! : [relationshipsToOmit.has('LedgerTransaction') ? {} as LedgerTransaction : mockLedgerTransaction({}, relationshipsToOmit)],
        publicId: overrides && overrides.hasOwnProperty('publicId') ? overrides.publicId! : faker.lorem.word(),
        reference: overrides && overrides.hasOwnProperty('reference') ? overrides.reference! : generateMockValue.reference(),
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : mockEnums.withdrawalStatus(),
        withdrawalId: overrides && overrides.hasOwnProperty('withdrawalId') ? overrides.withdrawalId! : generateMockValue.uuid(),
    };
};

export const mockWithdrawalCancelInput = (overrides?: Partial<WithdrawalCancelInput>, _relationshipsToOmit: Set<string> = new Set()): WithdrawalCancelInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('WithdrawalCancelInput');
    return {
        withdrawalId: overrides && overrides.hasOwnProperty('withdrawalId') ? overrides.withdrawalId! : generateMockValue.uuid(),
    };
};

export const mockWithdrawalCancelPayload = (overrides?: Partial<WithdrawalCancelPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'WithdrawalCancelPayload' } & WithdrawalCancelPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('WithdrawalCancelPayload');
    return {
        __typename: 'WithdrawalCancelPayload',
        withdrawal: overrides && overrides.hasOwnProperty('withdrawal') ? overrides.withdrawal! : relationshipsToOmit.has('Withdrawal') ? {} as Withdrawal : mockWithdrawal({}, relationshipsToOmit),
    };
};

export const mockWithdrawalConfirmInput = (overrides?: Partial<WithdrawalConfirmInput>, _relationshipsToOmit: Set<string> = new Set()): WithdrawalConfirmInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('WithdrawalConfirmInput');
    return {
        withdrawalId: overrides && overrides.hasOwnProperty('withdrawalId') ? overrides.withdrawalId! : generateMockValue.uuid(),
    };
};

export const mockWithdrawalConfirmPayload = (overrides?: Partial<WithdrawalConfirmPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'WithdrawalConfirmPayload' } & WithdrawalConfirmPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('WithdrawalConfirmPayload');
    return {
        __typename: 'WithdrawalConfirmPayload',
        withdrawal: overrides && overrides.hasOwnProperty('withdrawal') ? overrides.withdrawal! : relationshipsToOmit.has('Withdrawal') ? {} as Withdrawal : mockWithdrawal({}, relationshipsToOmit),
    };
};

export const mockWithdrawalConnection = (overrides?: Partial<WithdrawalConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'WithdrawalConnection' } & WithdrawalConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('WithdrawalConnection');
    return {
        __typename: 'WithdrawalConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('WithdrawalEdge') ? {} as WithdrawalEdge : mockWithdrawalEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('Withdrawal') ? {} as Withdrawal : mockWithdrawal({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockWithdrawalEdge = (overrides?: Partial<WithdrawalEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'WithdrawalEdge' } & WithdrawalEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('WithdrawalEdge');
    return {
        __typename: 'WithdrawalEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('Withdrawal') ? {} as Withdrawal : mockWithdrawal({}, relationshipsToOmit),
    };
};

export const mockWithdrawalEntry = (overrides?: Partial<WithdrawalEntry>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'WithdrawalEntry' } & WithdrawalEntry => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('WithdrawalEntry');
    return {
        __typename: 'WithdrawalEntry',
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        withdrawal: overrides && overrides.hasOwnProperty('withdrawal') ? overrides.withdrawal! : relationshipsToOmit.has('Withdrawal') ? {} as Withdrawal : mockWithdrawal({}, relationshipsToOmit),
    };
};

export const mockWithdrawalInitiateInput = (overrides?: Partial<WithdrawalInitiateInput>, _relationshipsToOmit: Set<string> = new Set()): WithdrawalInitiateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('WithdrawalInitiateInput');
    return {
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.usdCents(),
        depositAccountId: overrides && overrides.hasOwnProperty('depositAccountId') ? overrides.depositAccountId! : generateMockValue.uuid(),
        reference: overrides && overrides.hasOwnProperty('reference') ? overrides.reference! : generateMockValue.reference(),
    };
};

export const mockWithdrawalInitiatePayload = (overrides?: Partial<WithdrawalInitiatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'WithdrawalInitiatePayload' } & WithdrawalInitiatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('WithdrawalInitiatePayload');
    return {
        __typename: 'WithdrawalInitiatePayload',
        withdrawal: overrides && overrides.hasOwnProperty('withdrawal') ? overrides.withdrawal! : relationshipsToOmit.has('Withdrawal') ? {} as Withdrawal : mockWithdrawal({}, relationshipsToOmit),
    };
};

export const mockWithdrawalRevertInput = (overrides?: Partial<WithdrawalRevertInput>, _relationshipsToOmit: Set<string> = new Set()): WithdrawalRevertInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('WithdrawalRevertInput');
    return {
        withdrawalId: overrides && overrides.hasOwnProperty('withdrawalId') ? overrides.withdrawalId! : generateMockValue.uuid(),
    };
};

export const mockWithdrawalRevertPayload = (overrides?: Partial<WithdrawalRevertPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'WithdrawalRevertPayload' } & WithdrawalRevertPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('WithdrawalRevertPayload');
    return {
        __typename: 'WithdrawalRevertPayload',
        withdrawal: overrides && overrides.hasOwnProperty('withdrawal') ? overrides.withdrawal! : relationshipsToOmit.has('Withdrawal') ? {} as Withdrawal : mockWithdrawal({}, relationshipsToOmit),
    };
};

export const seedMocks = (seed: number) => faker.seed(seed);
