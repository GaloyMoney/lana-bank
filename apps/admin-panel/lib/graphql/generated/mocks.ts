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
import { AccountingCsv, AccountingCsvConnection, AccountingCsvDownloadLink, AccountingCsvDownloadLinkGenerateInput, AccountingCsvDownloadLinkGeneratePayload, AccountingCsvEdge, ApprovalProcess, ApprovalProcessApproveInput, ApprovalProcessApprovePayload, ApprovalProcessConnection, ApprovalProcessDenyInput, ApprovalProcessDenyPayload, ApprovalProcessEdge, ApprovalProcessVoter, AuditEntry, AuditEntryConnection, AuditEntryEdge, BalanceSheet, BalanceSheetModuleConfig, BalanceSheetModuleConfigureInput, BalanceSheetModuleConfigurePayload, BtcAmount, BtcBalanceDetails, BtcLedgerAccountBalance, BtcLedgerAccountBalanceRange, CancelledWithdrawalEntry, ChartNode, ChartOfAccounts, ChartOfAccountsCsvImportInput, ChartOfAccountsCsvImportPayload, Collateral, Committee, CommitteeAddUserInput, CommitteeAddUserPayload, CommitteeConnection, CommitteeCreateInput, CommitteeCreatePayload, CommitteeEdge, CommitteeRemoveUserInput, CommitteeRemoveUserPayload, CommitteeThreshold, CreditFacilitiesFilter, CreditFacilitiesSort, CreditFacility, CreditFacilityBalance, CreditFacilityCollateralUpdateInput, CreditFacilityCollateralUpdatePayload, CreditFacilityCollateralUpdated, CreditFacilityCollateralizationUpdated, CreditFacilityCompleteInput, CreditFacilityCompletePayload, CreditFacilityConnection, CreditFacilityCreateInput, CreditFacilityCreatePayload, CreditFacilityDisbursal, CreditFacilityDisbursalConnection, CreditFacilityDisbursalEdge, CreditFacilityDisbursalExecuted, CreditFacilityDisbursalInitiateInput, CreditFacilityDisbursalInitiatePayload, CreditFacilityEdge, CreditFacilityIncrementalPayment, CreditFacilityInterestAccrued, CreditFacilityOrigination, CreditFacilityPartialPaymentInput, CreditFacilityPartialPaymentPayload, CreditFacilityPaymentAllocation, CreditFacilityRepaymentPlanEntry, CreditModuleConfig, CreditModuleConfigureInput, CreditModuleConfigurePayload, Customer, CustomerConnection, CustomerCreateInput, CustomerCreatePayload, CustomerEdge, CustomerEmailUpdateInput, CustomerEmailUpdatePayload, CustomerTelegramIdUpdateInput, CustomerTelegramIdUpdatePayload, CustomersFilter, CustomersSort, Dashboard, Deposit, DepositAccount, DepositAccountBalance, DepositAccountHistoryEntryConnection, DepositAccountHistoryEntryEdge, DepositConnection, DepositEdge, DepositEntry, DepositModuleConfig, DepositModuleConfigureInput, DepositModuleConfigurePayload, DepositRecordInput, DepositRecordPayload, DisbursalEntry, Disbursed, Document, DocumentArchiveInput, DocumentArchivePayload, DocumentCreateInput, DocumentCreatePayload, DocumentDeleteInput, DocumentDeletePayload, DocumentDownloadLinksGenerateInput, DocumentDownloadLinksGeneratePayload, Duration, DurationInput, FacilityCvl, FacilityRemaining, GovernanceNavigationItems, Interest, JournalEntry, JournalEntryConnection, JournalEntryEdge, LedgerAccount, LedgerAccountBalanceRangeByCurrency, LedgerAccountConnection, LedgerAccountCsvCreateInput, LedgerAccountCsvCreatePayload, LedgerAccountEdge, LedgerTransaction, LedgerTransactionConnection, LedgerTransactionEdge, Loan, ManualTransactionEntryInput, ManualTransactionExecuteInput, ManualTransactionExecutePayload, Mutation, Outstanding, PageInfo, PaymentEntry, PermissionSet, PermissionSetConnection, PermissionSetEdge, Policy, PolicyAssignCommitteeInput, PolicyAssignCommitteePayload, PolicyConnection, PolicyEdge, ProfitAndLossModuleConfigureInput, ProfitAndLossStatement, ProfitAndLossStatementModuleConfig, ProfitAndLossStatementModuleConfigurePayload, Query, RealtimePrice, Report, ReportCreatePayload, ReportDownloadLink, ReportDownloadLinksGenerateInput, ReportDownloadLinksGeneratePayload, RoleAddPermissionSetsInput, RoleAddPermissionSetsPayload, RoleCreateInput, RoleCreatePayload, RoleEntity, RoleEntityConnection, RoleEntityEdge, RoleRemovePermissionSetsInput, RoleRemovePermissionSetsPayload, Subject, SumsubPermalinkCreateInput, SumsubPermalinkCreatePayload, System, SystemApproval, TermValues, TermsInput, TermsTemplate, TermsTemplateCreateInput, TermsTemplateCreatePayload, TermsTemplateUpdateInput, TermsTemplateUpdatePayload, Total, TransactionTemplate, TransactionTemplateConnection, TransactionTemplateEdge, TrialBalance, UnknownEntry, UsdAmount, UsdBalanceDetails, UsdLedgerAccountBalance, UsdLedgerAccountBalanceRange, User, UserAssignRoleInput, UserAssignRolePayload, UserCreateInput, UserCreatePayload, UserRevokeRoleInput, UserRevokeRolePayload, VisibleNavigationItems, Withdrawal, WithdrawalCancelInput, WithdrawalCancelPayload, WithdrawalConfirmInput, WithdrawalConfirmPayload, WithdrawalConnection, WithdrawalEdge, WithdrawalEntry, WithdrawalInitiateInput, WithdrawalInitiatePayload, AccountStatus, AccountingCsvStatus, ApprovalProcessStatus, ApprovalProcessType, CollateralAction, CollateralizationState, CreditFacilitiesFilterBy, CreditFacilitiesSortBy, CreditFacilityRepaymentStatus, CreditFacilityRepaymentType, CreditFacilityStatus, CustomerType, CustomersFilterBy, CustomersSortBy, DebitOrCredit, DisbursalStatus, DocumentStatus, InterestInterval, KycLevel, Layer, Period, ReportProgress, SortDirection, WithdrawalStatus } from './index';

faker.seed(0);

export const mockAccountingCsv = (overrides?: Partial<AccountingCsv>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'AccountingCsv' } & AccountingCsv => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('AccountingCsv');
    return {
        __typename: 'AccountingCsv',
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        csvId: overrides && overrides.hasOwnProperty('csvId') ? overrides.csvId! : generateMockValue.uuid(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : AccountingCsvStatus.Completed,
    };
};

export const mockAccountingCsvConnection = (overrides?: Partial<AccountingCsvConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'AccountingCsvConnection' } & AccountingCsvConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('AccountingCsvConnection');
    return {
        __typename: 'AccountingCsvConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('AccountingCsvEdge') ? {} as AccountingCsvEdge : mockAccountingCsvEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('AccountingCsv') ? {} as AccountingCsv : mockAccountingCsv({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
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
        accountingCsvId: overrides && overrides.hasOwnProperty('accountingCsvId') ? overrides.accountingCsvId! : generateMockValue.uuid(),
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

export const mockAccountingCsvEdge = (overrides?: Partial<AccountingCsvEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'AccountingCsvEdge' } & AccountingCsvEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('AccountingCsvEdge');
    return {
        __typename: 'AccountingCsvEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('AccountingCsv') ? {} as AccountingCsv : mockAccountingCsv({}, relationshipsToOmit),
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
        subjectCanSubmitDecision: overrides && overrides.hasOwnProperty('subjectCanSubmitDecision') ? overrides.subjectCanSubmitDecision! : faker.datatype.boolean(),
        target: overrides && overrides.hasOwnProperty('target') ? overrides.target! : relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit),
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

export const mockChartOfAccountsCsvImportInput = (overrides?: Partial<ChartOfAccountsCsvImportInput>, _relationshipsToOmit: Set<string> = new Set()): ChartOfAccountsCsvImportInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ChartOfAccountsCsvImportInput');
    return {
        chartId: overrides && overrides.hasOwnProperty('chartId') ? overrides.chartId! : generateMockValue.uuid(),
        file: overrides && overrides.hasOwnProperty('file') ? overrides.file! : faker.lorem.word(),
    };
};

export const mockChartOfAccountsCsvImportPayload = (overrides?: Partial<ChartOfAccountsCsvImportPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ChartOfAccountsCsvImportPayload' } & ChartOfAccountsCsvImportPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ChartOfAccountsCsvImportPayload');
    return {
        __typename: 'ChartOfAccountsCsvImportPayload',
        success: overrides && overrides.hasOwnProperty('success') ? overrides.success! : generateMockValue.boolean(),
    };
};

export const mockCollateral = (overrides?: Partial<Collateral>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Collateral' } & Collateral => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Collateral');
    return {
        __typename: 'Collateral',
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
        approvalProcess: overrides && overrides.hasOwnProperty('approvalProcess') ? overrides.approvalProcess! : relationshipsToOmit.has('ApprovalProcess') ? {} as ApprovalProcess : mockApprovalProcess({}, relationshipsToOmit),
        approvalProcessId: overrides && overrides.hasOwnProperty('approvalProcessId') ? overrides.approvalProcessId! : generateMockValue.uuid(),
        balance: overrides && overrides.hasOwnProperty('balance') ? overrides.balance! : relationshipsToOmit.has('CreditFacilityBalance') ? {} as CreditFacilityBalance : mockCreditFacilityBalance({}, relationshipsToOmit),
        canBeCompleted: overrides && overrides.hasOwnProperty('canBeCompleted') ? overrides.canBeCompleted! : generateMockValue.boolean(),
        collateralToMatchInitialCvl: overrides && overrides.hasOwnProperty('collateralToMatchInitialCvl') ? overrides.collateralToMatchInitialCvl! : generateMockValue.satoshis(),
        collateralizationState: overrides && overrides.hasOwnProperty('collateralizationState') ? overrides.collateralizationState! : mockEnums.collateralizationState(),
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        creditFacilityId: overrides && overrides.hasOwnProperty('creditFacilityId') ? overrides.creditFacilityId! : generateMockValue.uuid(),
        creditFacilityTerms: overrides && overrides.hasOwnProperty('creditFacilityTerms') ? overrides.creditFacilityTerms! : relationshipsToOmit.has('TermValues') ? {} as TermValues : mockTermValues({}, relationshipsToOmit),
        currentCvl: overrides && overrides.hasOwnProperty('currentCvl') ? overrides.currentCvl! : relationshipsToOmit.has('FacilityCvl') ? {} as FacilityCvl : mockFacilityCvl({}, relationshipsToOmit),
        customer: overrides && overrides.hasOwnProperty('customer') ? overrides.customer! : relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit),
        disbursals: overrides && overrides.hasOwnProperty('disbursals') ? overrides.disbursals! : [relationshipsToOmit.has('CreditFacilityDisbursal') ? {} as CreditFacilityDisbursal : mockCreditFacilityDisbursal({}, relationshipsToOmit)],
        facilityAmount: overrides && overrides.hasOwnProperty('facilityAmount') ? overrides.facilityAmount! : generateMockValue.usdCents(),
        history: overrides && overrides.hasOwnProperty('history') ? overrides.history! : [relationshipsToOmit.has('CreditFacilityCollateralUpdated') ? {} as CreditFacilityCollateralUpdated : mockCreditFacilityCollateralUpdated({}, relationshipsToOmit)],
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        maturesAt: overrides && overrides.hasOwnProperty('maturesAt') ? overrides.maturesAt! : generateMockValue.timestamp(),
        repaymentPlan: overrides && overrides.hasOwnProperty('repaymentPlan') ? overrides.repaymentPlan! : [relationshipsToOmit.has('CreditFacilityRepaymentPlanEntry') ? {} as CreditFacilityRepaymentPlanEntry : mockCreditFacilityRepaymentPlanEntry({}, relationshipsToOmit)],
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : mockEnums.creditFacilityStatus(),
        subjectCanComplete: overrides && overrides.hasOwnProperty('subjectCanComplete') ? overrides.subjectCanComplete! : faker.datatype.boolean(),
        subjectCanInitiateDisbursal: overrides && overrides.hasOwnProperty('subjectCanInitiateDisbursal') ? overrides.subjectCanInitiateDisbursal! : faker.datatype.boolean(),
        subjectCanRecordPayment: overrides && overrides.hasOwnProperty('subjectCanRecordPayment') ? overrides.subjectCanRecordPayment! : faker.datatype.boolean(),
        subjectCanUpdateCollateral: overrides && overrides.hasOwnProperty('subjectCanUpdateCollateral') ? overrides.subjectCanUpdateCollateral! : faker.datatype.boolean(),
    };
};

export const mockCreditFacilityBalance = (overrides?: Partial<CreditFacilityBalance>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityBalance' } & CreditFacilityBalance => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityBalance');
    return {
        __typename: 'CreditFacilityBalance',
        collateral: overrides && overrides.hasOwnProperty('collateral') ? overrides.collateral! : relationshipsToOmit.has('Collateral') ? {} as Collateral : mockCollateral({}, relationshipsToOmit),
        disbursed: overrides && overrides.hasOwnProperty('disbursed') ? overrides.disbursed! : relationshipsToOmit.has('Disbursed') ? {} as Disbursed : mockDisbursed({}, relationshipsToOmit),
        dueOutstanding: overrides && overrides.hasOwnProperty('dueOutstanding') ? overrides.dueOutstanding! : relationshipsToOmit.has('Outstanding') ? {} as Outstanding : mockOutstanding({}, relationshipsToOmit),
        facilityRemaining: overrides && overrides.hasOwnProperty('facilityRemaining') ? overrides.facilityRemaining! : relationshipsToOmit.has('FacilityRemaining') ? {} as FacilityRemaining : mockFacilityRemaining({}, relationshipsToOmit),
        interest: overrides && overrides.hasOwnProperty('interest') ? overrides.interest! : relationshipsToOmit.has('Interest') ? {} as Interest : mockInterest({}, relationshipsToOmit),
        outstanding: overrides && overrides.hasOwnProperty('outstanding') ? overrides.outstanding! : relationshipsToOmit.has('Outstanding') ? {} as Outstanding : mockOutstanding({}, relationshipsToOmit),
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

export const mockCreditFacilityCreateInput = (overrides?: Partial<CreditFacilityCreateInput>, _relationshipsToOmit: Set<string> = new Set()): CreditFacilityCreateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityCreateInput');
    return {
        customerId: overrides && overrides.hasOwnProperty('customerId') ? overrides.customerId! : generateMockValue.uuid(),
        disbursalCreditAccountId: overrides && overrides.hasOwnProperty('disbursalCreditAccountId') ? overrides.disbursalCreditAccountId! : generateMockValue.uuid(),
        facility: overrides && overrides.hasOwnProperty('facility') ? overrides.facility! : generateMockValue.usdCents(),
        terms: overrides && overrides.hasOwnProperty('terms') ? overrides.terms! : relationshipsToOmit.has('TermsInput') ? {} as TermsInput : mockTermsInput({}, relationshipsToOmit),
    };
};

export const mockCreditFacilityCreatePayload = (overrides?: Partial<CreditFacilityCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityCreatePayload' } & CreditFacilityCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityCreatePayload');
    return {
        __typename: 'CreditFacilityCreatePayload',
        creditFacility: overrides && overrides.hasOwnProperty('creditFacility') ? overrides.creditFacility! : relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit),
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
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        txId: overrides && overrides.hasOwnProperty('txId') ? overrides.txId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityOrigination = (overrides?: Partial<CreditFacilityOrigination>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityOrigination' } & CreditFacilityOrigination => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityOrigination');
    return {
        __typename: 'CreditFacilityOrigination',
        cents: overrides && overrides.hasOwnProperty('cents') ? overrides.cents! : generateMockValue.usdCents(),
        recordedAt: overrides && overrides.hasOwnProperty('recordedAt') ? overrides.recordedAt! : generateMockValue.timestamp(),
        txId: overrides && overrides.hasOwnProperty('txId') ? overrides.txId! : generateMockValue.uuid(),
    };
};

export const mockCreditFacilityPartialPaymentInput = (overrides?: Partial<CreditFacilityPartialPaymentInput>, _relationshipsToOmit: Set<string> = new Set()): CreditFacilityPartialPaymentInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityPartialPaymentInput');
    return {
        amount: overrides && overrides.hasOwnProperty('amount') ? overrides.amount! : generateMockValue.usdCents(),
        creditFacilityId: overrides && overrides.hasOwnProperty('creditFacilityId') ? overrides.creditFacilityId! : generateMockValue.uuid(),
        effective: overrides && overrides.hasOwnProperty('effective') ? overrides.effective! : faker.date.past({ years: 1, refDate: new Date(2022, 0) }).toISOString(),
    };
};

export const mockCreditFacilityPartialPaymentPayload = (overrides?: Partial<CreditFacilityPartialPaymentPayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'CreditFacilityPartialPaymentPayload' } & CreditFacilityPartialPaymentPayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('CreditFacilityPartialPaymentPayload');
    return {
        __typename: 'CreditFacilityPartialPaymentPayload',
        creditFacility: overrides && overrides.hasOwnProperty('creditFacility') ? overrides.creditFacility! : relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit),
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

export const mockCustomer = (overrides?: Partial<Customer>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Customer' } & Customer => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Customer');
    return {
        __typename: 'Customer',
        applicantId: overrides && overrides.hasOwnProperty('applicantId') ? overrides.applicantId! : generateMockValue.applicantId(),
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        creditFacilities: overrides && overrides.hasOwnProperty('creditFacilities') ? overrides.creditFacilities! : [relationshipsToOmit.has('CreditFacility') ? {} as CreditFacility : mockCreditFacility({}, relationshipsToOmit)],
        customerId: overrides && overrides.hasOwnProperty('customerId') ? overrides.customerId! : generateMockValue.uuid(),
        customerType: overrides && overrides.hasOwnProperty('customerType') ? overrides.customerType! : CustomerType.Bank,
        depositAccount: overrides && overrides.hasOwnProperty('depositAccount') ? overrides.depositAccount! : relationshipsToOmit.has('DepositAccount') ? {} as DepositAccount : mockDepositAccount({}, relationshipsToOmit),
        documents: overrides && overrides.hasOwnProperty('documents') ? overrides.documents! : [relationshipsToOmit.has('Document') ? {} as Document : mockDocument({}, relationshipsToOmit)],
        email: overrides && overrides.hasOwnProperty('email') ? overrides.email! : generateMockValue.email(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        level: overrides && overrides.hasOwnProperty('level') ? overrides.level! : mockEnums.kycLevel(),
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : mockEnums.accountStatus(),
        subjectCanCreateCreditFacility: overrides && overrides.hasOwnProperty('subjectCanCreateCreditFacility') ? overrides.subjectCanCreateCreditFacility! : faker.datatype.boolean(),
        telegramId: overrides && overrides.hasOwnProperty('telegramId') ? overrides.telegramId! : generateMockValue.telegramId(),
        transactions: overrides && overrides.hasOwnProperty('transactions') ? overrides.transactions! : [relationshipsToOmit.has('Deposit') ? {} as Deposit : mockDeposit({}, relationshipsToOmit)],
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
        field: overrides && overrides.hasOwnProperty('field') ? overrides.field! : CustomersFilterBy.AccountStatus,
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : AccountStatus.Active,
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
        reference: overrides && overrides.hasOwnProperty('reference') ? overrides.reference! : generateMockValue.reference(),
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
        chartOfAccountNonDomiciledCompanyDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountNonDomiciledCompanyDepositAccountsParentCode') ? overrides.chartOfAccountNonDomiciledCompanyDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountPrivateCompanyDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountPrivateCompanyDepositAccountsParentCode') ? overrides.chartOfAccountPrivateCompanyDepositAccountsParentCode! : faker.lorem.word(),
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
        chartOfAccountNonDomiciledIndividualDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountNonDomiciledIndividualDepositAccountsParentCode') ? overrides.chartOfAccountNonDomiciledIndividualDepositAccountsParentCode! : faker.lorem.word(),
        chartOfAccountPrivateCompanyDepositAccountsParentCode: overrides && overrides.hasOwnProperty('chartOfAccountPrivateCompanyDepositAccountsParentCode') ? overrides.chartOfAccountPrivateCompanyDepositAccountsParentCode! : faker.lorem.word(),
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
        dueOutstanding: overrides && overrides.hasOwnProperty('dueOutstanding') ? overrides.dueOutstanding! : relationshipsToOmit.has('Outstanding') ? {} as Outstanding : mockOutstanding({}, relationshipsToOmit),
        outstanding: overrides && overrides.hasOwnProperty('outstanding') ? overrides.outstanding! : relationshipsToOmit.has('Outstanding') ? {} as Outstanding : mockOutstanding({}, relationshipsToOmit),
        total: overrides && overrides.hasOwnProperty('total') ? overrides.total! : relationshipsToOmit.has('Total') ? {} as Total : mockTotal({}, relationshipsToOmit),
    };
};

export const mockDocument = (overrides?: Partial<Document>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Document' } & Document => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Document');
    return {
        __typename: 'Document',
        customerId: overrides && overrides.hasOwnProperty('customerId') ? overrides.customerId! : generateMockValue.uuid(),
        documentId: overrides && overrides.hasOwnProperty('documentId') ? overrides.documentId! : generateMockValue.uuid(),
        filename: overrides && overrides.hasOwnProperty('filename') ? overrides.filename! : generateMockValue.filename(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        status: overrides && overrides.hasOwnProperty('status') ? overrides.status! : mockEnums.documentStatus(),
    };
};

export const mockDocumentArchiveInput = (overrides?: Partial<DocumentArchiveInput>, _relationshipsToOmit: Set<string> = new Set()): DocumentArchiveInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DocumentArchiveInput');
    return {
        documentId: overrides && overrides.hasOwnProperty('documentId') ? overrides.documentId! : generateMockValue.uuid(),
    };
};

export const mockDocumentArchivePayload = (overrides?: Partial<DocumentArchivePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DocumentArchivePayload' } & DocumentArchivePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DocumentArchivePayload');
    return {
        __typename: 'DocumentArchivePayload',
        document: overrides && overrides.hasOwnProperty('document') ? overrides.document! : relationshipsToOmit.has('Document') ? {} as Document : mockDocument({}, relationshipsToOmit),
    };
};

export const mockDocumentCreateInput = (overrides?: Partial<DocumentCreateInput>, _relationshipsToOmit: Set<string> = new Set()): DocumentCreateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DocumentCreateInput');
    return {
        customerId: overrides && overrides.hasOwnProperty('customerId') ? overrides.customerId! : generateMockValue.uuid(),
        file: overrides && overrides.hasOwnProperty('file') ? overrides.file! : faker.lorem.word(),
    };
};

export const mockDocumentCreatePayload = (overrides?: Partial<DocumentCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DocumentCreatePayload' } & DocumentCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DocumentCreatePayload');
    return {
        __typename: 'DocumentCreatePayload',
        document: overrides && overrides.hasOwnProperty('document') ? overrides.document! : relationshipsToOmit.has('Document') ? {} as Document : mockDocument({}, relationshipsToOmit),
    };
};

export const mockDocumentDeleteInput = (overrides?: Partial<DocumentDeleteInput>, _relationshipsToOmit: Set<string> = new Set()): DocumentDeleteInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DocumentDeleteInput');
    return {
        documentId: overrides && overrides.hasOwnProperty('documentId') ? overrides.documentId! : generateMockValue.uuid(),
    };
};

export const mockDocumentDeletePayload = (overrides?: Partial<DocumentDeletePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DocumentDeletePayload' } & DocumentDeletePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DocumentDeletePayload');
    return {
        __typename: 'DocumentDeletePayload',
        deletedDocumentId: overrides && overrides.hasOwnProperty('deletedDocumentId') ? overrides.deletedDocumentId! : generateMockValue.uuid(),
    };
};

export const mockDocumentDownloadLinksGenerateInput = (overrides?: Partial<DocumentDownloadLinksGenerateInput>, _relationshipsToOmit: Set<string> = new Set()): DocumentDownloadLinksGenerateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DocumentDownloadLinksGenerateInput');
    return {
        documentId: overrides && overrides.hasOwnProperty('documentId') ? overrides.documentId! : generateMockValue.uuid(),
    };
};

export const mockDocumentDownloadLinksGeneratePayload = (overrides?: Partial<DocumentDownloadLinksGeneratePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'DocumentDownloadLinksGeneratePayload' } & DocumentDownloadLinksGeneratePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('DocumentDownloadLinksGeneratePayload');
    return {
        __typename: 'DocumentDownloadLinksGeneratePayload',
        documentId: overrides && overrides.hasOwnProperty('documentId') ? overrides.documentId! : generateMockValue.uuid(),
        link: overrides && overrides.hasOwnProperty('link') ? overrides.link! : faker.lorem.word(),
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

export const mockFacilityCvl = (overrides?: Partial<FacilityCvl>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'FacilityCVL' } & FacilityCvl => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('FacilityCvl');
    return {
        __typename: 'FacilityCVL',
        disbursed: overrides && overrides.hasOwnProperty('disbursed') ? overrides.disbursed! : generateMockValue.int(),
        total: overrides && overrides.hasOwnProperty('total') ? overrides.total! : generateMockValue.int(),
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

export const mockInterest = (overrides?: Partial<Interest>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Interest' } & Interest => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Interest');
    return {
        __typename: 'Interest',
        dueOutstanding: overrides && overrides.hasOwnProperty('dueOutstanding') ? overrides.dueOutstanding! : relationshipsToOmit.has('Outstanding') ? {} as Outstanding : mockOutstanding({}, relationshipsToOmit),
        outstanding: overrides && overrides.hasOwnProperty('outstanding') ? overrides.outstanding! : relationshipsToOmit.has('Outstanding') ? {} as Outstanding : mockOutstanding({}, relationshipsToOmit),
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
        history: overrides && overrides.hasOwnProperty('history') ? overrides.history! : relationshipsToOmit.has('JournalEntryConnection') ? {} as JournalEntryConnection : mockJournalEntryConnection({}, relationshipsToOmit),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : generateMockValue.uuid(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
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

export const mockLedgerAccountConnection = (overrides?: Partial<LedgerAccountConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LedgerAccountConnection' } & LedgerAccountConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LedgerAccountConnection');
    return {
        __typename: 'LedgerAccountConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('LedgerAccountEdge') ? {} as LedgerAccountEdge : mockLedgerAccountEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
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
        accountingCsv: overrides && overrides.hasOwnProperty('accountingCsv') ? overrides.accountingCsv! : relationshipsToOmit.has('AccountingCsv') ? {} as AccountingCsv : mockAccountingCsv({}, relationshipsToOmit),
    };
};

export const mockLedgerAccountEdge = (overrides?: Partial<LedgerAccountEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'LedgerAccountEdge' } & LedgerAccountEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('LedgerAccountEdge');
    return {
        __typename: 'LedgerAccountEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
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
        entries: overrides && overrides.hasOwnProperty('entries') ? overrides.entries! : [relationshipsToOmit.has('JournalEntry') ? {} as JournalEntry : mockJournalEntry({}, relationshipsToOmit)],
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
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

export const mockLoan = (overrides?: Partial<Loan>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Loan' } & Loan => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Loan');
    return {
        __typename: 'Loan',
        collateralToMatchInitialCvl: overrides && overrides.hasOwnProperty('collateralToMatchInitialCvl') ? overrides.collateralToMatchInitialCvl! : generateMockValue.satoshis(),
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

export const mockMutation = (overrides?: Partial<Mutation>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Mutation' } & Mutation => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Mutation');
    return {
        __typename: 'Mutation',
        accountingCsvDownloadLinkGenerate: overrides && overrides.hasOwnProperty('accountingCsvDownloadLinkGenerate') ? overrides.accountingCsvDownloadLinkGenerate! : relationshipsToOmit.has('AccountingCsvDownloadLinkGeneratePayload') ? {} as AccountingCsvDownloadLinkGeneratePayload : mockAccountingCsvDownloadLinkGeneratePayload({}, relationshipsToOmit),
        approvalProcessApprove: overrides && overrides.hasOwnProperty('approvalProcessApprove') ? overrides.approvalProcessApprove! : relationshipsToOmit.has('ApprovalProcessApprovePayload') ? {} as ApprovalProcessApprovePayload : mockApprovalProcessApprovePayload({}, relationshipsToOmit),
        approvalProcessDeny: overrides && overrides.hasOwnProperty('approvalProcessDeny') ? overrides.approvalProcessDeny! : relationshipsToOmit.has('ApprovalProcessDenyPayload') ? {} as ApprovalProcessDenyPayload : mockApprovalProcessDenyPayload({}, relationshipsToOmit),
        balanceSheetConfigure: overrides && overrides.hasOwnProperty('balanceSheetConfigure') ? overrides.balanceSheetConfigure! : relationshipsToOmit.has('BalanceSheetModuleConfigurePayload') ? {} as BalanceSheetModuleConfigurePayload : mockBalanceSheetModuleConfigurePayload({}, relationshipsToOmit),
        chartOfAccountsCsvImport: overrides && overrides.hasOwnProperty('chartOfAccountsCsvImport') ? overrides.chartOfAccountsCsvImport! : relationshipsToOmit.has('ChartOfAccountsCsvImportPayload') ? {} as ChartOfAccountsCsvImportPayload : mockChartOfAccountsCsvImportPayload({}, relationshipsToOmit),
        committeeAddUser: overrides && overrides.hasOwnProperty('committeeAddUser') ? overrides.committeeAddUser! : relationshipsToOmit.has('CommitteeAddUserPayload') ? {} as CommitteeAddUserPayload : mockCommitteeAddUserPayload({}, relationshipsToOmit),
        committeeCreate: overrides && overrides.hasOwnProperty('committeeCreate') ? overrides.committeeCreate! : relationshipsToOmit.has('CommitteeCreatePayload') ? {} as CommitteeCreatePayload : mockCommitteeCreatePayload({}, relationshipsToOmit),
        committeeRemoveUser: overrides && overrides.hasOwnProperty('committeeRemoveUser') ? overrides.committeeRemoveUser! : relationshipsToOmit.has('CommitteeRemoveUserPayload') ? {} as CommitteeRemoveUserPayload : mockCommitteeRemoveUserPayload({}, relationshipsToOmit),
        creditFacilityCollateralUpdate: overrides && overrides.hasOwnProperty('creditFacilityCollateralUpdate') ? overrides.creditFacilityCollateralUpdate! : relationshipsToOmit.has('CreditFacilityCollateralUpdatePayload') ? {} as CreditFacilityCollateralUpdatePayload : mockCreditFacilityCollateralUpdatePayload({}, relationshipsToOmit),
        creditFacilityComplete: overrides && overrides.hasOwnProperty('creditFacilityComplete') ? overrides.creditFacilityComplete! : relationshipsToOmit.has('CreditFacilityCompletePayload') ? {} as CreditFacilityCompletePayload : mockCreditFacilityCompletePayload({}, relationshipsToOmit),
        creditFacilityCreate: overrides && overrides.hasOwnProperty('creditFacilityCreate') ? overrides.creditFacilityCreate! : relationshipsToOmit.has('CreditFacilityCreatePayload') ? {} as CreditFacilityCreatePayload : mockCreditFacilityCreatePayload({}, relationshipsToOmit),
        creditFacilityDisbursalInitiate: overrides && overrides.hasOwnProperty('creditFacilityDisbursalInitiate') ? overrides.creditFacilityDisbursalInitiate! : relationshipsToOmit.has('CreditFacilityDisbursalInitiatePayload') ? {} as CreditFacilityDisbursalInitiatePayload : mockCreditFacilityDisbursalInitiatePayload({}, relationshipsToOmit),
        creditFacilityPartialPayment: overrides && overrides.hasOwnProperty('creditFacilityPartialPayment') ? overrides.creditFacilityPartialPayment! : relationshipsToOmit.has('CreditFacilityPartialPaymentPayload') ? {} as CreditFacilityPartialPaymentPayload : mockCreditFacilityPartialPaymentPayload({}, relationshipsToOmit),
        creditModuleConfigure: overrides && overrides.hasOwnProperty('creditModuleConfigure') ? overrides.creditModuleConfigure! : relationshipsToOmit.has('CreditModuleConfigurePayload') ? {} as CreditModuleConfigurePayload : mockCreditModuleConfigurePayload({}, relationshipsToOmit),
        customerCreate: overrides && overrides.hasOwnProperty('customerCreate') ? overrides.customerCreate! : relationshipsToOmit.has('CustomerCreatePayload') ? {} as CustomerCreatePayload : mockCustomerCreatePayload({}, relationshipsToOmit),
        customerDocumentAttach: overrides && overrides.hasOwnProperty('customerDocumentAttach') ? overrides.customerDocumentAttach! : relationshipsToOmit.has('DocumentCreatePayload') ? {} as DocumentCreatePayload : mockDocumentCreatePayload({}, relationshipsToOmit),
        customerEmailUpdate: overrides && overrides.hasOwnProperty('customerEmailUpdate') ? overrides.customerEmailUpdate! : relationshipsToOmit.has('CustomerEmailUpdatePayload') ? {} as CustomerEmailUpdatePayload : mockCustomerEmailUpdatePayload({}, relationshipsToOmit),
        customerTelegramIdUpdate: overrides && overrides.hasOwnProperty('customerTelegramIdUpdate') ? overrides.customerTelegramIdUpdate! : relationshipsToOmit.has('CustomerTelegramIdUpdatePayload') ? {} as CustomerTelegramIdUpdatePayload : mockCustomerTelegramIdUpdatePayload({}, relationshipsToOmit),
        depositModuleConfigure: overrides && overrides.hasOwnProperty('depositModuleConfigure') ? overrides.depositModuleConfigure! : relationshipsToOmit.has('DepositModuleConfigurePayload') ? {} as DepositModuleConfigurePayload : mockDepositModuleConfigurePayload({}, relationshipsToOmit),
        depositRecord: overrides && overrides.hasOwnProperty('depositRecord') ? overrides.depositRecord! : relationshipsToOmit.has('DepositRecordPayload') ? {} as DepositRecordPayload : mockDepositRecordPayload({}, relationshipsToOmit),
        documentArchive: overrides && overrides.hasOwnProperty('documentArchive') ? overrides.documentArchive! : relationshipsToOmit.has('DocumentArchivePayload') ? {} as DocumentArchivePayload : mockDocumentArchivePayload({}, relationshipsToOmit),
        documentDelete: overrides && overrides.hasOwnProperty('documentDelete') ? overrides.documentDelete! : relationshipsToOmit.has('DocumentDeletePayload') ? {} as DocumentDeletePayload : mockDocumentDeletePayload({}, relationshipsToOmit),
        documentDownloadLinkGenerate: overrides && overrides.hasOwnProperty('documentDownloadLinkGenerate') ? overrides.documentDownloadLinkGenerate! : relationshipsToOmit.has('DocumentDownloadLinksGeneratePayload') ? {} as DocumentDownloadLinksGeneratePayload : mockDocumentDownloadLinksGeneratePayload({}, relationshipsToOmit),
        ledgerAccountCsvCreate: overrides && overrides.hasOwnProperty('ledgerAccountCsvCreate') ? overrides.ledgerAccountCsvCreate! : relationshipsToOmit.has('LedgerAccountCsvCreatePayload') ? {} as LedgerAccountCsvCreatePayload : mockLedgerAccountCsvCreatePayload({}, relationshipsToOmit),
        manualTransactionExecute: overrides && overrides.hasOwnProperty('manualTransactionExecute') ? overrides.manualTransactionExecute! : relationshipsToOmit.has('ManualTransactionExecutePayload') ? {} as ManualTransactionExecutePayload : mockManualTransactionExecutePayload({}, relationshipsToOmit),
        policyAssignCommittee: overrides && overrides.hasOwnProperty('policyAssignCommittee') ? overrides.policyAssignCommittee! : relationshipsToOmit.has('PolicyAssignCommitteePayload') ? {} as PolicyAssignCommitteePayload : mockPolicyAssignCommitteePayload({}, relationshipsToOmit),
        profitAndLossStatementConfigure: overrides && overrides.hasOwnProperty('profitAndLossStatementConfigure') ? overrides.profitAndLossStatementConfigure! : relationshipsToOmit.has('ProfitAndLossStatementModuleConfigurePayload') ? {} as ProfitAndLossStatementModuleConfigurePayload : mockProfitAndLossStatementModuleConfigurePayload({}, relationshipsToOmit),
        reportCreate: overrides && overrides.hasOwnProperty('reportCreate') ? overrides.reportCreate! : relationshipsToOmit.has('ReportCreatePayload') ? {} as ReportCreatePayload : mockReportCreatePayload({}, relationshipsToOmit),
        reportDownloadLinksGenerate: overrides && overrides.hasOwnProperty('reportDownloadLinksGenerate') ? overrides.reportDownloadLinksGenerate! : relationshipsToOmit.has('ReportDownloadLinksGeneratePayload') ? {} as ReportDownloadLinksGeneratePayload : mockReportDownloadLinksGeneratePayload({}, relationshipsToOmit),
        roleAddPermissionSets: overrides && overrides.hasOwnProperty('roleAddPermissionSets') ? overrides.roleAddPermissionSets! : relationshipsToOmit.has('RoleAddPermissionSetsPayload') ? {} as RoleAddPermissionSetsPayload : mockRoleAddPermissionSetsPayload({}, relationshipsToOmit),
        roleCreate: overrides && overrides.hasOwnProperty('roleCreate') ? overrides.roleCreate! : relationshipsToOmit.has('RoleCreatePayload') ? {} as RoleCreatePayload : mockRoleCreatePayload({}, relationshipsToOmit),
        roleRemovePermissionSets: overrides && overrides.hasOwnProperty('roleRemovePermissionSets') ? overrides.roleRemovePermissionSets! : relationshipsToOmit.has('RoleRemovePermissionSetsPayload') ? {} as RoleRemovePermissionSetsPayload : mockRoleRemovePermissionSetsPayload({}, relationshipsToOmit),
        sumsubPermalinkCreate: overrides && overrides.hasOwnProperty('sumsubPermalinkCreate') ? overrides.sumsubPermalinkCreate! : relationshipsToOmit.has('SumsubPermalinkCreatePayload') ? {} as SumsubPermalinkCreatePayload : mockSumsubPermalinkCreatePayload({}, relationshipsToOmit),
        termsTemplateCreate: overrides && overrides.hasOwnProperty('termsTemplateCreate') ? overrides.termsTemplateCreate! : relationshipsToOmit.has('TermsTemplateCreatePayload') ? {} as TermsTemplateCreatePayload : mockTermsTemplateCreatePayload({}, relationshipsToOmit),
        termsTemplateUpdate: overrides && overrides.hasOwnProperty('termsTemplateUpdate') ? overrides.termsTemplateUpdate! : relationshipsToOmit.has('TermsTemplateUpdatePayload') ? {} as TermsTemplateUpdatePayload : mockTermsTemplateUpdatePayload({}, relationshipsToOmit),
        userAssignRole: overrides && overrides.hasOwnProperty('userAssignRole') ? overrides.userAssignRole! : relationshipsToOmit.has('UserAssignRolePayload') ? {} as UserAssignRolePayload : mockUserAssignRolePayload({}, relationshipsToOmit),
        userCreate: overrides && overrides.hasOwnProperty('userCreate') ? overrides.userCreate! : relationshipsToOmit.has('UserCreatePayload') ? {} as UserCreatePayload : mockUserCreatePayload({}, relationshipsToOmit),
        userRevokeRole: overrides && overrides.hasOwnProperty('userRevokeRole') ? overrides.userRevokeRole! : relationshipsToOmit.has('UserRevokeRolePayload') ? {} as UserRevokeRolePayload : mockUserRevokeRolePayload({}, relationshipsToOmit),
        withdrawalCancel: overrides && overrides.hasOwnProperty('withdrawalCancel') ? overrides.withdrawalCancel! : relationshipsToOmit.has('WithdrawalCancelPayload') ? {} as WithdrawalCancelPayload : mockWithdrawalCancelPayload({}, relationshipsToOmit),
        withdrawalConfirm: overrides && overrides.hasOwnProperty('withdrawalConfirm') ? overrides.withdrawalConfirm! : relationshipsToOmit.has('WithdrawalConfirmPayload') ? {} as WithdrawalConfirmPayload : mockWithdrawalConfirmPayload({}, relationshipsToOmit),
        withdrawalInitiate: overrides && overrides.hasOwnProperty('withdrawalInitiate') ? overrides.withdrawalInitiate! : relationshipsToOmit.has('WithdrawalInitiatePayload') ? {} as WithdrawalInitiatePayload : mockWithdrawalInitiatePayload({}, relationshipsToOmit),
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
        approvalProcessType: overrides && overrides.hasOwnProperty('approvalProcessType') ? overrides.approvalProcessType! : ApprovalProcessType.CreditFacilityApproval,
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
        accountingCsvsForLedgerAccountId: overrides && overrides.hasOwnProperty('accountingCsvsForLedgerAccountId') ? overrides.accountingCsvsForLedgerAccountId! : relationshipsToOmit.has('AccountingCsvConnection') ? {} as AccountingCsvConnection : mockAccountingCsvConnection({}, relationshipsToOmit),
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
        customer: overrides && overrides.hasOwnProperty('customer') ? overrides.customer! : relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit),
        customerByEmail: overrides && overrides.hasOwnProperty('customerByEmail') ? overrides.customerByEmail! : relationshipsToOmit.has('Customer') ? {} as Customer : mockCustomer({}, relationshipsToOmit),
        customers: overrides && overrides.hasOwnProperty('customers') ? overrides.customers! : relationshipsToOmit.has('CustomerConnection') ? {} as CustomerConnection : mockCustomerConnection({}, relationshipsToOmit),
        dashboard: overrides && overrides.hasOwnProperty('dashboard') ? overrides.dashboard! : relationshipsToOmit.has('Dashboard') ? {} as Dashboard : mockDashboard({}, relationshipsToOmit),
        deposit: overrides && overrides.hasOwnProperty('deposit') ? overrides.deposit! : relationshipsToOmit.has('Deposit') ? {} as Deposit : mockDeposit({}, relationshipsToOmit),
        depositConfig: overrides && overrides.hasOwnProperty('depositConfig') ? overrides.depositConfig! : relationshipsToOmit.has('DepositModuleConfig') ? {} as DepositModuleConfig : mockDepositModuleConfig({}, relationshipsToOmit),
        deposits: overrides && overrides.hasOwnProperty('deposits') ? overrides.deposits! : relationshipsToOmit.has('DepositConnection') ? {} as DepositConnection : mockDepositConnection({}, relationshipsToOmit),
        disbursal: overrides && overrides.hasOwnProperty('disbursal') ? overrides.disbursal! : relationshipsToOmit.has('CreditFacilityDisbursal') ? {} as CreditFacilityDisbursal : mockCreditFacilityDisbursal({}, relationshipsToOmit),
        disbursals: overrides && overrides.hasOwnProperty('disbursals') ? overrides.disbursals! : relationshipsToOmit.has('CreditFacilityDisbursalConnection') ? {} as CreditFacilityDisbursalConnection : mockCreditFacilityDisbursalConnection({}, relationshipsToOmit),
        document: overrides && overrides.hasOwnProperty('document') ? overrides.document! : relationshipsToOmit.has('Document') ? {} as Document : mockDocument({}, relationshipsToOmit),
        journalEntries: overrides && overrides.hasOwnProperty('journalEntries') ? overrides.journalEntries! : relationshipsToOmit.has('JournalEntryConnection') ? {} as JournalEntryConnection : mockJournalEntryConnection({}, relationshipsToOmit),
        ledgerAccount: overrides && overrides.hasOwnProperty('ledgerAccount') ? overrides.ledgerAccount! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        ledgerAccountByCode: overrides && overrides.hasOwnProperty('ledgerAccountByCode') ? overrides.ledgerAccountByCode! : relationshipsToOmit.has('LedgerAccount') ? {} as LedgerAccount : mockLedgerAccount({}, relationshipsToOmit),
        ledgerAccountCsvCreate: overrides && overrides.hasOwnProperty('ledgerAccountCsvCreate') ? overrides.ledgerAccountCsvCreate! : relationshipsToOmit.has('LedgerAccountCsvCreatePayload') ? {} as LedgerAccountCsvCreatePayload : mockLedgerAccountCsvCreatePayload({}, relationshipsToOmit),
        ledgerTransaction: overrides && overrides.hasOwnProperty('ledgerTransaction') ? overrides.ledgerTransaction! : relationshipsToOmit.has('LedgerTransaction') ? {} as LedgerTransaction : mockLedgerTransaction({}, relationshipsToOmit),
        ledgerTransactionsForTemplateCode: overrides && overrides.hasOwnProperty('ledgerTransactionsForTemplateCode') ? overrides.ledgerTransactionsForTemplateCode! : relationshipsToOmit.has('LedgerTransactionConnection') ? {} as LedgerTransactionConnection : mockLedgerTransactionConnection({}, relationshipsToOmit),
        me: overrides && overrides.hasOwnProperty('me') ? overrides.me! : relationshipsToOmit.has('Subject') ? {} as Subject : mockSubject({}, relationshipsToOmit),
        permissionSets: overrides && overrides.hasOwnProperty('permissionSets') ? overrides.permissionSets! : relationshipsToOmit.has('PermissionSetConnection') ? {} as PermissionSetConnection : mockPermissionSetConnection({}, relationshipsToOmit),
        policies: overrides && overrides.hasOwnProperty('policies') ? overrides.policies! : relationshipsToOmit.has('PolicyConnection') ? {} as PolicyConnection : mockPolicyConnection({}, relationshipsToOmit),
        policy: overrides && overrides.hasOwnProperty('policy') ? overrides.policy! : relationshipsToOmit.has('Policy') ? {} as Policy : mockPolicy({}, relationshipsToOmit),
        profitAndLossStatement: overrides && overrides.hasOwnProperty('profitAndLossStatement') ? overrides.profitAndLossStatement! : relationshipsToOmit.has('ProfitAndLossStatement') ? {} as ProfitAndLossStatement : mockProfitAndLossStatement({}, relationshipsToOmit),
        profitAndLossStatementConfig: overrides && overrides.hasOwnProperty('profitAndLossStatementConfig') ? overrides.profitAndLossStatementConfig! : relationshipsToOmit.has('ProfitAndLossStatementModuleConfig') ? {} as ProfitAndLossStatementModuleConfig : mockProfitAndLossStatementModuleConfig({}, relationshipsToOmit),
        realtimePrice: overrides && overrides.hasOwnProperty('realtimePrice') ? overrides.realtimePrice! : relationshipsToOmit.has('RealtimePrice') ? {} as RealtimePrice : mockRealtimePrice({}, relationshipsToOmit),
        report: overrides && overrides.hasOwnProperty('report') ? overrides.report! : relationshipsToOmit.has('Report') ? {} as Report : mockReport({}, relationshipsToOmit),
        reports: overrides && overrides.hasOwnProperty('reports') ? overrides.reports! : [relationshipsToOmit.has('Report') ? {} as Report : mockReport({}, relationshipsToOmit)],
        role: overrides && overrides.hasOwnProperty('role') ? overrides.role! : relationshipsToOmit.has('RoleEntity') ? {} as RoleEntity : mockRoleEntity({}, relationshipsToOmit),
        roles: overrides && overrides.hasOwnProperty('roles') ? overrides.roles! : relationshipsToOmit.has('RoleEntityConnection') ? {} as RoleEntityConnection : mockRoleEntityConnection({}, relationshipsToOmit),
        termsTemplate: overrides && overrides.hasOwnProperty('termsTemplate') ? overrides.termsTemplate! : relationshipsToOmit.has('TermsTemplate') ? {} as TermsTemplate : mockTermsTemplate({}, relationshipsToOmit),
        termsTemplates: overrides && overrides.hasOwnProperty('termsTemplates') ? overrides.termsTemplates! : [relationshipsToOmit.has('TermsTemplate') ? {} as TermsTemplate : mockTermsTemplate({}, relationshipsToOmit)],
        transactionTemplates: overrides && overrides.hasOwnProperty('transactionTemplates') ? overrides.transactionTemplates! : relationshipsToOmit.has('TransactionTemplateConnection') ? {} as TransactionTemplateConnection : mockTransactionTemplateConnection({}, relationshipsToOmit),
        trialBalance: overrides && overrides.hasOwnProperty('trialBalance') ? overrides.trialBalance! : relationshipsToOmit.has('TrialBalance') ? {} as TrialBalance : mockTrialBalance({}, relationshipsToOmit),
        user: overrides && overrides.hasOwnProperty('user') ? overrides.user! : relationshipsToOmit.has('User') ? {} as User : mockUser({}, relationshipsToOmit),
        users: overrides && overrides.hasOwnProperty('users') ? overrides.users! : [relationshipsToOmit.has('User') ? {} as User : mockUser({}, relationshipsToOmit)],
        withdrawal: overrides && overrides.hasOwnProperty('withdrawal') ? overrides.withdrawal! : relationshipsToOmit.has('Withdrawal') ? {} as Withdrawal : mockWithdrawal({}, relationshipsToOmit),
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
        lastError: overrides && overrides.hasOwnProperty('lastError') ? overrides.lastError! : faker.lorem.word(),
        progress: overrides && overrides.hasOwnProperty('progress') ? overrides.progress! : ReportProgress.Complete,
        reportId: overrides && overrides.hasOwnProperty('reportId') ? overrides.reportId! : generateMockValue.uuid(),
    };
};

export const mockReportCreatePayload = (overrides?: Partial<ReportCreatePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ReportCreatePayload' } & ReportCreatePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ReportCreatePayload');
    return {
        __typename: 'ReportCreatePayload',
        report: overrides && overrides.hasOwnProperty('report') ? overrides.report! : relationshipsToOmit.has('Report') ? {} as Report : mockReport({}, relationshipsToOmit),
    };
};

export const mockReportDownloadLink = (overrides?: Partial<ReportDownloadLink>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ReportDownloadLink' } & ReportDownloadLink => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ReportDownloadLink');
    return {
        __typename: 'ReportDownloadLink',
        reportName: overrides && overrides.hasOwnProperty('reportName') ? overrides.reportName! : faker.lorem.word(),
        url: overrides && overrides.hasOwnProperty('url') ? overrides.url! : generateMockValue.url(),
    };
};

export const mockReportDownloadLinksGenerateInput = (overrides?: Partial<ReportDownloadLinksGenerateInput>, _relationshipsToOmit: Set<string> = new Set()): ReportDownloadLinksGenerateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ReportDownloadLinksGenerateInput');
    return {
        reportId: overrides && overrides.hasOwnProperty('reportId') ? overrides.reportId! : generateMockValue.uuid(),
    };
};

export const mockReportDownloadLinksGeneratePayload = (overrides?: Partial<ReportDownloadLinksGeneratePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'ReportDownloadLinksGeneratePayload' } & ReportDownloadLinksGeneratePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('ReportDownloadLinksGeneratePayload');
    return {
        __typename: 'ReportDownloadLinksGeneratePayload',
        links: overrides && overrides.hasOwnProperty('links') ? overrides.links! : [relationshipsToOmit.has('ReportDownloadLink') ? {} as ReportDownloadLink : mockReportDownloadLink({}, relationshipsToOmit)],
        reportId: overrides && overrides.hasOwnProperty('reportId') ? overrides.reportId! : generateMockValue.uuid(),
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
        role: overrides && overrides.hasOwnProperty('role') ? overrides.role! : relationshipsToOmit.has('RoleEntity') ? {} as RoleEntity : mockRoleEntity({}, relationshipsToOmit),
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
        role: overrides && overrides.hasOwnProperty('role') ? overrides.role! : relationshipsToOmit.has('RoleEntity') ? {} as RoleEntity : mockRoleEntity({}, relationshipsToOmit),
    };
};

export const mockRoleEntity = (overrides?: Partial<RoleEntity>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'RoleEntity' } & RoleEntity => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('RoleEntity');
    return {
        __typename: 'RoleEntity',
        createdAt: overrides && overrides.hasOwnProperty('createdAt') ? overrides.createdAt! : generateMockValue.timestamp(),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : faker.string.uuid(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        permissionSets: overrides && overrides.hasOwnProperty('permissionSets') ? overrides.permissionSets! : [relationshipsToOmit.has('PermissionSet') ? {} as PermissionSet : mockPermissionSet({}, relationshipsToOmit)],
        roleId: overrides && overrides.hasOwnProperty('roleId') ? overrides.roleId! : generateMockValue.uuid(),
    };
};

export const mockRoleEntityConnection = (overrides?: Partial<RoleEntityConnection>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'RoleEntityConnection' } & RoleEntityConnection => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('RoleEntityConnection');
    return {
        __typename: 'RoleEntityConnection',
        edges: overrides && overrides.hasOwnProperty('edges') ? overrides.edges! : [relationshipsToOmit.has('RoleEntityEdge') ? {} as RoleEntityEdge : mockRoleEntityEdge({}, relationshipsToOmit)],
        nodes: overrides && overrides.hasOwnProperty('nodes') ? overrides.nodes! : [relationshipsToOmit.has('RoleEntity') ? {} as RoleEntity : mockRoleEntity({}, relationshipsToOmit)],
        pageInfo: overrides && overrides.hasOwnProperty('pageInfo') ? overrides.pageInfo! : relationshipsToOmit.has('PageInfo') ? {} as PageInfo : mockPageInfo({}, relationshipsToOmit),
    };
};

export const mockRoleEntityEdge = (overrides?: Partial<RoleEntityEdge>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'RoleEntityEdge' } & RoleEntityEdge => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('RoleEntityEdge');
    return {
        __typename: 'RoleEntityEdge',
        cursor: overrides && overrides.hasOwnProperty('cursor') ? overrides.cursor! : generateMockValue.cursor(),
        node: overrides && overrides.hasOwnProperty('node') ? overrides.node! : relationshipsToOmit.has('RoleEntity') ? {} as RoleEntity : mockRoleEntity({}, relationshipsToOmit),
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
        role: overrides && overrides.hasOwnProperty('role') ? overrides.role! : relationshipsToOmit.has('RoleEntity') ? {} as RoleEntity : mockRoleEntity({}, relationshipsToOmit),
    };
};

export const mockSubject = (overrides?: Partial<Subject>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'Subject' } & Subject => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('Subject');
    return {
        __typename: 'Subject',
        subjectCanCreateCustomer: overrides && overrides.hasOwnProperty('subjectCanCreateCustomer') ? overrides.subjectCanCreateCustomer! : faker.datatype.boolean(),
        subjectCanCreateTermsTemplate: overrides && overrides.hasOwnProperty('subjectCanCreateTermsTemplate') ? overrides.subjectCanCreateTermsTemplate! : faker.datatype.boolean(),
        subjectCanCreateUser: overrides && overrides.hasOwnProperty('subjectCanCreateUser') ? overrides.subjectCanCreateUser! : faker.datatype.boolean(),
        user: overrides && overrides.hasOwnProperty('user') ? overrides.user! : relationshipsToOmit.has('User') ? {} as User : mockUser({}, relationshipsToOmit),
        visibleNavigationItems: overrides && overrides.hasOwnProperty('visibleNavigationItems') ? overrides.visibleNavigationItems! : relationshipsToOmit.has('VisibleNavigationItems') ? {} as VisibleNavigationItems : mockVisibleNavigationItems({}, relationshipsToOmit),
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
        duration: overrides && overrides.hasOwnProperty('duration') ? overrides.duration! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        initialCvl: overrides && overrides.hasOwnProperty('initialCvl') ? overrides.initialCvl! : generateMockValue.int(),
        interestDueDuration: overrides && overrides.hasOwnProperty('interestDueDuration') ? overrides.interestDueDuration! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        liquidationCvl: overrides && overrides.hasOwnProperty('liquidationCvl') ? overrides.liquidationCvl! : generateMockValue.int(),
        marginCallCvl: overrides && overrides.hasOwnProperty('marginCallCvl') ? overrides.marginCallCvl! : generateMockValue.int(),
        obligationOverdueDuration: overrides && overrides.hasOwnProperty('obligationOverdueDuration') ? overrides.obligationOverdueDuration! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
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
        subjectCanUpdateTermsTemplate: overrides && overrides.hasOwnProperty('subjectCanUpdateTermsTemplate') ? overrides.subjectCanUpdateTermsTemplate! : faker.datatype.boolean(),
        termsId: overrides && overrides.hasOwnProperty('termsId') ? overrides.termsId! : generateMockValue.uuid(),
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
        duration: overrides && overrides.hasOwnProperty('duration') ? overrides.duration! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        initialCvl: overrides && overrides.hasOwnProperty('initialCvl') ? overrides.initialCvl! : generateMockValue.int(),
        interestDueDuration: overrides && overrides.hasOwnProperty('interestDueDuration') ? overrides.interestDueDuration! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        liquidationCvl: overrides && overrides.hasOwnProperty('liquidationCvl') ? overrides.liquidationCvl! : generateMockValue.int(),
        marginCallCvl: overrides && overrides.hasOwnProperty('marginCallCvl') ? overrides.marginCallCvl! : generateMockValue.int(),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        obligationOverdueDuration: overrides && overrides.hasOwnProperty('obligationOverdueDuration') ? overrides.obligationOverdueDuration! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
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
        duration: overrides && overrides.hasOwnProperty('duration') ? overrides.duration! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : generateMockValue.uuid(),
        initialCvl: overrides && overrides.hasOwnProperty('initialCvl') ? overrides.initialCvl! : generateMockValue.int(),
        interestDueDuration: overrides && overrides.hasOwnProperty('interestDueDuration') ? overrides.interestDueDuration! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
        liquidationCvl: overrides && overrides.hasOwnProperty('liquidationCvl') ? overrides.liquidationCvl! : generateMockValue.int(),
        marginCallCvl: overrides && overrides.hasOwnProperty('marginCallCvl') ? overrides.marginCallCvl! : generateMockValue.int(),
        obligationOverdueDuration: overrides && overrides.hasOwnProperty('obligationOverdueDuration') ? overrides.obligationOverdueDuration! : relationshipsToOmit.has('DurationInput') ? {} as DurationInput : mockDurationInput({}, relationshipsToOmit),
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
        accounts: overrides && overrides.hasOwnProperty('accounts') ? overrides.accounts! : relationshipsToOmit.has('LedgerAccountConnection') ? {} as LedgerAccountConnection : mockLedgerAccountConnection({}, relationshipsToOmit),
        name: overrides && overrides.hasOwnProperty('name') ? overrides.name! : generateMockValue.name(),
        total: overrides && overrides.hasOwnProperty('total') ? overrides.total! : relationshipsToOmit.has('LedgerAccountBalanceRangeByCurrency') ? {} as LedgerAccountBalanceRangeByCurrency : mockLedgerAccountBalanceRangeByCurrency({}, relationshipsToOmit),
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
        role: overrides && overrides.hasOwnProperty('role') ? overrides.role! : relationshipsToOmit.has('RoleEntity') ? {} as RoleEntity : mockRoleEntity({}, relationshipsToOmit),
        subjectCanAssignRoleToUser: overrides && overrides.hasOwnProperty('subjectCanAssignRoleToUser') ? overrides.subjectCanAssignRoleToUser! : faker.datatype.boolean(),
        subjectCanRevokeRoleFromUser: overrides && overrides.hasOwnProperty('subjectCanRevokeRoleFromUser') ? overrides.subjectCanRevokeRoleFromUser! : faker.datatype.boolean(),
        userId: overrides && overrides.hasOwnProperty('userId') ? overrides.userId! : generateMockValue.uuid(),
    };
};

export const mockUserAssignRoleInput = (overrides?: Partial<UserAssignRoleInput>, _relationshipsToOmit: Set<string> = new Set()): UserAssignRoleInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UserAssignRoleInput');
    return {
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : generateMockValue.uuid(),
        roleId: overrides && overrides.hasOwnProperty('roleId') ? overrides.roleId! : generateMockValue.uuid(),
    };
};

export const mockUserAssignRolePayload = (overrides?: Partial<UserAssignRolePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'UserAssignRolePayload' } & UserAssignRolePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UserAssignRolePayload');
    return {
        __typename: 'UserAssignRolePayload',
        user: overrides && overrides.hasOwnProperty('user') ? overrides.user! : relationshipsToOmit.has('User') ? {} as User : mockUser({}, relationshipsToOmit),
    };
};

export const mockUserCreateInput = (overrides?: Partial<UserCreateInput>, _relationshipsToOmit: Set<string> = new Set()): UserCreateInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UserCreateInput');
    return {
        email: overrides && overrides.hasOwnProperty('email') ? overrides.email! : generateMockValue.email(),
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

export const mockUserRevokeRoleInput = (overrides?: Partial<UserRevokeRoleInput>, _relationshipsToOmit: Set<string> = new Set()): UserRevokeRoleInput => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UserRevokeRoleInput');
    return {
        id: overrides && overrides.hasOwnProperty('id') ? overrides.id! : generateMockValue.uuid(),
        roleId: overrides && overrides.hasOwnProperty('roleId') ? overrides.roleId! : generateMockValue.uuid(),
    };
};

export const mockUserRevokeRolePayload = (overrides?: Partial<UserRevokeRolePayload>, _relationshipsToOmit: Set<string> = new Set()): { __typename: 'UserRevokeRolePayload' } & UserRevokeRolePayload => {
    const relationshipsToOmit: Set<string> = new Set(_relationshipsToOmit);
    relationshipsToOmit.add('UserRevokeRolePayload');
    return {
        __typename: 'UserRevokeRolePayload',
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

export const seedMocks = (seed: number) => faker.seed(seed);
